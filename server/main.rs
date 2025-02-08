#[macro_use]
extern crate rocket;

mod routes;
use std::sync::Mutex;

use routes::*;

struct TokenManager {
    token: Mutex<String>,
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(fairings::HeaderCapitalizer)
        .configure(rocket::Config {
            port: 8080,
            address: "127.0.0.1".parse().unwrap(),
            ..Default::default()
        })
        .manage(TokenManager {
            token: Mutex::new(String::new()),
        })
        .mount(
            "/",
            routes![
                home::index,
                home::get_resource_with_path,
                home::get_resource_with_param,
                home::recursive_redirect,
                home::absolute_redirect,
                home::secret,
            ],
        )
}
