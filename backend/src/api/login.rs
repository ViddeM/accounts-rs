use crate::db::account::AccountRepository;
use rocket::form::Form;
use rocket::response::content::Html;
use rocket::response::Redirect;
use rocket::{Either, State};
use rocket_dyn_templates::Template;
use std::collections::BTreeMap;

#[derive(FromForm)]
pub struct LoginForm {
    email: String,
    password: String,
}

#[get("/login")]
pub async fn get_login_page() -> Html<Template> {
    let mut data: BTreeMap<&str, &str> = BTreeMap::new();
    Html(Template::render("login", &data))
}

#[post("/login", data = "<user_input>")]
pub async fn post_login(
    account_repository: &State<AccountRepository>,
    user_input: Form<LoginForm>,
) -> Either<Html<Template>, Redirect> {
    let mut data = BTreeMap::new();

    let account_res = account_repository
        .get_by_email_and_password(&user_input.email, &user_input.password)
        .await;

    let account = match account_res {
        Err(err) => {
            println!("Failed communcating with DB: {:?}", err);
            data.insert("error", "Something went wrong");
            return Either::Left(Html(Template::render("login", data)));
        }
        Ok(val) => val,
    };

    match account {
        Some(_) => Either::Right(Redirect::to("/")),
        None => {
            data.insert("error", "Invalid email/password");
            Either::Left(Html(Template::render("login", data)))
        }
    }
}
