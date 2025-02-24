# Azeirli-Computer-Communications-2025
A collection of automated testers and grading scripts for four exercises in computer communications. This repository includes test cases, sample inputs/outputs, and scripts for updating grades and generating reports.

## Features:
* Automated testers for Exercises 1-4
* Scripts for grading and report generation
* Sample input/output files for validation
* Organized structure for easy use and maintenance

## Usage
- Install Git using the commands:
    - `sudo apt update`
    - `sudo apt install git`
- Verify Git installation with:
    - `git --version`
- Install the Rust programing language using the commands:
    - `sudo apt install build-essential`
    - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Clone the repository:
    - `git clone --branch assignment2 --recurse-submodules https://github.com/nota-dr/Azeirli-Computer-Communications-2025.git`
- Navigate to the repository directory::
    - `cd Azeirli-Computer-Communications-2025`
- Place your assignment files inside **testee** folder.
- Run the tester:
    - `cargo run -p assignment2-tester --bin assignment2-tester`

## General Guidelines
- **Log Files:** Each test generates a log file containing your exercise's output for the corresponding test.
- **Test Inputs:** Inputs for each test are located in the `lib.rs` file, within the `template_args` function..
- **Local Server:** Some requests are sent to `localhost`, indicating a local server is running.
- **Testing Against the Local Server:** To test your exercise against the local server without running the tester:
    - `cargo run -p assignment2-tester --bin axum_server`