use nats;
use actix::prelude::*;

use super::*;

pub struct NATSExecutorSync(nats::Client);
impl NATSExecutorSync {
    pub fn new(client: nats::Client) -> Self {
        NATSExecutorSync(client)
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
    type Result = Result<String, nats::NatsError>;

    fn handle(&mut self, msg: RequestWithReply, _: &mut Self::Context) -> Self::Result {
        self.0.make_request(&msg.subject, &msg.data)
    }
}

impl<T: 'static +  From<Vec<u8>>> Handler<WaitEventWithInbox<T>> for NATSExecutorSync {
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
