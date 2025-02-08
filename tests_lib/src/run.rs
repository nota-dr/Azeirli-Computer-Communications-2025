use std::io::Write;
use std::process::ExitStatus;
use tokio::io::AsyncReadExt;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Status {
    Timeout = 124 << 8,
    Sigint = 130 << 8,
    Sigabrt = 134 << 8,
    Sigkill = 137 << 8,
    Sigsegv = 139 << 8,
    Sigpipe = 141 << 8,
}

pub struct ProcessOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub status: Result<ExitStatus, std::io::Error>,
}

impl ProcessOutput {
    pub fn new(
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        status: Result<ExitStatus, std::io::Error>,
    ) -> Self {
        Self {
            stdout,
            stderr,
            status,
        }
    }
}

pub async fn pipe_reader<R>(mut pipe: R) -> Vec<u8>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut buffer = Vec::new();
    let mut temp_buf = [0u8; 1024];
    while let Ok(n) = pipe.read(&mut temp_buf).await {
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&temp_buf[..n]);
    }
    buffer
}

pub struct TestSpawner {
    child: tokio::process::Child,
    out_task: Option<tokio::task::JoinHandle<Vec<u8>>>,
    err_task: Option<tokio::task::JoinHandle<Vec<u8>>>,
    pub args: Vec<String>,
}

impl TestSpawner {
    pub async fn new(
        cmd: &str,
        cwd: &std::path::PathBuf,
        memcheck: bool,
        test_name: Option<&str>,
    ) -> Self {
        // split the command into a vector of strings
        let args_vec: Vec<String> = cmd.split_whitespace().map(String::from).collect();
        let copied_args_vec = args_vec.clone();

        // construct the path to the executable
        let elf_path = cwd.join(&args_vec[0]);

        // check if the executable exists
        if !elf_path.exists() {
            panic!("[-] Cannot run exercise, {:?} is not found", elf_path);
        }

        let args = if memcheck {
            // Ensure name is provided when memcheck is enabled
            let log_name = match test_name {
                Some(name) => name,
                None => panic!("[-] Name must be provided when memcheck is enabled"),
            };

            // construct valgrind arguments
            let valgrind = vec![
                String::from("valgrind"),
                String::from("--leak-check=full"),
                String::from("--tool=memcheck"),
                String::from("--show-leak-kinds=all"),
                String::from("--track-origins=yes"),
                String::from("--verbose"),
                String::from("--error-exitcode=1"),
                String::from("-v"),
                format!("--log-file=valgrind - {}", log_name),
            ];

            // chain valgrind arguments with the command arguments
            valgrind.into_iter().chain(copied_args_vec).collect()
        } else {
            copied_args_vec
        };

        let mut child = tokio::process::Command::new(&args[0])
            .args(&args[1..])
            .current_dir(cwd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("[!] Failed to start child process");

        let stdout = child.stdout.take().expect("[!] Failed to get stdout");
        let stderr = child.stderr.take().expect("[!] Failed to get stderr");

        // Spawn asynchronous tasks to handle stdout and stderr
        let out_task = tokio::spawn(pipe_reader(stdout));
        let err_task = tokio::spawn(pipe_reader(stderr));

        Self {
            child: child,
            out_task: Some(out_task),
            err_task: Some(err_task),
            args: args_vec,
        }
    }
}

impl TestSpawner {
    pub async fn wait(&mut self, finish_timeout: u64) -> ProcessOutput {
        let secs = tokio::time::Duration::from_secs(finish_timeout);
        let result = tokio::time::timeout(secs, async {
            let status = self.child.wait().await;
            status
        })
        .await;

        let result = match result {
            Ok(status) => status,
            Err(_timeout) => {
                self.child.kill().await.unwrap();
                self.child.wait().await.unwrap();
                // timed out - return corresponding status code
                Ok(ExitStatus::from_raw(Status::Timeout as i32))
            }
        };

        let stdout = self
            .out_task
            .take()
            .unwrap()
            .await
            .expect("[-] Failed to read stdout");

        let stderr = self
            .err_task
            .take()
            .unwrap()
            .await
            .expect("[-] Failed to read stderr");

        ProcessOutput::new(stdout, stderr, result)
    }
}

pub fn compile(input: &str, cwd: &std::path::PathBuf) -> String {
    let args: Vec<&str> = input.split_whitespace().collect();
    if args.len() < 5 {
        panic!("[!] Invalid gcc input: {}", input);
    }

    let elf_path = cwd.join(args.last().as_ref().unwrap());
    if elf_path.exists() {
        std::fs::remove_file(elf_path).expect("[-] Failed to remove existing executable");
    }

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(input)
        .current_dir(cwd)
        .output()
        .expect("[-] Failed to run compilation command");

    let log_path = cwd.join("compilation_output.txt");
    let mut logfile = std::fs::File::create(&log_path)
        .expect(format!("[-] Failed to create compilation log file: {:?}", &log_path).as_str());

    logfile
        .write_all(&output.stdout)
        .expect("[-] Failed to write to compilation log file");

    logfile
        .write_all(&output.stderr)
        .expect("[-] Failed to write to compilation log file");

    let needle = b"error";
    if let Some(_) = output
        .stderr
        .windows(needle.len())
        .position(|window| window == needle)
    {
        return String::from("error");
    }

    let needle = b"warning";
    if let Some(_) = output
        .stderr
        .windows(needle.len())
        .position(|window| window == needle)
    {
        return String::from("warning");
    }

    String::from("success")
}

pub fn log_exit_code(exit_code: &ExitStatus) -> bool {
    if exit_code.success() {
        return true;
    }

    let mut success = true;
    match &exit_code.code() {
        Some(code) => {
            if *code == Status::Timeout as i32 {
                println!("[-] Test timed out");
                success = false;
            } else if *code == Status::Sigsegv as i32 {
                println!("[-] Test crashed with SIGSEGV");
                success = false;
            } else if *code == Status::Sigabrt as i32 {
                println!("[-] Test crashed with SIGABRT");
                success = false;
            } else {
                println!("[!] Test exited with status code: {}", code);
            }
        }
        None => {
            println!("[-] Test exited with unknown status code");
            success = false;
        }
    }
    success
}

pub fn check_valgrind_leaks(log_path: &std::path::PathBuf) -> bool {
    let log_contents = match std::fs::read_to_string(log_path) {
        Ok(contents) => contents,
        Err(_) => {
            panic!("[-] Failed to read valgrind log file");
        }
    };

    let needle = "ERROR SUMMARY: 0 errors from 0 contexts";
    if log_contents.contains(needle) {
        true
    } else {
        false
    }
}
