use std::process::{Command, ExitStatus, Stdio};
use std::io::{Write, Read};
use std::fs::{self, File};
use std::thread;
use std::time;
use std::env;

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
    pub status: ExitStatus,
}

pub fn compile<'a>(input: &'a str, log_dir: &str) -> &'a str {
    let mut args = input.split_whitespace();
    let cmd = args.next().expect("[!] No command found");
    let args: Vec<&str> = args.collect();
    let last_arg = *args.last()
        .expect("[!] No args found for gcc command");

    let is_exists = fs::exists(last_arg)
    .expect("[!] Error checking for executable!");

    if is_exists {
        if let Err(e) = fs::remove_file(last_arg) {
            panic!("[!] Error removing executable: {:?}", e);
        }
    }
    
    let output = Command::new(cmd)
    .args(args)
    .output()
    .expect("[!] Errored while compiling");

    // let stdout = String::from_utf8_lossy(&output.stdout);
    // let stderr = String::from_utf8_lossy(&output.stderr);
    // let both = stdout + stderr;

    let log_path = env::current_dir().unwrap()
    .join(log_dir)
    .join("compilation_output.txt");

    let mut logfile = File::create(&log_path)
    .expect("[!] Error creating compilation_output.txt");

    logfile.write_all(&output.stdout).expect("[!] Error writing to compilation_output.txt");
    logfile.write_all(&output.stderr).expect("[!] Error writing to compilation_output.txt");

    let needle = b"error";
    if let Some(_) = output.stderr.windows(needle.len()).position(|window| window == needle) {
        return "error";
    }

    let needle = b"warning";
    if let Some(_) = output.stderr.windows(needle.len()).position(|window| window == needle) {
        return "warning";
    }

    "success"
}

pub fn exec_and_wait(cmd: &str, timeout: time::Duration) -> ProcessOutput {
    let mut args = cmd.split_whitespace();
    let cmd = args.next().expect("[!] No command provided");
    let args = args.collect::<Vec<&str>>();

    let start_time = std::time::Instant::now();
    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("[!] Failed to execute command");

    loop {
        match child.try_wait() {
            // process still running
            Ok(None) => {
                if start_time.elapsed() > timeout {
                    child.kill().expect("[!] Failed to kill process");
                    return ProcessOutput {
                        stdout: Vec::new(),
                        stderr: String::from("Timeouit occurred").as_bytes().to_vec(),
                        // the upper 8 bits are the actual exit code
                        status: ExitStatus::from_raw(Status::Timeout as i32),
                    };
                }
                // sleep for a bit if timeout hasn't been reached
                thread::sleep(time::Duration::from_millis(250));
            }
            // process has finished
            Ok(Some(status)) => {
                let mut stdout_vec: Vec<u8> = Vec::new();
                let mut stderr_vec: Vec<u8> = Vec::new();
                let mut stdout = child.stdout.take().expect("[!] Failed to get stdout");
                let mut stderr = child.stderr.take().expect("[!] Failed to get stderr");
                stdout
                    .read_to_end(&mut stdout_vec)
                    .expect("[!] Failed to read stdout");
                stderr
                    .read_to_end(&mut stderr_vec)
                    .expect("[!] Failed to read stderr");
                return ProcessOutput {
                    stdout: stdout_vec,
                    stderr: stderr_vec,
                    status: status,
                };
            }
            // error occurred
            Err(e) => {
                panic!("[!] Error while execute bash command: {:?}", e);
            }
        }
    }
}

pub fn run_assignment(execution_path: &str, args: &str, timeout: time::Duration, name: &str, log_dir: Option<&str>) -> Vec<u8>{
    let is_execution_path_exists = fs::exists(execution_path)
    .expect("[!] Error checking for executable!");

    if !is_execution_path_exists {
        panic!("[!] File does not exist: {}", execution_path);
    }

    let cmd = format!("{} {}", execution_path, args);
    let output = exec_and_wait(&*cmd, timeout);

    let status = output.status.code().unwrap();
    if status == Status::Timeout as i32 {
        println!("[!] Timeout occurred");
    }

    if status == Status::Sigabrt as i32 {
        println!("[!] SIGABRT occurred");
    }

    if status == Status::Sigsegv as i32 {
        println!("[!] Segmentation Fault occurred");
    }

    if let Some(dir) = log_dir {
        let log_path = env::current_dir().unwrap()
        .join(dir)
        .join(format!("output - {}.txt", name));
        
        let mut f = File::create(&log_path)
        .expect("Unable to create file");

        f.write_all(&output.stdout).expect("Unable to write data");
        f.write_all(&output.stderr).expect("Unable to write data");
    }
    
    output.stdout.into_iter().chain(output.stderr.into_iter()).collect::<Vec<u8>>()
}