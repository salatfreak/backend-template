use rocket::http::{Header, Status};
use serde::Deserialize;
use serde_json::json;

mod common;

#[test]
fn test_auth() {
    let client = common::client();

    // register with invalid email
    let resp = client.post("/api/auth/register").json(&json!({
        "name": "Alice",
        "email": "alice",
        "password": "supersecret",
    })).dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);

    // register
    let resp = client.post("/api/auth/register").json(&json!({
        "name": "Alice",
        "email": "alice@example.com",
        "password": "supersecret",
    })).dispatch();
    assert_eq!(resp.status(), Status::NoContent);

    // receive email and extract token
    let email = common::mailer(&client).receive_dummy().unwrap();
    let content = String::from_utf8(email.formatted()).unwrap();
    assert_eq!(&email.envelope().to()[0].to_string(), "alice@example.com");
    let token = content.split("\r\n")
        .find(|l| l.starts_with("Your registration token:")).unwrap()
        .rsplit_once(" ").unwrap().1;
    assert_eq!(token.len(), 32);

    // confirm registration with wrong token
    let resp = client.post("/api/auth/confirm")
        .json(&json!({ "token": "12345678911131517192123252729310" }))
        .dispatch();
    assert_eq!(resp.status(), Status::NotFound);

    // confirm registration
    let resp = client.post("/api/auth/confirm")
        .json(&json!({ "token": token })).dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let login: LoginResponse = resp.into_json().unwrap();
    assert_eq!(login.name, "Alice");
    assert_eq!(login.role, "user");

    // receive email
    let email = common::mailer(&client).receive_dummy().unwrap();
    let content = String::from_utf8(email.formatted()).unwrap();
    assert_eq!(&email.envelope().to()[0].to_string(), "alice@example.com");
    assert!(content.contains("Welcome aboard, Alice!"));

    // logout
    let header = format!("apikey {}", login.token);
    let resp = client.post("/api/auth/logout")
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::NoContent);

    // try logging out again
    let resp = client.post("/api/auth/logout")
        .header(Header::new("Authorization", header)).dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);

    // change password
    let resp = client.post("/api/auth/password/reset").json(&json!({
        "email": "alice@example.com",
    })).dispatch();
    assert_eq!(resp.status(), Status::NoContent);

    // receive email and extract token
    let email = common::mailer(&client).receive_dummy().unwrap();
    let content = String::from_utf8(email.formatted()).unwrap();
    assert_eq!(&email.envelope().to()[0].to_string(), "alice@example.com");
    let token = content.split("\r\n")
        .find(|l| l.starts_with("Your reset token:")).unwrap()
        .rsplit_once(" ").unwrap().1;
    assert_eq!(token.len(), 32);

    // confirm password with wrong token
    let resp = client.post("/api/auth/password/confirm").json(&json!({
        "token": "12345678911131517192123252729310", "password": "othersecret",
    })).dispatch();
    assert_eq!(resp.status(), Status::NotFound);

    // confirm password with wrong token
    let resp = client.post("/api/auth/password/confirm").json(&json!({
        "token": token, "password": "newsecret",
    })).dispatch();
    assert_eq!(resp.status(), Status::NoContent);

    // try logging in with wrong password
    let resp = client.post("/api/auth/login").json(&json!({
        "email": "alice@example.com",
        "password": "supersecret",
    })).dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);

    // login
    let resp = client.post("/api/auth/login").json(&json!({
        "email": "alice@example.com",
        "password": "newsecret",
    })).dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let login: LoginResponse = resp.into_json().unwrap();
    assert_eq!(login.name, "Alice");
}

#[derive(Deserialize)]
struct LoginResponse {
    name: String,
    role: String,
    token: String,
}