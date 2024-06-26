//! Sending and templating of emails.

use html_escape::encode_safe;
use lettre::{
    address::AddressError,
    message::{Mailbox, MultiPart},
    transport::smtp::{self, PoolConfig},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use rocket::{error, fairing::AdHoc, State};
use std::{
    path::PathBuf, sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex}
};
use tokio::{fs, io};

use crate::config::MailConfig;

/// Type alias for abbreviation in route handlers.
pub type Mail = State<Mailer>;

/// Create and mount mailer to the rocket instance.
pub fn mount(config: MailConfig, templates: PathBuf) -> AdHoc {
    let MailConfig { url, pool_size, from } = config;
    let conn = Mailer::new(&url, pool_size, &from, templates);

    AdHoc::try_on_ignite("SMTP Mailer", |rocket| async {
        match conn {
            Ok(conn) => Ok(rocket.manage(conn)),
            Err(err) => { error!("Mailer: {err:?}"); Err(rocket) }
        }
    })
}

/// Address, email, SMTP, and IO error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid email address")]
    Address { #[from] source: AddressError },
    #[error("invalid email format")]
    Email { #[from] source: lettre::error::Error },
    #[error("SMTP error")]
    Smtp { #[from] source: smtp::Error },
    #[error("error loading template")]
    IO { #[from] source: io::Error }
}

/// Email to be sent.
#[derive(Clone)]
pub struct Email {
    pub subject: String,
    pub text: String,
    pub html: String,
}

/// Email connection and sending interface.
#[derive(Clone)]
pub struct Mailer {
    transport: Transport,
    from: Mailbox,
    templates: PathBuf,
}

/// Transport enum for production SMTP or dummy queue.
#[derive(Clone)]
enum Transport {
    SMTP(AsyncSmtpTransport::<Tokio1Executor>),
    Dummy(DummyTransport),
}

/// Dummy mail transport exclusively intended for testing.
#[derive(Clone)]
struct DummyTransport {
    sender: Sender<Message>,
    receiver: Arc<Mutex<Receiver<Message>>>,
}

impl DummyTransport {
    /// Create new dummy transport.
    fn new() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver: Arc::new(Mutex::new(receiver)) }
    }

    /// Send message into the dummy transport.
    fn send(&self, message: Message) {
        self.sender.send(message).unwrap();
    }

    /// Receive next message from the dummy transport.
    fn receive(&self) -> Message {
        self.receiver.lock().unwrap().recv().unwrap()
    }
}

impl Mailer {
    // Create new mailer.
    pub fn new(
        url: &str, pool_size: u32, from: &str, templates: PathBuf,
    ) -> Result<Self, Error> {
        let transport = match url {
            "dummy" => Transport::Dummy(DummyTransport::new()),
            url => Transport::SMTP(
                AsyncSmtpTransport::<Tokio1Executor>::from_url(url)?
                    .pool_config(PoolConfig::new().max_size(pool_size))
                    .build()
            ),
        };

        Ok(Self { transport, from: from.parse()?, templates })
    }

    /// Send email to specified receiver.
    pub async fn send(&self, to: &str, email: Email) -> Result<(), Error> {
        // construct message object
        let message = Message::builder()
            .from(self.from.clone())
            .to(to.parse()?)
            .subject(&email.subject)
            .multipart(MultiPart::alternative_plain_html(
                email.text, email.html,
            ))?;

        // send message or store it for testing
        match &self.transport {
            Transport::SMTP(tpt) => { tpt.send(message).await?; }
            Transport::Dummy(tpt) => { tpt.send(message); }
        }

        // return successfully
        Ok(())
    }

    /// Load template and substitute variables.
    pub async fn template(
        &self, name: &str, vars: &[(&str, &str)]
    ) -> Result<Email, Error> {
        // load templates
        let path = self.templates.join(name);
        let mut subject = fs::read_to_string(path.join("subject.txt")).await?;
        let mut text = fs::read_to_string(path.join("content.txt")).await?;
        let mut html = fs::read_to_string(path.join("content.html")).await?;

        // substitute variables
        for (key, value) in vars {
            subject = subject.replace(&format!("{{{key}}}"), value);
            text = text.replace(&format!("{{{key}}}"), value);
            html = html.replace(&format!("{{{key}}}"), &encode_safe(value));
        }

        // construct email object
        Ok(Email { subject, text, html })
    }

    /// Receive next message if using dummy transport.
    pub fn receive_dummy(&self) -> Result<Message, ()> {
        match &self.transport {
            Transport::Dummy(tpt) => Ok(tpt.receive()),
            _ => Err(()),
        }
    }
}
