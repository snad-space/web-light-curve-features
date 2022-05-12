#[cfg(test)]
mod tests;
mod v0_1;
mod v0_2;
mod v0_4;

#[macro_use]
extern crate rocket;
use rocket::response::Redirect;
use rocket::serde::json::Json;

#[get("/help")]
fn help() -> Redirect {
    Redirect::to("https://github.com/hombit/web-light-curve-features")
}

#[get("/versions")]
fn versions() -> Json<&'static [&'static str]> {
    Json(&["v0.1", "v0.2", "v0.4", "latest"])
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![v0_1::index, help, versions])
        .mount("/api/v0.1.17/", routes![v0_1::index])
        .mount("/api/v0.1/", routes![v0_1::index])
        .mount("/api/v0.2.2/", routes![v0_2::index])
        .mount("/api/v0.2/", routes![v0_2::index])
        .mount("/api/v0.4.3/", routes![v0_4::index])
        .mount("/api/v0.4/", routes![v0_4::index])
        .mount("/api/latest/", routes![v0_4::index])
}
