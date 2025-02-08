use assignment2_tester::hw2_tests;
use batch_runner::*;
use std::io::{BufWriter, Write};
use tests_lib;
use xxhash_rust::xxh32::Xxh32;

const SUBMISSIONS_DIR: &str = "submissions";
const FAILED_DIR: &str = "failed";
const RESULTS_DIR: &str = "results_04_1";
const CSV_NAME: &str = "ex1_04_1.csv";


#[allow(dead_code)]
fn main() {
    let mut tm = tests_lib::TestManager::new("assignment2", "client", Some("testee"));

    tm.add_test(
        "Usage1",
        "http://httpbin.org -r 10 pr1=1 pr2=2 pr3=3 pr4=4 pr5=5 pr6=6 pr7=7 pr8=8 pr9=9",
        false,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::Usage), 5, true),
        None,
    );

    tm.add_test(
        "Usage2",
        "-r 10 pr1=1 pr2=2 pr3=3 pr4=4 5 pr6=6 pr7=7 pr8=8 pr9=9 pr10=10 http://httpbin.org",
        false,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::Usage), 5, true),
        None,
    );

    tm.add_test(
        "HTTP Request Structure",
        "http://httpbin.org",
        false,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::HttpRequestStructure), 20, true),
        None,
    );

    tm.add_test(
        "HTTP Response Contains Text",
        "-r 1 country=israel http://universities.hipolabs.com/search",
        false,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::ResponseContainsText), 20, true),
        None,
    );

    tm.add_test(
        "HTTP Response Contains Image",
        "http://localhost:8080/resources/meow.png",
        false,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::ResponseContainsImage), 30, true),
        None,
    );

    tm.add_test(
        "Relative Redirect N Times",
        "http://localhost:8080/recursive/3",
        true,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::RelativeRedirectTimes), 20, true),
        None,
    );

    tm.add_test(
        "Absolute Redirect",
        "http://localhost:8080/absolute",
        true,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::AbsoluteRedirect), 20, true),
        None,
    );

    tm.add_test(
        "Life Cycle",
        "http://localhost:8080/resources?file=Approval.gif",
        true,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::LifeCycle), 30, true),
        None,
    );

    tm.add_test(
        "Valgrind",
        "http://localhost:8080/",
        false,
        tests_lib::ValidatorConfig::new(Box::new(hw2_tests::Valgrind), 20, false),
        None,
    );

    let cwd = std::env::current_dir().unwrap();
    let assignment_workspace = cwd.join("assignments").join(&tm.name);
    let subs_dir = assignment_workspace.join(SUBMISSIONS_DIR);
    let fails_dir = assignment_workspace.join(FAILED_DIR);
    let results_dir = assignment_workspace.join(RESULTS_DIR);

    if !fails_dir.exists() {
        std::fs::create_dir(&fails_dir).expect("Could not create fails directory");
    }

    if !results_dir.exists() {
        std::fs::create_dir(&results_dir).expect("Could not create results directory");
    }

    let csv_file = std::fs::File::create(assignment_workspace.join(CSV_NAME))
        .expect("Could not create CSV file");

    let mut csv_buf = BufWriter::new(csv_file);
    csv_buf
        .write_all("\u{FEFF}".as_bytes())
        .expect("Could not write BOM to CSV file");

    let mut wtr = csv::Writer::from_writer(csv_buf);
    let csv_prefix = vec!["Hash", "Name", "Compilation"];
    let csv_suffix = vec!["Submitted late", "File Type", "Comment", "Bonus", "Grade"];
    let test_names = tm.get_test_names();
    let csv_headers = csv_prefix
        .iter()
        .chain(test_names.iter())
        .chain(csv_suffix.iter())
        .collect::<Vec<_>>();

    wtr.write_record(csv_headers)
        .expect("Could not write CSV headers");

    let mut hasher = Xxh32::new(0);
    let exercies = std::fs::read_dir(subs_dir).unwrap();
    for ex in exercies {
        let ex_path = ex.unwrap().path();
        let ex_name = ex_path.file_name().unwrap().to_str().unwrap();
        let student_name = ex_name.split('_').next().unwrap();

        hasher.update(student_name.as_bytes());
        let hash = format!("{:x}", hasher.digest());

        let mime = get_mime(&ex_path).expect("Could not determine MIME type");

        let compression_type = get_compression_type(&ex_path);

        if compression_type == CompressAlgo::Invalid {
            std::fs::rename(&ex_path, fails_dir.join(ex_name))
                .expect(format!("Could not move {:?} to fails directory", ex_path).as_str());
        } else if compression_type == CompressAlgo::Zip {
            if let Err(_) = unzip(&ex_path, &tm.cwd) {
                std::fs::rename(&ex_path, fails_dir.join(ex_name))
                    .expect(format!("Could not move {:?} to fails directory", ex_path).as_str());
            }
        } else {
            if let Err(_) = untar(&ex_path, &tm.cwd, compression_type) {
                std::fs::rename(&ex_path, fails_dir.join(ex_name))
                    .expect(format!("Could not move {:?} to fails directory", ex_path).as_str());
            }
        }

        // compile student's code and run tests
        let compilation = tm.compile_assignment("gcc -Wall *.c -o client");
        let tests_results = if compilation == "error" {
            let res: Vec<&str> = std::iter::repeat("failed").take(test_names.len()).collect();
            res
        } else {
            // if compilation succeeded, run tests
            let tests_results = tm.run_tests();
            let res: Vec<&str> = tests_results
                .iter()
                .map(|r| if r.1 { "passed" } else { "failed" })
                .collect();
            res
        };

        let mut record = vec![&hash, student_name, &compilation];
        record.extend(tests_results);
        record.extend(&["", &mime, "", "", ""]);
        wtr.write_record(record)
            .expect("Could not write CSV record");

        // create student's result directory
        let student_dir = results_dir.join(hash);
        std::fs::create_dir(&student_dir)
            .expect(format!("Could not create directory: {:?}", student_dir).as_str());

        // move all student's files to his result directory
        let files = std::fs::read_dir(&tm.cwd)
            .expect(format!("Could not read directory: {:?}", tm.cwd).as_str());

        for file in files {
            let file_path = file.unwrap().path();
            let file_name = file_path.file_name().unwrap().to_str().unwrap();
            std::fs::rename(&file_path, student_dir.join(file_name))
                .expect(format!("Could not move {:?} to {:?}", file_path, student_dir).as_str());
        }
    }
}
