extern crate actix;
extern crate backoff;
extern crate futures;
#[macro_use]
extern crate log;
pub extern crate nitox;

use actix::prelude::*;
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use futures::{future, Future};
use nitox::{
    protocol::commands::{Message as NatsMessage, PubCommand as NatsPublish},
    NatsClient, NatsClientOptions, NatsError,
};

mod messages;
pub use self::messages::*;

/// Actor to give to Actix to do the background processing of NATS messages/requests
#[derive(Default)]
pub struct NATSActor {
    inner: Option<NatsClient>,
    backoff: ExponentialBackoff,
    opts: NatsClientOptions,
}

impl NATSActor {
    /// Start new `Supervisor` with `NATSActor`.
    pub fn start(opts: NatsClientOptions) -> Addr<NATSActor> {
        let mut backoff = ExponentialBackoff::default();
        backoff.max_elapsed_time = None;

        debug!(target: "actix-nats", "Starting Supervisor/Actor with opts {:#?}", opts);
        Supervisor::start(move |_| NATSActor {
            opts,
            backoff,
            inner: None,
        })
    }
}

impl Actor for NATSActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!(target: "actix-nats", "Starting client...");
        NatsClient::from_options(self.opts.clone())
            .and_then(|client| {
                debug!(target: "actix-nats", "Client created {:#?}", client);
                client.connect()
            }).into_actor(self)
            .map(|client, act, _| {
                info!(target: "actix-nats", "Connected to NATS server: {:#?}", client);
                act.inner = Some(client);
                act.backoff.reset();
            }).map_err(|err, act, ctx| {
                error!(target: "actix-nats", "Cannot connect to NATS server: {}", err);
                if let Some(timeout) = act.backoff.next_backoff() {
                    ctx.run_later(timeout, |_, ctx| ctx.stop());
                }
            }).wait(ctx);
    }
}

impl Supervised for NATSActor {
    fn restarting(&mut self, _: &mut Self::Context) {
        debug!(target: "actix-nats", "Supervisor restarted actor");
        self.inner.take();
        self.backoff.reset();
    }
}

impl Handler<PublishMessage> for NATSActor {
    type Result = ResponseFuture<(), NatsError>;

    fn handle(&mut self, msg: PublishMessage, _: &mut Self::Context) -> Self::Result {
        if let Some(ref mut client) = self.inner {
            let cmd = match NatsPublish::builder()
                .subject(msg.subject)
                .payload(msg.data)
                .build()
            {
                Ok(cmd) => cmd,
                Err(e) => return Box::new(future::err(NatsError::CommandBuildError(e))),
            };

            Box::new(client.publish(cmd))
        } else {
            error!(target: "actix-nats", "Cannot send message because client is not ready");
            Box::new(future::err(NatsError::ServerDisconnected(None)))
        }
    }
}

impl Handler<RequestWithReply> for NATSActor {
    type Result = ResponseFuture<NatsMessage, NatsError>;

    fn handle(&mut self, msg: RequestWithReply, _: &mut Self::Context) -> Self::Result {
        if let Some(ref mut client) = self.inner {
            debug!(target: "actix-nats", "Sending request with payload {:#?}", msg);
            Box::new(client.request(msg.subject, msg.data.into()))
        } else {
            error!(target: "actix-nats", "Cannot send message because client is not ready");
            Box::new(future::err(NatsError::ServerDisconnected(None)))
        }
    }
}
