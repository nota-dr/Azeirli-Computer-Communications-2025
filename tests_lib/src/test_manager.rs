struct TestInfo {
    name: String,
    input: String,
    ok: bool,
}

struct Test {
    into: TestInfo,
    test: Box<dyn Fn() -> bool>,
}

pub struct TestManager {
    elf: String,
    test_dir: String,
    tests: Vec<Test>,
}