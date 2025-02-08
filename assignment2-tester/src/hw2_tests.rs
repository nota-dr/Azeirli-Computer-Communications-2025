use reqwest::blocking::Client;
use reqwest::{header, Version};
use tests_lib::{check_valgrind_leaks, ProcessOutput, Validator};

// forward declaration for dynamic dispatch
pub struct Usage;
pub struct HttpRequestStructure;
pub struct HttpRequestStructureWithParams;
pub struct ResponseContainsText;
pub struct ResponseContainsImage;
pub struct RelativeRedirectTimes;
pub struct AbsoluteRedirect;
pub struct LifeCycle;
pub struct Valgrind;

fn send_http_request(url: &str) -> reqwest::blocking::Response {
    let client = Client::new();
    client
        .get(url)
        .version(Version::HTTP_11)
        .header(header::CONNECTION, "close")
        .send()
        .expect("[-] reqwest client could not send request successfully")
}

impl Validator for Usage {
    fn validate(
        &self,
        _: &Vec<String>,
        _: Option<Vec<u8>>,
        result: ProcessOutput,
        _: &std::path::PathBuf,
    ) -> bool {
        let expected = "Usage: client";
        let output = [result.stdout, result.stderr].concat();
        let output = String::from_utf8_lossy(&output);
        if output
            .trim()
            .to_lowercase()
            .contains(&expected.to_lowercase())
        {
            return true;
        }
        false
    }
}

fn request_structure_verifier(req_parts: Vec<&str>, output: ProcessOutput) -> bool {
    let output = [output.stdout, output.stderr].concat();
    let output = String::from_utf8_lossy(&output).to_lowercase();
    for part in req_parts {
        if !output.contains(&part.to_lowercase()) {
            return false;
        }
    }
    true
}

impl Validator for HttpRequestStructure {
    fn validate(
        &self,
        _: &Vec<String>,
        _: Option<Vec<u8>>,
        result: ProcessOutput,
        _: &std::path::PathBuf,
    ) -> bool {
        let expected = vec![
            "GET / HTTP/1.1\r\n",
            "Host: httpbin.org\r\n",
            "Connection: close\r\n",
        ];

        request_structure_verifier(expected, result)
    }
}

impl Validator for HttpRequestStructureWithParams {
    fn validate(
        &self,
        _: &Vec<String>,
        _: Option<Vec<u8>>,
        result: ProcessOutput,
        _: &std::path::PathBuf,
    ) -> bool {
        let expected = vec![
            "GET /search?country=israel HTTP/1.1\r\n",
            "Host: universities.hipolabs.com",
            "Connection: close\r\n",
        ];

        request_structure_verifier(expected, result)
    }
}

fn response_verifier(url: &str, output: ProcessOutput) -> bool {
    let output = [output.stdout, output.stderr].concat();
    let response = send_http_request(url);

    if !response.status().is_success() {
        panic!(
            "[-] reqwest respond failed with status code: {}",
            response.status()
        );
    }

    let body = response
        .bytes()
        .expect("[-] Failed to read response body from reqwest client");

    output.windows(body.len()).any(|window| window == body)
}

impl Validator for ResponseContainsText {
    fn validate(
        &self,
        _: &Vec<String>,
        _: Option<Vec<u8>>,
        result: ProcessOutput,
        _: &std::path::PathBuf,
    ) -> bool {
        response_verifier(
            "http://universities.hipolabs.com/search?country=israel",
            result,
        )
    }
}

impl Validator for ResponseContainsImage {
    fn validate(
        &self,
        _: &Vec<String>,
        _: Option<Vec<u8>>,
        result: ProcessOutput,
        _: &std::path::PathBuf,
    ) -> bool {
        response_verifier("http://localhost:8080/resources/meow.png", result)
    }
}

impl Validator for RelativeRedirectTimes {
    fn validate(
        &self,
        args: &Vec<String>,
        _: Option<Vec<u8>>,
        result: ProcessOutput,
        _: &std::path::PathBuf,
    ) -> bool {

        let response_headers_n_times: Vec<&str> = vec![
            "http/1.1 308 permanent redirect",
            "content-length: 0",
        ];

        let times = args
            .last()
            .unwrap()
            .chars()
            .last()
            .unwrap()
            .to_digit(10)
            .unwrap() as usize;

        let output = [result.stdout, result.stderr].concat();
        let output = String::from_utf8_lossy(&output).to_lowercase();
        for header in response_headers_n_times {
            if output.matches(header).count() != times {
                return false;
            }
        }
        output.contains("200 ok")
    }
}

impl Validator for AbsoluteRedirect {
    fn validate(
        &self,
        _: &Vec<String>,
        _: Option<Vec<u8>>,
        result: ProcessOutput,
        _: &std::path::PathBuf,
    ) -> bool {
        let response_headers: Vec<&str> = vec![
            "http/1.1 308 permanent redirect",
            "location: http://www.pdf995.com/why.html",
            "server: netcom-ex2",
            "cache-control: no-store",
            "date:",
            "connection: close",
            "content-length: 0",
        ];

        let output = [result.stdout, result.stderr].concat();
        let output = String::from_utf8_lossy(&output).to_lowercase();
        for header in response_headers {
            if !output.contains(header) {
                return false;
            }
        }
        output.contains("200 ok")
    }
}

impl Validator for LifeCycle {
    fn validate(
        &self,
        _: &Vec<String>,
        _: Option<Vec<u8>>,
        result: ProcessOutput,
        _: &std::path::PathBuf,
    ) -> bool {
        let method = b"GET /";
        let start_of_req_headers = result
            .stdout
            .windows(method.len())
            .position(|window| window == method);
        let end_of_req_headers = result
            .stdout
            .windows(4)
            .position(|window| window == b"\r\n\r\n");

        let start_of_req_headers = match start_of_req_headers {
            Some(index) => index,
            None => return false,
        };

        let end_of_req_headers = match end_of_req_headers {
            Some(index) => index,
            None => return false,
        } + 4;

        let request_len = end_of_req_headers as i32 - start_of_req_headers as i32;
        if request_len < 0 {
            return false;
        }

        let request_printf = format!("LEN = {}", request_len as u64).into_bytes();

        if let None = result
            .stdout
            .windows(request_printf.len())
            .position(|window| window == request_printf)
        {
            return false;
        }

        let status = b"HTTP/1.1 200 OK";
        let start_of_response = result
            .stdout
            .windows(status.len())
            .position(|window| window == status);

        let total_printf = b"Total received response bytes";
        let end_of_response = result
            .stdout
            .windows(total_printf.len())
            .position(|window| window == total_printf);

        let start_of_response = match start_of_response {
            Some(index) => index,
            None => return false,
        };

        let end_of_response = match end_of_response {
            Some(index) => index,
            None => return false,
        };

        let start_of_printf = result.stdout[..end_of_response]
            .iter()
            .rposition(|&x| x == b'\n')
            .unwrap();

        let end_of_response = start_of_printf;
        let total_num_of_bytes: i32 = end_of_response as i32 - start_of_response as i32;
        if total_num_of_bytes < 0 {
            return false;
        }
        // allow students to be off by +-1 byte
        let summarize_printf_v1 = format!("bytes: {}", total_num_of_bytes as u64).into_bytes();
        let summarize_printf_v2 =
            format!("bytes: {}", (total_num_of_bytes - 1) as u64).into_bytes();
        let summarize_printf_v3 =
            format!("bytes: {}", (total_num_of_bytes + 1) as u64).into_bytes();

        // check there is something in the response's body
        if total_num_of_bytes < 3000 {
            return false;
        }

        let summerize_printf_pos_v1 = result
            .stdout
            .windows(summarize_printf_v1.len())
            .position(|window| window == summarize_printf_v1);
        let summerize_printf_pos_v2 = result
            .stdout
            .windows(summarize_printf_v2.len())
            .position(|window| window == summarize_printf_v2);
        let summerize_printf_pos_v3 = result
            .stdout
            .windows(summarize_printf_v3.len())
            .position(|window| window == summarize_printf_v3);

        if !(summerize_printf_pos_v1.is_some()
            || summerize_printf_pos_v2.is_some()
            || summerize_printf_pos_v3.is_some())
        {
            return false;
        }
        true
    }
}

impl Validator for Valgrind {
    fn validate(
        &self,
        _: &Vec<String>,
        _: Option<Vec<u8>>,
        _: ProcessOutput,
        cwd: &std::path::PathBuf,
    ) -> bool {
        let life_cycle_metadata = cwd.join("output - Life Cycle.txt");
        let life_cycle_valgrind = cwd.join("valgrind - Life Cycle");
        let absolute_redirect_valgrind = cwd.join("valgrind - Absolute Redirect");
        let relative_redirect_valgrind = cwd.join("valgrind - Relative Redirect N Times");

        let life_cycle_metadata = std::fs::metadata(&life_cycle_metadata)
            .expect("[-] Failed to read metadata for Life Cycle");

        if !check_valgrind_leaks(&life_cycle_valgrind) {
            return false;
        }

        if !check_valgrind_leaks(&absolute_redirect_valgrind) {
            return false;
        }

        if !check_valgrind_leaks(&relative_redirect_valgrind) {
            return false;
        }

        // check there is something in the response's body
        if life_cycle_metadata.len() < 20000 {
            return false;
        }

        true
    }
}
