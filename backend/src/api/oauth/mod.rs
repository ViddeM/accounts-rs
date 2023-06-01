use rocket::Route;

pub mod access_token;
pub mod authorize;

pub fn oauth_routes() -> Vec<Route> {
    routes![
        authorize::get_authorization,
        access_token::post_access_token,
    ]
}
