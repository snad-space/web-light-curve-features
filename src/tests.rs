use rocket::http::ContentType;
use rocket::local::blocking::Client;

/// Check if all API versions exist
#[test]
fn versions() {
    let client = Client::tracked(super::rocket()).unwrap();
    let versions = client
        .get("/versions")
        .dispatch()
        .into_json::<Vec<String>>()
        .unwrap();
    for version in versions {
        // I'm too lazy to send actual data, so no body is needed
        let req = client
            .post(format!("/api/{}/", version))
            .header(ContentType::JSON);
        let resp = req.dispatch();
        let status = resp.status();
        assert_ne!(status.code, 404, "{:?}", status.reason());
    }
}
