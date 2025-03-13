# Azeirli-Computer-Communications-2025
A collection of automated testers and grading scripts for four exercises in computer communications. This repository includes test cases, sample inputs/outputs, and scripts for updating grades and generating reports.

## Features:
* Automated testers for Exercises 1-4
* Scripts for grading and report generation
* Sample input/output files for validation
* Organized structure for easy use and maintenance

## Usage: Running Precompiled Binaries (Recommended)
1. **Open terminal**

2. **Clone the Repository**
- `git clone --branch assignment3 --recurse-submodules https://github.com/nota-dr/Azeirli-Computer-Communications-2025.git`
3. **Navigate to the repository directory**
- `cd Azeirli-Computer-Communications-2025`
5. **Place your assignment files**
- Place your assignment files inside **testee** folder.
6. **Run the tester**
- `./target/release/assignment2-tester`

## Installation and Compilation from Scratch 
- Only needed if you don't want to run the precompiled binaries.
- Use this if you prefer to build the tester manually.
1. **Install Required Dependencies**
- Install Git using the commands:
    - `sudo apt update`
    - `sudo apt install git`
- Verify Git installation with:
    - `git --version`
- Install the Rust programing language using the commands:
    - `sudo apt install build-essential`
    - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. **Clone the Repository**
    - `git clone --branch assignment3 --recurse-submodules https://github.com/nota-dr/Azeirli-Computer-Communications-2025.git`
3. **Navigate to the repository directory**
- Navigate to the repository directory:
    - `cd Azeirli-Computer-Communications-2025`
4. **Place your assignment files**
- Place your assignment files inside **testee** folder.
5. **Run the tester**
- `cargo run -p assignment3-tester`


## General Guidelines
- **Test Inputs:** 
    - Inputs for each test are printed to the screen.
- **Log Files:**
    - Each test generates a log file containing the output of your exercise.
    - Log files come in three types:
    1. **Output Logs:**
        - Log files that start with the word **output** contain the standard output of your exercise.
    2. **Communication Logs:**
        - Log files that start with the word **communicate** contain the responses from requests sent to your server.
    3. **Valgrind Logs:**
        - Log files that start with the word **valgrind** contain Valgrind results for the corresponding test.
    - **Debugging Failed Tests:**
        - If you fail any test, check the log files for details.
        - You can see exactly what was tested by reviewing the source code in **hw3_tests.rs**.
        - Each test includes a function called validate, which verifies your responses.