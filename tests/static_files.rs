use rocket::http::{ContentType, Status};

mod common;

#[test]
fn test_index() {
    let client = common::client();
    let resp = client.get("/").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    assert_eq!(resp.content_type(), Some(ContentType::HTML));
    assert!(resp.into_string().unwrap().contains("assets/favicon.ico"));
}

#[test]
fn test_favicon() {
    let client = common::client();
    let resp = client.get("/assets/favicon.ico").dispatch();

    assert_eq!(resp.status(), Status::Ok);
    assert_eq!(resp.content_type(), Some(ContentType::Icon));
}

#[test]
fn test_not_found() {
    let client = common::client();

    let resp = client.get("/non-existent-path").dispatch();
    assert_eq!(resp.status(), Status::NotFound);
    assert_eq!(resp.content_type(), Some(ContentType::HTML));
    assert!(resp.into_string().unwrap().contains("assets/favicon.ico"));

    let resp = client.get("/assets/non-existent-path").dispatch();
    assert_eq!(resp.status(), Status::NotFound);
    assert_eq!(resp.content_type(), Some(ContentType::HTML));
    assert!(resp.into_string().unwrap().contains("assets/favicon.ico"));
}
