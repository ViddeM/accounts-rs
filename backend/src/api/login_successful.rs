use crate::util::session::Session;
use rocket::response::content::Html;
use rocket_dyn_templates::Template;
use std::collections::BTreeMap;

const LOGIN_SUCCESSFUL_TEMPLATE_NAME: &str = "login-successful";

#[get("/login_successful")]
pub async fn get_login_successful(_session: Session) -> Html<Template> {
    let data: BTreeMap<&str, String> = BTreeMap::new();
    Html(Template::render(LOGIN_SUCCESSFUL_TEMPLATE_NAME, &data))
}
