use backend_template::{mail::Mailer, rocket};
use rocket::local::blocking::Client;
use std::env;

pub fn client() -> Client {
    // set config variables for reproducibility
    env::set_var("DATABASE_ADDRESS", "memory");
    env::set_var("DATABASE_NAMESPACE", "test");
    env::set_var("DATABASE_DATABASE", "test");
    env::set_var("MAIL_URL", "dummy");
    env::set_var("MAIL_FROM", "sender@example.com");
    env::set_var("API_OWNER", "owner@example.com:$argon2id$v=19$m=19456,t=2,p=1$Cs/sCdezmQUdBcu2ZM76rQ$c6Hg3Z0XLtVakCfGr+xazw96dDH5dRXm68r/9Jea2ks");
    env::set_var("FILES_PATH", "static");
    env::set_var("OPENAPI_ENABLE", "false");

    // construct rocket instance and test client
    Client::untracked(rocket()).expect("error creating test client")
}

#[allow(dead_code)]
pub fn mailer<'r>(client: &'r Client) -> &'r Mailer {
    client.rocket().state::<Mailer>().unwrap()
}
