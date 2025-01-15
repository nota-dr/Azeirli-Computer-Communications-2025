extern crate rocket;

use super::guards::RedirectGuard;

use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::content::RawHtml;
use rocket::response::Redirect;
use rocket::State;
use std::path::Path;
use uuid::Uuid;

use crate::TokenManager;

#[get("/")]
pub async fn index() -> &'static str {
    "Welcome to my test server!"
}

#[get("/meow.png")]
pub async fn meow() -> Option<NamedFile> {
    let project_root = env!("CARGO_MANIFEST_DIR");
    NamedFile::open(Path::new(project_root).join("resources/meow.png"))
        .await
        .ok()
}

#[get("/recursive-redirect/<n>")]
pub async fn recursive_redirect(n: u32, state: &State<TokenManager>) -> Result<Redirect, Status> {
    if n == 1 {
        let mut pstate = state.token.lock().unwrap();
        *pstate = Uuid::new_v4().to_string();
        Ok(Redirect::to(format!("/secret?token={}", *pstate)))
    } else if n > 255 {
        Err(Status::BadRequest)
    } else {
        Ok(Redirect::to(uri!(recursive_redirect(n - 1))))
    }
}

#[get("/secret?<token>")]
pub async fn secret(token: &str, _guard: RedirectGuard) -> Result<RawHtml<&'static str>, Status> {
    let html_content = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Secret Page</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            background-color: #fff;
            color: #000;
            margin: 0;
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
            height: 100vh;
        }
        .container {
            text-align: center;
            border: 2px solid #000;
            padding: 20px;
            border-radius: 10px;
            background-color: #f9f9f9;
            box-shadow: 0 0 20px rgba(0, 0, 0, 0.1);
        }
        .container h1 {
            font-size: 2.5rem;
            color: #000;
        }
        .container p {
            font-size: 1.2rem;
            margin-top: 10px;
            color: #333;
        }
        .container a {
            margin-top: 20px;
            display: inline-block;
            text-decoration: none;
            background-color: #000;
            color: #fff;
            padding: 10px 20px;
            border-radius: 5px;
            font-weight: bold;
        }
        .container a:hover {
            background-color: #333;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Welcome to the Secret Page</h1>
        <p>This is a hidden corner of the internet where secrets lie.</p>
        <p>Shhh... don't tell anyone about this page!</p>
        <a href="\#">Discover More</a>
    </div>
</body>
</html>"#;

    Ok(RawHtml(html_content))
}

#[get("/absolute-redirect")]
pub async fn absolute_redirect() -> Redirect {
    Redirect::permanent("http://oldcoolinnermorning.neverssl.com/online/")
}
