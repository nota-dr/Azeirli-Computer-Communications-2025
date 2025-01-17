    /************************************************************
     * EX2 – HTTP client
     *
     * Implements a simple HTTP/1.1 client supporting GET requests,
     * optional parameters appended as a query string, and automatic
     * handling of 3XX (HTTP) redirects (now without a fixed limit!).
     *
     * Usage:
     *   client [-r n <pr1=value1 pr2=value2 …>] <URL>
     *
     ************************************************************/

    #include <stdio.h>
    #include <stdlib.h>
    #include <string.h>
    #include <unistd.h>
    #include <sys/types.h>
    #include <sys/socket.h>
    #include <netinet/in.h>
    #include <netdb.h>     // for gethostbyname, herror
    #include <ctype.h>     // for isdigit
    #include <errno.h>

    /* Larger buffer sizes to avoid assumptions. */
    #define REQUEST_BUFFER_SIZE    2048
    #define MAX_BUFFER_SIZE        8192

    /* Reasonable upper limits for this assignment. */
    #define MAX_HOST_LEN           1024
    #define MAX_PATH_LEN           4096
    #define MAX_URL_LEN            8192
    #define MAX_LOCATION_LEN       4096

    /*
     * Data structure to hold command-line results.
     */
    typedef struct {
        char *url;         // URL must start with http://
        int  numParams;    // number of name=value pairs
        char **params;     // array of "name=value" strings
    } CmdArgs;

    /* Function Prototypes */
    static void printUsageAndExit();
    static int  isPositiveNumberUnder16Bit(const char *str);
    static void parseArguments(int argc, char *argv[], CmdArgs *cmd);
    static void parseURL(const char *url, char *host, int *port, char *path);
    static int  buildHTTPRequest(const char *host,
                                 const char *path,
                                 int numParams,
                                 char **params,
                                 char *requestBuffer);
    static int  connectToServer(const char *hostname, int port);
    static int  sendAll(int sockfd, const char *buf, size_t len);
    static int  receiveResponse(int sockfd, unsigned char **response, int *responseSize);
    static int  extractStatusCode(const unsigned char *response);
    static int  extractLocationHeader(const unsigned char *response, char *locationURL);
    static int  isHTTP(const char *maybeURL);

    int main(int argc, char *argv[])
    {
        CmdArgs cmd;
        parseArguments(argc, argv, &cmd);

        // Removed the fixed limit of 10; now we allow “endless” redirects
        // (danger: this can cause infinite loops with bad or circular redirects).
        int redirectCount = 0;

        // Use a larger buffer for the current URL.
        char currentURL[MAX_URL_LEN] = {0};
        strncpy(currentURL, cmd.url, sizeof(currentURL) - 1);

        int originalNumParams = cmd.numParams;
        char **originalParams = cmd.params;

        while (1) {
            /* Parse URL -> host, port, path. If invalid => usage. */
            char host[MAX_HOST_LEN] = {0};
            char path[MAX_PATH_LEN] = {0};
            int  port = 80;
            parseURL(currentURL, host, &port, path);

            /* Build the HTTP request. If error => usage. */
            char request[REQUEST_BUFFER_SIZE] = {0};
            int numParamsToSend = (redirectCount == 0) ? originalNumParams : 0;
            char **paramsToSend = (redirectCount == 0) ? originalParams : NULL;
            if (buildHTTPRequest(host, path, numParamsToSend, paramsToSend, request) < 0) {
                printUsageAndExit();
            }

            /* Print the request per assignment instructions. */
            printf("HTTP request =\n%s\nLEN = %d\n", request, (int)strlen(request));

            /* Connect to server -> system call errors => perror/herror. */
            int sockfd = connectToServer(host, port);
            if (sockfd < 0) {
                /* connectToServer already calls herror/perror. */
                exit(1);
            }

            /* Send request -> system call error => perror. */
            if (sendAll(sockfd, request, strlen(request)) < 0) {
                perror("send");
                close(sockfd);
                exit(1);
            }

            /* Receive response -> system call error => perror. */
            unsigned char *response = NULL;
            int responseSize = 0;
            if (receiveResponse(sockfd, &response, &responseSize) < 0) {
                perror("recv");
                close(sockfd);
                free(response);
                exit(1);
            }
            close(sockfd);

            /* Print the response. */
            if (response) {
                fwrite(response, 1, responseSize, stdout);
                printf("\n   Total received response bytes: %d\n",responseSize);
            }

            /* Check 3XX for redirect. */
            int statusCode = extractStatusCode(response);
            if (statusCode >= 300 && statusCode < 400) {
                /* Extract Location: (ignore case via strncasecmp in extractLocationHeader) */
                char locationURL[MAX_LOCATION_LEN] = {0};
                if (extractLocationHeader(response, locationURL) == 0) {
                    /* Only follow if starts with http:// */
                    if (isHTTP(locationURL)) {
                        free(response);
                        response = NULL;
                        strncpy(currentURL, locationURL, sizeof(currentURL) - 1);
                        redirectCount++;
                        continue;
                    }
                    else {
                        /*
                         * The code below attempts to reuse "http://<host>" from currentURL
                         * to handle a relative location. We do minimal checks here but ensure
                         * we do not overflow.
                         */
                        size_t prefixLen = strlen("http://");
                        if (strlen(currentURL) >= prefixLen + strlen(host)) {
                            // null-terminate right after "http://host"
                            currentURL[prefixLen + strlen(host)] = '\0';
                        }
                        // If the new location doesn't start with '/', add it
                        if (locationURL[0] != '/') {
                            strncat(currentURL, "/", sizeof(currentURL) - strlen(currentURL) - 1);
                        }
                        // Append the rest
                        strncat(currentURL, locationURL, sizeof(currentURL) - strlen(currentURL) - 1);

                        free(response);
                        response = NULL;
                        redirectCount++;
                        continue;
                    }
                }
            }

            /* Not a redirect we can follow => done. */
            if (response) {
                free(response);
                response = NULL;
            }
            break;
        }

        /* Cleanup allocated param strings. */
        if (originalParams) {
            for (int i = 0; i < originalNumParams; i++) {
                free(originalParams[i]);
            }
            free(originalParams);
        }
        return 0;
    }

    /*
     * Print usage and exit(1).
     * Exactly one newline after usage line, as demanded by the assignment.
     */
    static void printUsageAndExit()
    {
        fprintf(stderr, "Usage: client [-r n < pr1=value1 pr2=value2 …>] <URL>\n");
        exit(1);
    }

    /*
     * Check if string is a positive integer < 65536, used for parsing -r number or port.
     * If invalid => we consider it usage error, so we do not print anything except usage.
     */
    static int isPositiveNumberUnder16Bit(const char *str)
    {
        if (!str || !*str) return 0;

        char *endptr = NULL;
        errno = 0;
        long val = strtol(str, &endptr, 10);
        if (*endptr != '\0' || errno == ERANGE || val <= 0 || val >= 65536) {
            return 0;
        }
        return 1;
    }

    /*
     * parseArguments:
     *   - If parsing fails in any way => usage.
     *   - If successful, fill cmd->url, cmd->numParams, cmd->params.
     */
    static void parseArguments(int argc, char *argv[], CmdArgs *cmd)
    {
        cmd->url       = NULL;
        cmd->numParams = 0;
        cmd->params    = NULL;

        int i = 1;
        int foundR = 0;

        while (i < argc) {
            if (strcmp(argv[i], "-r") == 0) {
                if (foundR) {
                    /* Another -r => usage */
                    printUsageAndExit();
                }
                foundR = 1;
                i++;
                if (i >= argc) {
                    printUsageAndExit();
                }
                if (!isPositiveNumberUnder16Bit(argv[i])) {
                    printUsageAndExit();
                }
                long n = strtol(argv[i], NULL, 10);
                i++;

                cmd->params = (char **)malloc(sizeof(char*) * n);
                if (!cmd->params) {
                    perror("malloc");
                    exit(1);
                }

                for (int j = 0; j < (int)n; j++) {
                    if (i >= argc) {
                        /* Not enough params => usage */
                        printUsageAndExit();
                    }
                    /* Must contain '=' => else usage */
                    if (!strchr(argv[i], '=')) {
                        printUsageAndExit();
                    }
                    cmd->params[j] = strdup(argv[i]);
                    if (!cmd->params[j]) {
                        perror("strdup");
                        exit(1);
                    }
                    i++;
                }
                cmd->numParams = (int)n;

                /* If the next token also looks like name=value, that is extra => usage */
                if (i < argc) {
                    if (strchr(argv[i], '=') && argv[i][0] != '-') {
                        printUsageAndExit();
                    }
                }
            }
            else {
                /* This should be the URL if not yet found, else usage */
                if (!cmd->url) {
                    cmd->url = argv[i];
                    i++;
                } else {
                    /* More tokens after URL => usage */
                    printUsageAndExit();
                }
            }
        }

        /* Must have a URL => else usage */
        if (!cmd->url) {
            printUsageAndExit();
        }
    }

    /*
     * parseURL:
     *   - Must begin with "http://"
     *   - Then optional :<port>
     *   - Then optional /<path>
     *   If invalid => usage.
     */
    static void parseURL(const char *url, char *host, int *port, char *path)
    {
        const char *prefix = "http://";
        size_t prefixLen = strlen(prefix);
        if (strncmp(url, prefix, prefixLen) != 0) {
            /* usage if not starting with http:// */
            printUsageAndExit();
        }

        const char *p = url + prefixLen;

        /* Extract hostname */
        const char *hostStart = p;
        while (*p && *p != ':' && *p != '/') {
            p++;
        }
        int lenHost = (int)(p - hostStart);
        if (lenHost <= 0 || lenHost >= MAX_HOST_LEN) {
            printUsageAndExit();
        }
        strncpy(host, hostStart, (size_t)lenHost);
        host[lenHost] = '\0';

        /* default path = "/" */
        path[0] = '/';
        path[1] = '\0';

        /* If we see ':', parse port => else default 80 */
        if (*p == ':') {
            p++;
            char portBuf[10];
            int idx = 0;
            while (*p && *p != '/' && idx < 9) {
                if (!isdigit((unsigned char)*p)) {
                    printUsageAndExit();
                }
                portBuf[idx++] = *p;
                p++;
            }
            portBuf[idx] = '\0';
            if (!isPositiveNumberUnder16Bit(portBuf)) {
                printUsageAndExit();
            }
            *port = (int)strtol(portBuf, NULL, 10);
        }

        /* If we see '/', parse path. */
        if (*p == '/') {
            if (strlen(p) >= MAX_PATH_LEN) {
                // too large
                printUsageAndExit();
            }
            strncpy(path, p, MAX_PATH_LEN - 1);
            path[MAX_PATH_LEN - 1] = '\0';
        }
    }

    /*
     * buildHTTPRequest:
     *   If error => return -1 => parseUsageAndExit() in caller.
     */
    static int buildHTTPRequest(const char *host,
                                const char *path,
                                int numParams,
                                char **params,
                                char *requestBuffer)
    {
        // Construct finalPath (path + optional query string)
        char finalPath[MAX_PATH_LEN + 100]; // +100 to be safe with added query
        memset(finalPath, 0, sizeof(finalPath));
        strncpy(finalPath, path, sizeof(finalPath) - 1);

        if (numParams > 0) {
            // If there's no '?' yet, add it; otherwise add '&'
            if (!strchr(finalPath, '?')) {
                strncat(finalPath, "?", sizeof(finalPath) - strlen(finalPath) - 1);
            } else {
                strncat(finalPath, "&", sizeof(finalPath) - strlen(finalPath) - 1);
            }
            // Append each param as &name=value
            for (int i = 0; i < numParams; i++) {
                if (i > 0) {
                    strncat(finalPath, "&", sizeof(finalPath) - strlen(finalPath) - 1);
                }
                strncat(finalPath, params[i], sizeof(finalPath) - strlen(finalPath) - 1);
            }
        }

        /*
         * IMPORTANT: Include "Connection: close" in the request headers,
         * so we get the full response (including headers).
         */
        int ret = snprintf(
            requestBuffer,
            REQUEST_BUFFER_SIZE,
            "GET %s HTTP/1.1\r\n"
            "Host: %s\r\n"
            "Connection: close\r\n"
            "\r\n",
            finalPath,
            host
        );

        if (ret < 0 || ret >= REQUEST_BUFFER_SIZE) {
            return -1; // truncated or error
        }
        return 0;
    }

    /*
     * connectToServer: system calls => if fail, herror/perror => returns -1.
     */
    static int connectToServer(const char *hostname, int port)
    {
        struct hostent *server = gethostbyname(hostname);
        if (!server) {
            herror("gethostbyname");
            return -1;
        }

        int sockfd = socket(AF_INET, SOCK_STREAM, 0);
        if (sockfd < 0) {
            perror("socket");
            return -1;
        }

        struct sockaddr_in serv_addr;
        memset(&serv_addr, 0, sizeof(serv_addr));
        serv_addr.sin_family = AF_INET;
        serv_addr.sin_port   = htons(port);

        memcpy(&serv_addr.sin_addr.s_addr, server->h_addr_list[0], (size_t)server->h_length);

        if (connect(sockfd, (struct sockaddr*)&serv_addr, sizeof(serv_addr)) < 0) {
            perror("connect");
            close(sockfd);
            return -1;
        }
        return sockfd;
    }

    /*
     * sendAll: loop until fully sent or error => perror in caller if < 0.
     */
    static int sendAll(int sockfd, const char *buf, size_t len)
    {
        size_t totalSent = 0;
        while (totalSent < len) {
            ssize_t n = send(sockfd, buf + totalSent, len - totalSent, 0);
            if (n < 0) {
                return -1;
            }
            totalSent += (size_t)n;
        }
        return 0;
    }

    /*
     * receiveResponse: read until server closes connection => if fail => perror in caller.
     */
    static int receiveResponse(int sockfd, unsigned char **response, int *responseSize)
    {
        *response = NULL;
        *responseSize = 0;

        size_t capacity = 0;
        size_t size = 0;
        unsigned char *tmp = NULL;

        for (;;) {
            char buffer[MAX_BUFFER_SIZE];
            ssize_t bytesRead = read(sockfd, buffer, sizeof(buffer));
            if (bytesRead < 0) {
                perror("read");
                return -1;
            }
            if (bytesRead == 0) {
                /* no more data */
                break;
            }
            if (size + (size_t)bytesRead >= capacity) {
                size_t newCap = (capacity == 0) ? (size_t)bytesRead + 1 : capacity * 2;
                if (newCap < size + (size_t)bytesRead) {
                    newCap = size + (size_t)bytesRead;
                }
                tmp = realloc(*response, newCap);
                if (!tmp) {
                    perror("realloc");
                    free(*response);
                    *response = NULL;
                    return -1;
                }
                *response = tmp;
                capacity = newCap;
            }
            memcpy((*response) + size, buffer, (size_t)bytesRead);
            size += (size_t)bytesRead;
        }

        if (*response) {
            (*response)[size] = '\0';
        }
        *responseSize = (int)size;

        // We do not want to free(*response); only free(tmp) if we had used it as a
        // separate pointer.  Actually, 'tmp' and '*response' can be the same pointer.
        // If the last realloc assigned to *response, 'tmp' = *response.
        // So we typically do NOT free(tmp) here.  It's safer to just remove this:
        //
        // if (tmp != NULL)
        //     free(tmp);
        //
        // That would incorrectly free the actual data.

        return 0;
    }

    /*
     * extractStatusCode: parse numeric code from "HTTP/x.x ???".
     */
    static int extractStatusCode(const unsigned char *response)
    {
        if (!response || !*response) {
            return -1;
        }
        const char *p = strstr((const char*) response, "HTTP/");
        if (!p) return -1;

        p = strchr(p, ' ');
        if (!p) return -1;
        while (*p == ' ') p++;

        char codeBuf[4] = {0};
        strncpy(codeBuf, p, 3);
        codeBuf[3] = '\0';

        char *endptr = NULL;
        errno = 0;
        long val = strtol(codeBuf, &endptr, 10);
        if (*endptr != '\0' || errno == ERANGE) {
            return -1;
        }
        return (int)val;
    }

    /*
     * extractLocationHeader: look for "location:" ignoring case, copy up to CR/LF.
     */
    static int extractLocationHeader(const unsigned char *response, char *locationURL)
    {
        if (!response) {
            return -1;
        }

        // We will do a case-insensitive search for "Location:" or "location:", etc.
        const char *needle = "Location:";
        size_t needleLen = strlen(needle);

        for (const char *p = (const char*) response; *p; p++) {
            if (strncasecmp(p, needle, needleLen) == 0) {
                p += needleLen;
                while (*p == ' ' || *p == '\t') {
                    p++;
                }
                int i = 0;
                // Copy until CR or LF or end of string, up to our buffer limit.
                while (*p && *p != '\r' && *p != '\n' && i < (MAX_LOCATION_LEN - 1)) {
                    locationURL[i++] = *p++;
                }
                locationURL[i] = '\0';
                return 0;
            }
        }
        return -1;
    }

    /*
     * isHTTP: returns 1 if starts with "http://", else 0.
     */
    static int isHTTP(const char *maybeURL)
    {
        if (!maybeURL) return 0;
        return (strncmp(maybeURL, "http://", 7) == 0) ? 1 : 0;
    }
