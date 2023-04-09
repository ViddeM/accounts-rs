use rocket::Route;

pub mod user;

pub fn external_routes() -> Vec<Route> {
    routes![user::get_user_info]
}
