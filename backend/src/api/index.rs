use rocket::response::Redirect;

#[get("/")]
pub fn index() -> Redirect {
    Redirect::permanent(uri!("/api/login"))
}
