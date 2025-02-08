use super::utils::helpers;
use super::GlobalState;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract, response};
use hmac::{Hmac, Mac};
use std::sync::{Arc, Mutex};
use serde::Deserialize;
use sha2::Sha256;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use tokio::io::AsyncReadExt;
use uuid::Uuid;


type HmacSha256 = Hmac<Sha256>;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Params {
    pub file: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Token {
    pub token: String,
}

pub async fn index() -> &'static str {
    "Welcome to EX2 tester!"
}

async fn load_file_into_response(file_name: String) -> Result<impl IntoResponse, u32> {
    let crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file_path = crate_path
        .join("src")
        .join("axum")
        .join("resources")
        .join(&file_name);

    match tokio::fs::File::open(&file_path).await {
        Ok(mut file) => {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await.map_err(|_e| 500 as u32)?;

            let body = axum::body::Body::from(buf);
            let mime = helpers::get_mime(&file_path).unwrap();

            Ok(Response::builder()
                .header(hyper::header::CONTENT_TYPE, mime)
                .body(body)
                .map_err(|_| 500 as u32)?)
        }
        Err(_) => Err(404),
    }
}

fn sign_token(secret: &[u8], ts: u64) -> String {
    let msg = format!("from recursive redirect: {}", ts);
    let mut hmac = HmacSha256::new_from_slice(secret).expect("HMAC can take key of any size");
    hmac.update(msg.as_bytes());
    let result = hmac.finalize().into_bytes();
    result
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

pub async fn resource_by_path_handler(
    extract::Path(file): extract::Path<String>,
) -> impl IntoResponse {
    let response = match load_file_into_response(file).await {
        Ok(response) => response.into_response(),
        Err(404) => (StatusCode::NOT_FOUND, helpers::err_template(404 as u32)).into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            helpers::err_template(500),
        )
            .into_response(),
    };
    response
}

pub async fn resource_by_query_handler(
    extract::Query(params): extract::Query<Params>,
) -> impl IntoResponse {
    let response = match load_file_into_response(params.file).await {
        Ok(response) => response.into_response(),
        Err(404) => (StatusCode::NOT_FOUND, helpers::err_template(404 as u32)).into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            helpers::err_template(500),
        )
            .into_response(),
    };
    response
}

pub async fn secret_handler(
    extract::Query(Token { token }): extract::Query<Token>,
    extract::State(state): extract::State<Arc<Mutex<GlobalState>>>,
) -> impl IntoResponse {
    let state = state.lock().unwrap();
    let expected_token = sign_token(&state.secret, state.timestemp);
    if token != expected_token {
        return (StatusCode::UNAUTHORIZED, helpers::err_template(401 as u32)).into_response();
    } else {
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
        return response::Html(html_content).into_response();
    }
}

pub async fn recursive_redirect_handler(
    extract::Path(n): extract::Path<String>,
    extract::State(state): extract::State<Arc<Mutex<GlobalState>>>,
) -> impl IntoResponse {
    let n: u8 = n.parse().unwrap_or(0);
    if n == 1 {
        // create secret
        let secret = Uuid::new_v4().as_bytes().to_vec();

        // create timestamp
        let ts = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        // sign the token
        let token = sign_token(&secret, ts);

        // update state
        let mut state = state.lock().unwrap();
        state.secret = secret;
        state.timestemp = ts;

        response::Redirect::permanent(&format!("/secret?token={}", token)).into_response()
    } else if n > 0 && n < 255 {
        response::Redirect::permanent(&format!("/recursive/{}", n - 1)).into_response()
    } else {
        println!("Bad request");
        (StatusCode::BAD_REQUEST, helpers::err_template(400 as u32)).into_response()
    }
}

pub async fn absolute_redirect_handler() -> impl IntoResponse {
    response::Redirect::permanent("http://www.pdf995.com/why.html").into_response()
}

pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, helpers::err_template(404 as u32)).into_response()
}
