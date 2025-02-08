use super::run::*;
use crate::ProcessOutput;
use std::ops::Deref;

pub trait Validator {
    fn validate(
        &self,
        args: &Vec<String>,
        comm_out: Option<Vec<u8>>,
        result: ProcessOutput,
        cwd: &std::path::PathBuf,
    ) -> bool;
}

pub struct ValidatorConfig {
    validator: Box<dyn Validator>,
    timeout: u64,
    log_output: bool,
}

impl ValidatorConfig {
    pub fn new(validator: Box<dyn Validator>, timeout: u64, log_output: bool) -> Self {
        Self {
            validator,
            timeout,
            log_output,
        }
    }
}

impl Deref for ValidatorConfig {
    type Target = dyn Validator;

    fn deref(&self) -> &Self::Target {
        self.validator.as_ref()
    }
}

pub trait Communicator {
    fn communicate(&self, io_timeout: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

pub struct CommunicatorConfig {
    communicator: Box<dyn Communicator>,
    timeout: u64,
    log_output: bool,
}

impl CommunicatorConfig {
    pub fn new(communicator: Box<dyn Communicator>, timeout: u64, log_output: bool) -> Self {
        Self {
            communicator,
            timeout,
            log_output,
        }
    }
}

impl Deref for CommunicatorConfig {
    type Target = dyn Communicator;

    fn deref(&self) -> &Self::Target {
        self.communicator.as_ref()
    }
}

struct Test<'a> {
    name: &'a str,
    args: &'a str,
    memcheck: bool,
    validator: ValidatorConfig,
    communicator: Option<CommunicatorConfig>,
}

impl<'a> Test<'a> {
    fn new(
        name: &'a str,
        args: &'a str,
        memcheck: bool,
        validator: ValidatorConfig,
        communicator: Option<CommunicatorConfig>,
    ) -> Self {
        Self {
            name,
            args,
            memcheck,
            validator,
            communicator,
        }
    }
}

impl<'a> Test<'a> {
    fn run(&self, executable: &str, cwd: &std::path::PathBuf) -> bool {
        println!("[*] Running {} test...", self.name);
        println!("[*] Command: {}", self.args);

        // construct the command to run the exercise
        let cmd = format!("./{} {}", executable, self.args);
        // run the exercise in a shell as a child process
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut test_proc =
            rt.block_on(TestSpawner::new(&cmd, cwd, self.memcheck, Some(self.name)));

        // optionally - send/receive data
        let comm_out: Option<Vec<u8>> = match self.communicator {
            Some(ref comm) => {
                //TODO: when an error occurs, take care of it later on
                let comm_out = comm.communicate(comm.timeout).unwrap();
                if comm.log_output {
                    let log_path = cwd.join(format!("communicate - {}.txt", self.name));
                    std::fs::write(&log_path, &comm_out)
                        .expect(format!("Could not write to file: {:?}", log_path).as_str());
                }
                Some(comm_out)
            }
            None => None,
        };

        // wait for the process to finish
        let result = rt.block_on(test_proc.wait(self.validator.timeout));

        if self.validator.log_output {
            let log_path = cwd.join(format!("output - {}.txt", self.name));
            std::fs::write(&log_path, &result.stdout)
                .expect(format!("Could not write to file: {:?}", log_path).as_str());
        }

        match result.status {
            Ok(ref status) => {
                log_exit_code(status);
            }
            // error could be broken pipe, etc
            Err(ref e) => {
                println!("[-] Failed to run test: {}", e);
            }
        }

        println!();
        self.validator
            .validate(&test_proc.args, comm_out, result, cwd)
    }
}

#[allow(dead_code)]
pub struct TestManager<'a> {
    pub name: String,
    elf: String,
    pub cwd: std::path::PathBuf,
    tests: Vec<Test<'a>>,
}

impl<'a> TestManager<'a> {
    pub fn new(assignment: &str, elf: &str, test_dirname: Option<&str>) -> Self {
        let cwd = match test_dirname {
            Some(dir) => std::env::current_dir().unwrap().join(dir),
            None => std::env::current_dir().unwrap(),
        };

        Self {
            name: String::from(assignment),
            elf: elf.to_string(),
            cwd: cwd,
            tests: Vec::new(),
        }
    }
}

impl<'a> TestManager<'a> {
    pub fn add_test(
        &mut self,
        name: &'a str,
        args: &'a str,
        memcheck: bool,
        validator: ValidatorConfig,
        communicator: Option<CommunicatorConfig>,
    ) {
        let test = Test::new(name, args, memcheck, validator, communicator);
        self.tests.push(test);
    }
}

impl TestManager<'_> {
    pub fn compile_assignment(&self, cmd: &str) -> String {
        println!("[*] Compiling assignment...");
        let res = compile(cmd, &self.cwd);

        if res == "error" {
            println!("[-] Compilation failed");
        } else if res == "warning" {
            println!("[!] Encountered warnings during compilation");
        } else {
            println!("[+] Compilation successful");
        }

        println!();
        res
    }
}

#[allow(dead_code)]
// implemented for the stedents, they can use this to run individual test cases
impl TestManager<'_> {
    pub fn run_test_case(&self, name: &str) -> bool {
        match self.tests.iter().find(|&test| test.name == name) {
            Some(test) => {
                let outcome = test.run(&self.elf, &self.cwd);
                outcome
            }
            None => {
                println!("[!] Test case not found: {}", name);
                false
            }
        }
    }
}

impl TestManager<'_> {
    pub fn run_tests(&self) -> Vec<(&str, bool)> {
        self.tests
            .iter()
            .map(|test| {
                let outcome = test.run(&self.elf, &self.cwd);
                (test.name, outcome)
            })
            .collect()
    }
}

#[allow(dead_code)]
impl TestManager<'_> {
    pub fn get_test_names(&self) -> Vec<&str> {
        self.tests.iter().map(|test| test.name).collect()
    }
}
