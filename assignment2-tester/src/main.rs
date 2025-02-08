mod hw2_tests;
use libc::{prctl, PR_SET_PDEATHSIG, SIGHUP};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;
use tests_lib;

fn main() {
    let mut server = unsafe {
        Command::new("cargo")
            .arg("run")
            .arg("-p")
            .arg("assignment2-tester")
            .arg("--bin")
            .arg("axum_server")
            .stdout(Stdio::null())
            .pre_exec(|| {
                prctl(PR_SET_PDEATHSIG, SIGHUP);
                Ok(())
            })
            .spawn()
            .expect("Failed to start server")
    };

    sleep(Duration::from_secs(10));

    match std::process::Command::new("valgrind")
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => {
            println!("[+] Valgrind is installed.");
            println!("[+] Version: {}", String::from_utf8_lossy(&output.stdout));
            println!()
        }
        Ok(_) => {
            println!("[+] Valgrind command found, but it failed to run properly.");
            std::process::exit(0);
        }
        Err(_) => {
            println!("[-] Valgrind is not installed or not in PATH Please install it.");
            std::process::exit(0);
        }
    }

    let mut te = tests_lib::TestManager::new("assignment2", "client", Some("testee"));

    te.add_test(
        "Usage1",
        "http://httpbin.org -r 10 pr1=1 pr2=2 pr3=3 pr4=4 pr5=5 pr6=6 pr7=7 pr8=8 pr9=9",
        false,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::Usage), 5, true),
        None,
    );

    te.add_test(
        "Usage2",
        "-r 10 pr1=1 pr2=2 pr3=3 pr4=4 5 pr6=6 pr7=7 pr8=8 pr9=9 pr10=10 http://httpbin.org",
        false,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::Usage), 5, true),
        None,
    );

    te.add_test(
        "HTTP Request Structure",
        "http://httpbin.org",
        false,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::HttpRequestStructure), 20, true),
        None,
    );

    te.add_test(
        "HTTP Response Contains Text",
        "-r 1 country=israel http://universities.hipolabs.com/search",
        false,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::ResponseContainsText), 20, true),
        None,
    );

    te.add_test(
        "HTTP Response Contains Image",
        "http://localhost:8080/resources/meow.png",
        false,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::ResponseContainsImage), 30, true),
        None,
    );

    te.add_test(
        "Relative Redirect N Times",
        "http://localhost:8080/recursive/3",
        true,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::RelativeRedirectTimes), 20, true),
        None,
    );

    te.add_test(
        "Absolute Redirect",
        "http://localhost:8080/absolute",
        true,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::AbsoluteRedirect), 20, true),
        None,
    );

    te.add_test(
        "Life Cycle",
        "http://localhost:8080/resources?file=Approval.gif",
        true,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::LifeCycle), 30, true),
        None,
    );

    te.add_test(
        "Valgrind",
        "http://localhost:8080/",
        false,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::Valgrind), 20, false),
        None,
    );

    let compilation = te.compile_assignment("gcc -Wall *.c -o client");
    if compilation != "error" {
        println!("----- Tests Results -----");
        for (name, ok) in te.run_tests() {
            if ok {
                println!("[+] {}... \x1b[32mok\x1b[0m", name);
            } else {
                println!("[-] {}... \x1b[31mfailed\x1b[0m", name);
            }
        }
    } else {
        println!("Failed to compile assignment");
    }

    server.kill().unwrap();
}
