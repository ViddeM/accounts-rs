use rocket::Route;

pub mod access_token;
pub mod authorize;
pub mod consent;

pub fn oauth_routes() -> Vec<Route> {
    routes![
        authorize::get_authorization,
        access_token::post_access_token,
        access_token::get_access_token,
        consent::post_consent,
    ]
}
