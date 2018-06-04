extern crate nats;
extern crate actix;

use actix::prelude::*;

pub struct NATSExecutor(nats::Client);
impl Actor for NATSExecutor {
    type Context = SyncContext<Self>;
}

pub struct PublishMessage {
    subject: String,
    data: Vec<u8>
}

impl Message for PublishMessage {
    type Result = Result<(), nats::NatsError>;
}

impl Handler<PublishMessage> for NATSExecutor {
    type Result = Result<(), nats::NatsError>;

    fn handle(&mut self, msg: PublishMessage, _: &mut Self::Context) -> Self::Result {
        self.0.publish(&msg.subject, &msg.data)
    }
}

pub struct RequestWithReply {
    subject: String,
    data: Vec<u8>
}

impl Message for RequestWithReply {
    type Result = Result<String, nats::NatsError>;
}

impl Handler<RequestWithReply> for NATSExecutor {
    type Result = Result<String, nats::NatsError>;

    fn handle(&mut self, msg: RequestWithReply, _: &mut Self::Context) -> Self::Result {
        self.0.make_request(&msg.subject, &msg.data)
    }
}
