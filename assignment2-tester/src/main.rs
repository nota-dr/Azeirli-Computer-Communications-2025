mod tests;
use tests_lib;

fn main() {
    let mut manager = tests_lib::TestManager::new(
        "assignment2",
        "client",
        "assignment2-tester/testee"
    );

    manager.add_test(
        "Usage1",
        "http://httpbin.org -r 10 pr1=1 pr2=2 pr3=3 pr4=4 pr5=5 pr6=6 pr7=7 pr8=8 pr9=9",
        5,
        Box::new(tests::Usage)
    );

    manager.add_test(
        "Usage2",
        "-r 10 pr1=1 pr2=2 pr3=3 pr4=4 5 pr6=6 pr7=7 pr8=8 pr9=9 pr10=10 http://httpbin.org",
        5,
        Box::new(tests::Usage)
    );

    manager.add_test(
        "Validate HTTP Request Structure",
        "http://httpbin.org",
        5,
        Box::new(tests::HttpRequestStructure)
    );

    manager.add_test(
        "Validate HTTP Response Test",
        "-r 1 country=israel http://universities.hipolabs.com/search",
        10,
        Box::new(tests::HttpResponseWithText)
    );

    manager.compile("gcc -Wall assignment2-tester/testee/client.c -o assignment2-tester/testee/client");
    println!("----- Tests Results -----");
    for (name, ok) in manager.run_tests() {
        if ok {
            println!("[+] {}..\x1b[32mok\x1b[0m", name);
        } else {
            println!("[-] {}..\x1b[31mfailed\x1b[0m", name);
        }
    }
}
