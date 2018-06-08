extern crate nats;
extern crate actix;

use actix::prelude::*;
use std::marker::PhantomData;

pub struct NATSExecutor(nats::Client);
impl NATSExecutor {
    pub fn new(client: nats::Client) -> Self {
        NATSExecutor(client)
    }
}

impl Actor for NATSExecutor {
    type Context = SyncContext<Self>;
}

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

impl RequestWithReply {
    pub fn new(subject: String, data: Vec<u8>) -> Self {
        RequestWithReply {
            subject, data
        }
    }
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

pub struct WaitEventWithInbox<T> where T: From<Vec<u8>> {
    inbox: String,
    _meh: PhantomData<T>
}

impl<T: From<Vec<u8>>> WaitEventWithInbox<T> {
    pub fn new(inbox: String) -> Self {
        WaitEventWithInbox {
            inbox,
            _meh: PhantomData::default()
        }
    }
}

impl<T: 'static +  From<Vec<u8>>> Message for WaitEventWithInbox<T> {
    type Result = Result<T, String>;
}

impl<T: 'static +  From<Vec<u8>>> Handler<WaitEventWithInbox<T>> for NATSExecutor {
    type Result = Result<T, String>;

    fn handle(&mut self, msg: WaitEventWithInbox<T>, _: &mut Self::Context) -> Self::Result {
        for e in self.0.events() {
            if e.subject == msg.inbox {
                return Ok(e.msg.into());
            }
        }
        Err("No event found".into())
    }
}

