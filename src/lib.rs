extern crate actix;
extern crate nats;

use actix::prelude::*;

/// PublishMessage is a message that publishes a buffer to the NATS queue but doesn't wait for a reply afterwards.
/// Think of it as the Pub part in PubSub.
pub struct PublishMessage {
    subject: String,
    data: Vec<u8>,
}

impl PublishMessage {
    pub fn new(subject: String, data: Vec<u8>) -> Self {
        PublishMessage { subject, data }
    }
}

impl Message for PublishMessage {
    type Result = Result<(), nats::NatsError>;
}

/// RequestWithReply creates a reply inbox in NATS, publishes a message (Request in terms of NATS grammar)
/// and waits for the reply to come back.
pub struct RequestWithReply {
    subject: String,
    data: Vec<u8>,
    inbox: Option<String>,
}

impl RequestWithReply {
    pub fn new(subject: String, data: Vec<u8>) -> Self {
        RequestWithReply {
            subject,
            data,
            inbox: None,
        }
    }
}

impl Message for RequestWithReply {
    type Result = Result<Vec<u8>, nats::NatsError>;
}

/// Actor to give to Actix to do the background processing of NATS messages/requests
pub struct NATSExecutorSync(nats::Client);
impl NATSExecutorSync {
    fn new(client: nats::Client) -> Self {
        NATSExecutorSync(client)
    }

    /// Starts the executor. Give it a number of threads and a factory `Fn() -> nats::Client` that handles client creation and you're good to go.
    pub fn start<F>(threads: usize, client_factory: F) -> Addr<Self>
    where
        F: Fn() -> nats::Client + Send + Sync + 'static,
    {
        SyncArbiter::start(threads, move || Self::new(client_factory()))
    }
}

impl Actor for NATSExecutorSync {
    type Context = SyncContext<Self>;
}

impl Handler<PublishMessage> for NATSExecutorSync {
    type Result = Result<(), nats::NatsError>;

    fn handle(&mut self, msg: PublishMessage, _: &mut Self::Context) -> Self::Result {
        self.0.publish(&msg.subject, &msg.data)
    }
}

impl Handler<RequestWithReply> for NATSExecutorSync {
    type Result = Result<Vec<u8>, nats::NatsError>;

    fn handle(&mut self, mut msg: RequestWithReply, _: &mut Self::Context) -> Self::Result {
        msg.inbox = Some(self.0.make_request(&msg.subject, &msg.data)?);
        Ok(self.0.wait()?.msg)
    }
}
