use rocket::Route;
pub mod activate_account;
pub mod create_account;
pub mod index;
pub mod login;
pub mod login_successful;
pub mod logout;
pub mod password_reset;

pub mod password_validation;

pub fn core_routes() -> Vec<Route> {
    routes![
        index::index,
        login::get_login_page,
        login::post_login,
        create_account::create_account,
        create_account::get_create_account,
        login_successful::get_login_successful,
        activate_account::get_activate_account,
        password_reset::get_forgot_password,
        password_reset::post_forgot_password,
        password_reset::get_reset_password,
        password_reset::post_reset_password,
        logout::post_logout,
    ]
}
