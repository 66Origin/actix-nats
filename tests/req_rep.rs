extern crate actix;
extern crate actix_nats;
extern crate futures;
#[macro_use]
extern crate log;
extern crate env_logger;

use actix::prelude::*;
use actix_nats::{nitox, NATSActor, RequestWithReply};
use futures::{future, Future};

const ROUNDTRIP_COUNT: usize = 1_000_000;

#[test]
fn system_lots_of_requests() {
    let connect_cmd = nitox::commands::ConnectCommand::builder().build().unwrap();

    let opts = nitox::NatsClientOptions::builder()
        .connect_command(connect_cmd)
        .cluster_uri("localhost:4222")
        .build()
        .unwrap();

    let sys = System::new("system_lots_of_requests");
    let actor = NATSActor::start(opts);

    Arbiter::spawn_fn(move || {
        let mut fn_vec = vec![];

        for _ in 0..ROUNDTRIP_COUNT {
            let req = RequestWithReply::new("foo-requests".into(), "foo".into());
            fn_vec.push(actor.send(req).then(|res| {
                match res {
                    Ok(msg) => (),
                    _ => panic!("Should not happen {:?}", res),
                }

                Ok(())
            }));
        }

        future::join_all(fn_vec).map(|_| ())
    });

    sys.run();
}
