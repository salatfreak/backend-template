use rocket::{http::{Header, Status}, local::blocking::Client};
use serde::Deserialize;
use serde_json::json;

mod common;

#[test]
fn test_user() {
    let client = common::client();

    // login owner
    let owner = login(&client, "owner@example.com", "supersecret");

    // register user
    let login = register(&client, "Alice", "alice@example.com", "supersecret");
    assert_eq!(login.role, "user");
    let header = format!("apikey {}", login.token);

    // try listing user without token
    let resp = client.get("/api/users").dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);

    // try listing users
    let resp = client.get("/api/users")
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::Forbidden);

    // try updating name without token
    let resp = client.patch(format!("/api/users/{}", login.id))
        .json(&json!({ "name": "Bob" })).dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);

    // try updating owner name
    let resp = client.patch(format!("/api/users/{}", owner.id))
        .header(Header::new("Authorization", header.clone()))
        .json(&json!({ "name": "Pupsnase" })).dispatch();
    assert_eq!(resp.status(), Status::Forbidden);

    // update own name
    let resp = client.patch(format!("/api/users/{}", login.id))
        .header(Header::new("Authorization", header.clone()))
        .json(&json!({ "name": "Alicia" })).dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let user: UserResponse = resp.into_json().unwrap();
    assert_eq!(user.name, "Alicia");

    // try showing owner info
    let resp = client.get(format!("/api/users/{}", owner.id))
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::Forbidden);

    // show own info
    let resp = client.get(format!("/api/users/{}", login.id))
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let user: UserResponse = resp.into_json().unwrap();
    assert_eq!(user.name, "Alicia");

    // try deleting without token
    let resp = client.delete(format!("/api/users/{}", login.id)).dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);

    // try deleting owner
    let resp = client.delete(format!("/api/users/{}", owner.id))
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::Forbidden);

    // delete self
    let resp = client.delete(format!("/api/users/{}", login.id))
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::NoContent);

    // try logging in
    let resp = client.post("/api/auth/login").json(&json!({
        "email": "alice@example.com", "password": "supersecret",
    })).dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[test]
fn test_admin() {
    let client = common::client();

    // login owner
    let owner = login(&client, "owner@example.com", "supersecret");
    let owner_header = format!("apikey {}", owner.token);

    // register user
    let login = register(&client, "Alice", "alice@example.com", "supersecret");
    assert_eq!(login.role, "user");
    let header = format!("apikey {}", login.token);

    // make user admin
    let resp = client.patch(format!("/api/users/{}", login.id))
        .header(Header::new("Authorization", owner_header))
        .json(&json!({ "role": "admin" })).dispatch();
    assert_eq!(resp.status(), Status::Ok);

    // list users
    let resp = client.get("/api/users")
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let users: Vec<UserResponse> = resp.into_json().unwrap();
    assert_eq!(users.len(), 2);

    // try getting non-existent user
    let resp = client.get("/api/users/non-existent-user")
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::NotFound);

    // get owner info
    let resp = client.get(format!("/api/users/{}", owner.id))
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let user: UserResponse = resp.into_json().unwrap();
    assert_eq!(user.name, "Owner");

    // try changing own role
    let resp = client.patch(format!("/api/users/{}", login.id))
        .header(Header::new("Authorization", header.clone()))
        .json(&json!({ "role": "user" })).dispatch();
    assert_eq!(resp.status(), Status::Forbidden);

    // try deleting owner
    let resp = client.delete(format!("/api/users/{}", owner.id))
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::Forbidden);
}

#[test]
fn test_owner() {
    let client = common::client();

    // login owner
    let owner = login(&client, "owner@example.com", "supersecret");
    let header = format!("apikey {}", owner.token);

    // register other user
    let login = register(&client, "Alice", "alice@example.com", "supersecret");
    assert_eq!(login.role, "user");

    // change other user name and role
    let resp = client.patch(format!("/api/users/{}", login.id))
        .header(Header::new("Authorization", header.clone()))
        .json(&json!({ "name": "Alicia", "role": "admin" })).dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let user: UserResponse = resp.into_json().unwrap();
    assert_eq!(user.name, "Alicia");

    // delete other user
    let resp = client.delete(format!("/api/users/{}", login.id))
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::NoContent);

    // delete self
    let resp = client.delete(format!("/api/users/{}", owner.id))
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::NoContent);

    // try listing users
    let resp = client.get("/api/users")
        .header(Header::new("Authorization", header.clone())).dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

fn register(
    client: &Client, name: &str, email: &str, password: &str
) -> LoginResponse {
    // register
    let resp = client.post("/api/auth/register").json(&json!({
        "name": name, "email": email, "password": password,
    })).dispatch();
    assert_eq!(resp.status(), Status::NoContent);

    // receive email and extract token
    let email = common::mailer(&client).receive_dummy().unwrap();
    let content = String::from_utf8(email.formatted()).unwrap();
    let token = content.split("\r\n")
        .find(|l| l.starts_with("Your registration token:")).unwrap()
        .rsplit_once(" ").unwrap().1;

    // confirm registration
    let result = client.post("/api/auth/confirm")
        .json(&json!({ "token": token })).dispatch()
        .into_json().unwrap();

    // consume confirmation email
    common::mailer(&client).receive_dummy().unwrap();

    // return login object
    result
}

fn login(client: &Client, email: &str, password: &str) -> LoginResponse {
    client.post("/api/auth/login").json(&json!({
        "email": email, "password": password,
    })).dispatch().into_json().unwrap()
}


#[derive(Deserialize)]
struct LoginResponse {
    id: String,
    role: String,
    token: String,
}

#[derive(Deserialize)]
struct UserResponse {
    name: String,
}