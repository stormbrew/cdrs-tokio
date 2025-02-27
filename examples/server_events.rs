use std::iter::Iterator;
use std::sync::Arc;

use cdrs_tokio::authenticators::NoneAuthenticator;
use cdrs_tokio::cluster::session::new as new_session;
use cdrs_tokio::cluster::{ClusterTcpConfig, NodeTcpConfigBuilder};
use cdrs_tokio::compression::Compression;
use cdrs_tokio::frame::events::{ChangeType, ServerEvent, SimpleServerEvent, Target};
use cdrs_tokio::load_balancing::RoundRobin;

const _ADDR: &'static str = "127.0.0.1:9042";

#[tokio::main]
async fn main() {
    let node = NodeTcpConfigBuilder::new("127.0.0.1:9042", Arc::new(NoneAuthenticator {})).build();
    let cluster_config = ClusterTcpConfig(vec![node]);
    let lb = RoundRobin::new();
    let no_compression = new_session(&cluster_config, lb)
        .await
        .expect("session should be created");

    let (listener, stream) = no_compression
        .listen(
            "127.0.0.1:9042",
            &NoneAuthenticator {},
            vec![SimpleServerEvent::SchemaChange],
        )
        .await
        .expect("listen error");

    tokio::spawn(listener.start(Compression::None));

    let new_tables = stream
        // inspects all events in a stream
        .inspect(|event| println!("inspect event {:?}", event))
        // filter by event's type: schema changes
        .filter(|event| event == &SimpleServerEvent::SchemaChange)
        // filter by event's specific information: new table was added
        .filter(|event| match event {
            &ServerEvent::SchemaChange(ref event) => {
                event.change_type == ChangeType::Created && event.target == Target::Table
            }
            _ => false,
        });

    println!("Start listen for server events");

    for change in new_tables {
        println!("server event {:?}", change);
    }
}
