use std::time;
use std::collections::HashMap;

use super::proc;

struct TestInfo<'a> {
    name: &'a str,
    args: &'a str,
    timeout: time::Duration,
}

impl<'a> TestInfo<'a> {
    fn new(name: &'a str, args: &'a str, timeout: u64) -> Self {
        Self {
            name,
            args,
            timeout: time::Duration::from_secs(timeout),
        }
    }
}

pub trait Validator {
    fn validate(&self, output: Vec<u8>) -> bool;
}

struct Test<'a> {
    info: TestInfo<'a>,
    test: Box<dyn Validator>,
}

impl<'a> Test<'a> {
    fn new(info: TestInfo<'a>, test: Box<dyn Validator>) -> Self {
        Self {info, test}
    }
}

impl<'a> Test<'a> {
    fn run(&self, execution_path: &str, log_dir: Option<&str>) -> bool {
        // might change this and implement it in specific test library
        println!("[*] Running {} test...", self.info.name);
        println!("[*] Command line arguments: {}", self.info.args);
        println!();
            
        let output = proc::run_assignment(
            execution_path,
            self.info.args,
            self.info.timeout,
            self.info.name,
            log_dir
            );
        
        self.test.validate(output)
    }
}

pub struct TestManager<'a> {
    assignment_name: String,
    elf: String,
    test_dir: String,
    execution_path: String,
    tests: HashMap<&'a str, Test<'a>>,
}

impl<'a> TestManager<'a> {
    pub fn new(assignment_name: &str, elf: &str, test_dir: &str) -> Self {
        Self {
            assignment_name: assignment_name.to_string(),
            elf: elf.to_string(),
            test_dir: test_dir.to_string(),
            execution_path: format!("./{test_dir}/{elf}"),
            tests: HashMap::new()
        }
    }
}

impl<'a> TestManager<'a> {
    pub fn add_test(&mut self, name: &'a str, args: &'a str, timeout: u64, validator: Box<dyn Validator>) {
        let info = TestInfo::new(name, args, timeout);
        let test = Test::new(info, validator);
        self.tests.insert(name, test);
    }
}

impl TestManager<'_> {
    pub fn compile<'a> (&self, input: &'a str) -> &'a str {
        println!("[*] Compiling assignment...");
        let res = proc::compile(input, &*self.test_dir);
        if res == "error" {
            println!("[-] Compilation failed");
        } else if res == "warning" {
            println!("[!] Encountered warnings during compilation");
        } else {
            println!("[+] Compilation successful");
        }
        res
    }
}

impl TestManager<'_> {
    pub fn run_test_case(&self, name: &str) -> bool {
        match self.tests.get(name) {
            Some(test) => test.run(&self.execution_path, Some(&self.test_dir)),
            None => {
                println!("[!] Test case not found: {}", name);
                false
            }
        }
    }
}

impl TestManager<'_> {
    pub fn run_tests(&self) -> HashMap<String, bool>{
        self.tests.iter().map(|(name, _)| {
            (name.to_string(), self.run_test_case(name))
        }).collect()
    }
}