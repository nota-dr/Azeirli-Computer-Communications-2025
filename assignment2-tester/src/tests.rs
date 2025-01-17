use tests_lib::Validator;
use reqwest::Version;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONNECTION};

// forward declaration for dynamic dispatch
pub struct Usage;
pub struct HttpRequestStructure;
pub struct HttpRequestStructureWithParams;
pub struct HttpResponseWithText;

impl Validator for Usage {
    fn validate(&self, output: Vec<u8>) -> bool {
        let expected = "Usage: client [-r n < pr1=value1 pr2=value2 …>]";
        let output = String::from_utf8_lossy(&output);
        if output.trim().to_lowercase().contains(&expected.to_lowercase()) {
            return true
        }
        false
    }
}

fn request_structure_verifier(req_parts: Vec<&str>, output: Vec<u8>) -> bool {
    let output = String::from_utf8_lossy(&output).to_lowercase();
    for part in req_parts {
        if !output.contains(&part.to_lowercase()) {
            return false
        }
    }
    true
}

impl Validator for HttpRequestStructure {
    fn validate(&self, output: Vec<u8>) -> bool {
        let expected = vec!["GET / HTTP/1.1\r\n", "Host: httpbin.org\r\n", "Connection: close\r\n"];
        request_structure_verifier(expected, output)
    }
}

impl Validator for HttpRequestStructureWithParams {
    fn validate(&self, output: Vec<u8>) -> bool {
         let expected = vec!["GET /search?country=israel HTTP/1.1\r\n", "Host: universities.hipolabs.com", "Connection: close\r\n"];
        request_structure_verifier(expected, output)
    }
    
}

fn response_verifier(url: &str, output: Vec<u8>) -> bool {
    let client = Client::new();
    let response = client
        .get(url)
        .version(Version::HTTP_11)
        .header("Connection", "close")
        .send()
        .expect("[-] reqwest client could not send request successfully");

    if !response.status().is_success() {
        panic!("[-] reqwest respond failed with status code: {}", response.status());
    }

    let body = response.bytes()
        .expect("[-] Failed to read response body from reqwest client");

    output.windows(body.len()).any(|window| window == body)
}

impl Validator for HttpResponseWithText {
    fn validate(&self, output: Vec<u8>) -> bool {
        response_verifier("http://universities.hipolabs.com/search?country=israel", output)
    }
}
