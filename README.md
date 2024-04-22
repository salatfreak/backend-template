# Rust Backend Template
This repository contains a template for conveniently building highly performant
and well-secured RESTful backends in [Rust][rust]. It relies on the
[Rocket][rocket] web framework and the *Rust*-based [SurrealDB][surreal]
database, and provides a project structure, testing, database migrations,
emailing support, proof of work, signup, and login logic, and OpenAPI
specification generation and inspection.

Take the route for retrieving a specific user for example:

```rust
#[get("/<id>")]
pub async fn route(
    user: Login<User>, db: &Database, id: &str
) -> Result<Json<UserOut>, Status> {
    // only allow reading own info if not admin
    let authorized = user.is(Admin) || user.id == id;
    if !authorized { return Err(Status::Forbidden); }

    // fetch user from database
    let user: Option<UserOut> = db.select(("user", id)).await
        .expect("error retrieving user");

    // return user or not found status
    user.map(Json).ok_or(Status::NotFound)
}
```

The function signature alone already specifies the ressource path, the
supported input data and output data, and restricts access to logged in users.
The first lines inside the function concisely limit access to administrators or
retrieving one's own user data, and the rest of the function elegantly
retrieves and serializes the data and correctly handles errors by returning
appropriate status codes.

The documentation for this repository is admittedly not very extensive but the
code should be rather clear on its own. Feel free to contribute to both the
code base and the documentation.

[rust]: https://www.rust-lang.org/
[rocket]: https://rocket.rs/
[surreal]: https://surrealdb.com/

## The Case for This Tech Stack
This tech stack is the result of the author's attempt to assemble the best open
source technologies for an online platform for registered users. It is intended
to be used in tandem with a single page application writting in Svelte or a
similar front-end technology.

The backend and front-end are deliberately developed separately to achive
separation of concerns and improve security. The backend is the most sensitive
part of the system and should maintain a small and efficient code base.

*Rust* is chosen as the programming language, because it focusses on
performance, correctness, security, and convenience like hardly any other one.
It outperforms any interpreted or garbage-collected languages while
guaranteeing memory safety and forcing the programmer to explicitly handle the
possible program states and exceptions. *Rust* libraries and programs tend to
require fewer bug fixes and run in a stable and reliable fashion.

*Rocket* is chosen as the web framework because of its elegant design and
concise interface. It allows you to achive more with less code and helps you to
maintain clarity over what your code does. Its extensibility makes it easy to
build features like the login request guard provided by this backend template.

*SurrealDB* is chosen as the database, because it is written in *Rust* and ties
in with it neatly, offers an excellent query language, and supports horizontal
scaling from the start. *SurrealDB* queries are often multiple times shorter
and way easier to understand compared to traditional *SQL* databases.

Database migrations, *OpenAPI* specification support, emailing, and proof of
work evaluation have been added to allow focussing on the actual data models
and features immediately.

## Development
When running the server during development, you'll still need a SurrealDB
database instance to connect to. You can simply start an in-memory instance
with podman by executing the following command.

```sh
podman run --rm --publish 127.0.0.1:8080:8080 \
  docker.io/surrealdb/surrealdb:v1.4.0 \
  start --no-banner -b 0.0.0.0:8080 --auth -u root -p root
```

This will also enable you to inspect the state of your database and test
queries using tools like [Surrealist][surrealist]. Surrealist additionally
allows you to design and export your database schemata to help you write your
migration files.

Besides the database, the backend also needs an SMTP server to connect to for
sending emails. You can specify "dummy" as the MAIL_URL to simply store the
emails in memory but a better alternative would be a fake SMTP server like
[smtp4dev][smtp4dev], which comes with a web interface for inspecting sent
emails. You can simply start it by executing the following command and open the
web interface in your browser on localhost port 5000.

```
podman run \
  --publish 127.0.0.1:2525:2525 \
  --publish 127.0.0.1:5000:5000 \
  docker.io/rnwood/smtp4dev:3.3.4 \
    --smtpport=2525 \
    --imapport=1433 \
    --urls=http://0.0.0.0:5000/ \
    --db=/tmp/database.sqlite
```

The backend's configuration in debug mode defaults to connecting to the
database and SMTP servers started with these parameters.

When [Cargo][cargo] is installed, the program can simply be compiled and
started using `cargo run`. *OpenSSL* needs to be installed on the system.

In debug mode, a [RapiDoc][rapidoc] web interface will by default be provided
under */doc* for testing the routes during development.

The test cases can be run by executing `cargo test`.

[surrealist]: https://surrealist.app/
[smtp4dev]: https://github.com/rnwood/smtp4dev
[rapidoc]: https://rapidocweb.com/
[cargo]: https://doc.rust-lang.org/cargo/

## Environment variables
The backend server can be configured using environment variables. Besides the
variables accepted by the *Rocket* framework, the following variables may be
used. Variables required for production are marked bold. See the next section
for some example values.

Name | Type | Default | Purpose
-----|------|---------|--------
**DATABASE_ADDRESS** | str | | SurrealDB address
DATABASE_USERNAME | str | | SurrealDB username
DATABASE_PASSWORD | str | | SurrealDB password
DATABASE_NAMESPACE | str | default | SurrealDB namespace
DATABASE_DATABASE | str | default | SurrealDB database name
**MAIL_URL** | str | | SMTP(S) URL
MAIL_POOL_SIZE | int | 1 | SMTP(S) connection pool size
**MAIL_FROM** | str | | email sender address
API_OWNER | str | | colon separated email address and password hash for owner
FILES_PATH | str | /usr/local/share/backend | path to static HTTP files and email templates
OPENAPI_ENABLE | bool | false | enable serving OpenAPI specification and RapiDoc frontend

## Deployment
The deployment is easiest using the container image. It can be built by
executing `podman build -t backend-template .`.

The following commands could be used to deploy the backend server with
a SurrealDB instance in a *podman* *pod*. Replace the database username,
password, the servers and email addresses accordingly for your deployment.

```sh
# create a shared pod
podman pod create -p 8000:8000 --exit-policy stop --name backend-template

# spawn a SurrealDB instance
podman run --detach --pod backend-template --name surrealdb \
  --volume surrealdb:/data \
  docker.io/surrealdb/surrealdb:v1.4.0 \
  start --no-banner -b 0.0.0.0:8080 \
    --auth -u USERNAME -p PASSWORD /data/database.db

# spawn the backend server
podman run --pod backend-template \
  --env DATABASE_ADDRESS='ws://surrealdb:8080/' \
  --env DATABASE_USERNAME='USERNAME' \
  --env DATABASE_PASSWORD='PASSWORD' \
  --env API_OWNER='OWNER@EXAMPLE.COM:$argon2id$v=19$m=19456,t=2,p=1$...' \
  --env MAIL_URL='smtps://MAILUSER:MAILPASSWORD@SMTP.EXAMPLE.COM' \
  --env MAIL_FROM='SENDER@EXAMPLE.COM' \
  backend-template
```

Systems like [docker-compose][compose] or [kubernetes][k8s] can of course be
used just as well. Since the backend server itself is stateless, multiple
instances can be spawned for load balancing.

[compose]: https://docs.docker.com/compose/
[k8s]: https://kubernetes.io/

*The software in this repository has been written by a human.*
