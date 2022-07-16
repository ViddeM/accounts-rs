use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Request};

const SESSION_COOKIE_KEY: &str = "accounts-rs";

pub struct Session {}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("The client lacked a session cookie")]
    MissingCookie,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = SessionError;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        match request.cookies().get_private(SESSION_COOKIE_KEY) {
            Some(a) => println!("Cookie exists! {}", a),
            None => {
                return rocket::request::Outcome::Failure((
                    Status::Unauthorized,
                    SessionError::MissingCookie,
                ))
            }
        }

        rocket::request::Outcome::Success(Session {})
    }
}

pub fn set_session(cookies: &CookieJar<'_>, content: String) {
    cookies.add_private(
        Cookie::build(SESSION_COOKIE_KEY, content)
            .secure(true)
            .finish(),
    );
}
