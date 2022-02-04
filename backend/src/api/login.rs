use crate::db::login_details::LoginDetailsRepository;
use rocket::form::Form;
use rocket::response::content::Html;
use rocket::response::Redirect;
use rocket::{Either, State};
use rocket_dyn_templates::Template;
use std::collections::BTreeMap;

const ERR_INVALID_PASSWORD: &str = "Invalid password";

#[derive(FromForm)]
pub struct LoginForm {
    email: String,
    password: String,
}

#[get("/login")]
pub async fn get_login_page() -> Html<Template> {
    let data: BTreeMap<&str, &str> = BTreeMap::new();
    Html(Template::render("login", &data))
}

#[post("/login", data = "<user_input>")]
pub async fn post_login(
    login_details_repository: &State<LoginDetailsRepository>,
    user_input: Form<LoginForm>,
) -> Either<Html<Template>, Redirect> {
    let mut data = BTreeMap::new();

    let login_details = match login_details_repository
        .get_by_email_and_password(&user_input.email, &user_input.password)
        .await
    {
        Err(err) => {
            println!("Failed communcating with DB: {:?}", err);
            data.insert("error", "Something went wrong");
            return Either::Left(Html(Template::render("login", data)));
        }
        Ok(val) => val,
    };

    match login_details {
        Some(_) => Either::Right(Redirect::to("/")),
        None => {
            data.insert("error", "Invalid password");
            Either::Left(Html(Template::render("login", data)))
        }
    }
}
