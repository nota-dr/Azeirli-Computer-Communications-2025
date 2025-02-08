extern crate rocket;

use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request;
use rocket::request::{FromRequest, Request};

use crate::TokenManager;

pub struct RedirectGuard;

#[rocket::async_trait]
impl<'a> FromRequest<'a> for RedirectGuard {
    type Error = ();

    async fn from_request(req: &'a Request<'_>) -> request::Outcome<Self, Self::Error> {
        let query_params = req.uri().query();

        if query_params.is_none() {
            return Outcome::Error((Status::BadRequest, ()));
        }

        let query_params = query_params.unwrap();
        let params: Vec<&str> = query_params.as_str().split("=").collect();

        if params.len() != 2 {
            return Outcome::Error((Status::BadRequest, ()));
        }

        let value = params[1];

        let token = req.rocket().state::<TokenManager>();
        if token.is_none() {
            println!("[!] TokenManager not found");
            return Outcome::Error((Status::InternalServerError, ()));
        }

        let token_guard = token.unwrap().token.lock().unwrap();
        let token = token_guard.as_str();
        match token {
            _ if token == value => Outcome::Success(RedirectGuard),
            _ => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}
