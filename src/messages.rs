use actix::prelude::*;
use nitox::{commands::Message as NatsMessage, NatsError};

/// PublishMessage is a message that publishes a buffer to the NATS queue but doesn't wait for a reply afterwards.
/// Think of it as the Pub part in PubSub.
#[derive(Debug, Clone)]
pub struct PublishMessage {
    pub(crate) subject: String,
    pub(crate) data: Vec<u8>,
}

impl PublishMessage {
    pub fn new(subject: String, data: Vec<u8>) -> Self {
        PublishMessage { subject, data }
    }
}

impl Message for PublishMessage {
    type Result = Result<(), NatsError>;
}

/// Subscribe is a message that subscribes to a topic in the NATS queue
/// Think of it as the Sub part in PubSub.
/*#[derive(Debug, Clone)]
pub struct Subscribe {
    pub(crate) subject: String,
    pub(crate) unsub_after: Option<usize>,
}

impl Subscribe {
    pub fn new(subject: String, unsub_after: Option<usize>) -> Self {
        Subscribe {
            subject,
            unsub_after,
        }
    }
}

impl Message for Subscribe {
    type Result = Box<dyn Stream<Item = NatsMessage, Error = NatsError>>;
}*/

/// RequestWithReply creates a reply inbox in NATS, publishes a message (Request in terms of NATS grammar)
/// and waits for the reply to come back.
#[derive(Debug, Clone)]
pub struct RequestWithReply {
    pub(crate) subject: String,
    pub(crate) data: Vec<u8>,
}

impl RequestWithReply {
    pub fn new(subject: String, data: Vec<u8>) -> Self {
        RequestWithReply { subject, data }
    }
}

impl Message for RequestWithReply {
    type Result = Result<NatsMessage, NatsError>;
}
