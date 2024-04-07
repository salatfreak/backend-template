use backend_template::rocket;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    rocket().ignite().await?.launch().await?;
    Ok(())
}
