use rocket::http::{Cookie, CookieJar};
use rocket::request::{FromRequest, Request};

const SESSION_COOKIE_KEY: &str = "session";

pub struct Session {}

#[derive(Debug)]
pub enum SessionError {}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = SessionError;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        match request.cookies().get_private(SESSION_COOKIE_KEY) {
            Some(a) => println!("Cookie exists! {}", a),
            None => println!("Cookie doesn't exist"),
        }

        rocket::request::Outcome::Success(Session {})
    }
}

pub fn set_session(cookies: &CookieJar<'_>, content: String) {
    cookies.add_private(Cookie::new(SESSION_COOKIE_KEY, content));
}
