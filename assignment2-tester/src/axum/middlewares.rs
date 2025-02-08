use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use chrono::Utc;


pub async fn add_common_headers(req: Request, next: Next) -> Response {

    let mut res = next.run(req).await;
    res.headers_mut().insert(hyper::header::SERVER, HeaderValue::from_static("netcom-ex2"));
    res.headers_mut().insert(hyper::header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if !res.headers_mut().contains_key(hyper::header::DATE) {
        let date = Utc::now().format("%a %d %b %Y %H:%M:%S GMT").to_string();
        res.headers_mut().insert(hyper::header::DATE, HeaderValue::from_str(&date).unwrap());
    }
    res
}