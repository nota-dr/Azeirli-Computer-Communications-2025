use rocket::fairing::{Fairing, Info, Kind};
use rocket::Request;
use rocket::Response;
use rocket::http::Header;


pub struct HeaderCapitalizer;

#[rocket::async_trait]
impl Fairing for HeaderCapitalizer {
    fn info(&self) -> Info {
        Info {
            name: "Header Capitalizer",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        let new_headers: Vec<(String, String)> = response.headers()
            .iter()
            .map(|h| {
                let header = h.name().to_string();
                let mut new_header = header.chars();
                let name = new_header.next().unwrap().to_uppercase().chain(new_header).collect();
                let value = h.value().to_string();
                // println!("header: -> {}: {}", name, value);
                (name, value)
            })
            .collect();

        let headers: Vec<String> = response.headers()
            .iter()
            .map(|h| h.name().to_string())
            .collect();
        
        // remove original headers
        for header in headers {
            response.remove_header(&header);
        }

        for (name, value) in new_headers {
            response.set_header(Header::new(name, value));
        }

        for header in response.headers().iter() {
            println!("header: -> {}: {}", header.name(), header.value());
        }

        response.set_header(Header::new("Test", "Hello"));

    }
}