use crate::services::session_service::Session;
use rocket_dyn_templates::Template;
use std::collections::BTreeMap;

const LOGIN_SUCCESSFUL_TEMPLATE_NAME: &str = "login-successful";

#[get("/login_successful")]
pub async fn get_login_successful(_session: Session) -> Template {
    let data: BTreeMap<&str, String> = BTreeMap::new();
    Template::render(LOGIN_SUCCESSFUL_TEMPLATE_NAME, data)
}
