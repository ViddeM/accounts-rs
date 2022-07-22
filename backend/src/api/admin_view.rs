use std::collections::BTreeMap;

use rocket_dyn_templates::Template;

use crate::services::admin_session_service::AdminSession;

const ADMIN_VIEW_TEMPLATE_NAME: &str = "admin-view";

#[get("/")]
pub async fn get_admin_view(admin_session: AdminSession) -> Template {
    let data: BTreeMap<&str, &str> =
        BTreeMap::from([("name", admin_session.account.first_name.as_str())]);

    Template::render(ADMIN_VIEW_TEMPLATE_NAME, &data)
}
