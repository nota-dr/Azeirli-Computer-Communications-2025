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
        .manage(TokenManager {
            token: Mutex::new(String::new()),
        })
        .mount(
            "/",
            routes![
                home::index,
                home::meow,
                home::recursive_redirect,
                home::absolute_redirect,
                home::secret
            ],
        )
}
