use rocket::response::Redirect;

#[get("/")]
pub fn index() -> Redirect {
    Redirect::to(uri!("/api/core/login"))
}
