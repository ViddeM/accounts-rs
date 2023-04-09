use rocket::Route;

pub mod me;
pub mod oauth_client;
pub mod users;
pub mod whitelist;

pub fn site_routes() -> Vec<Route> {
    routes![
        me::get_me,
        users::get_users,
        whitelist::get_whitelist,
        whitelist::add_email_to_whitelist,
        whitelist::delete_email_from_whitelist,
        oauth_client::get_oauth_clients,
        oauth_client::post_new_client,
        oauth_client::delete_client,
    ]
}
