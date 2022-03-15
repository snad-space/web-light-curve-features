mod v0_1_17;

#[macro_use]
extern crate rocket;
use rocket::response::Redirect;

#[get("/help")]
fn help() -> Redirect {
    Redirect::to("https://github.com/hombit/web-light-curve-features")
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![v0_1_17::index, help])
        .mount("/api/v0.1.17/", routes![v0_1_17::index])
}
