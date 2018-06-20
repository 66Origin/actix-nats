extern crate nats;
extern crate actix;

use actix::prelude::*;

pub struct PublishMessage {
    subject: String,
    data: Vec<u8>
}

impl PublishMessage {
    pub fn new(subject: String, data: Vec<u8>) -> Self {
        PublishMessage {
            subject, data
        }
    }
}

impl Message for PublishMessage {
    type Result = Result<(), nats::NatsError>;
}

pub struct RequestWithReply {
    subject: String,
    data: Vec<u8>,
    inbox: Option<String>
}

impl RequestWithReply {
    pub fn new(subject: String, data: Vec<u8>) -> Self {
        RequestWithReply {
            subject,
            data,
            inbox: None
        }
    }
}

impl Message for RequestWithReply {
    type Result = Result<Vec<u8>, nats::NatsError>;
}

pub struct NATSExecutorSync(nats::Client);
impl NATSExecutorSync {
    fn new(client: nats::Client) -> Self {
        NATSExecutorSync(client)
    }

    pub fn start<F>(threads: usize, client_factory: F) -> Addr<Self>
        where F: Fn() -> nats::Client + Send + Sync + 'static
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
        Ok(self.0.wait()?.msg.into())
    }
}
