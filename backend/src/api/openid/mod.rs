use rocket::Route;

pub mod configuration;
pub mod userinfo;

pub fn openid_routes() -> Vec<Route> {
    routes![userinfo::get_userinfo]
}
