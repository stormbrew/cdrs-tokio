### Cluster configuration

Apache Cassandra is designed to be a scalable and higly available database. So most often developers work with multi node Cassandra clusters. For instance Apple's setup includes 75k nodes, Netflix 2.5k nodes, Ebay >100 nodes.

That's why CDRS driver was designed with multinode support in mind. In order to connect to Cassandra cluster via CDRS connection configuration should be provided:

```rust
use cdrs_tokio::authenticators::NoneAuthenticator;
use cdrs_tokio::cluster::{ClusterTcpConfig, NodeTcpConfigBuilder, TcpConnectionPool};

fn main() {
  let node = NodeTcpConfigBuilder::new("127.0.0.1:9042", Arc::new(NoneAuthenticator {})).build();
  let cluster_config = ClusterTcpConfig(vec![node]);
}
```

`ClusterTcpConfig` receives a vector of Cassandra nodes configurations. `NodeTcpConfigBuilder` is a builder that provides methods for configuring bb8 pool of connections to a given node:

```rust
let node_address = "127.0.0.1:9042";
let authenticator = NoneAuthenticator {};
let node = NodeTcpConfigBuilder::new(node_address, authenticator)
  .max_size(5)
  .min_idle(4)
  .max_lifetime(Some(Duration::from_secs(60)))
  .idle_timeout(Duration::from_secs(60))
  .build();
let no_compression: CurrentSession =
  new_session(&cluster_config, RoundRobin::new()).expect("session should be created");
```

All existing `NodeTcpConfigBuilder` methods have the same behaviour as ones from `bb8::Builder`, so for more details please refer to [r2d2](https://docs.rs/r2d2/0.8.2/r2d2/struct.Builder.html) official documentation.

For each node configuration, `Authenticator` should be provided. `Authenticator` is a trait that the structure should implement so it can be used by CDRS session for authentication. Out of the box CDRS provides two types of authenticators:

- `cdrs_tokio::authenticators::NoneAuthenticator` that should be used if authentication is disabled by a node ([Cassandra authenticator](http://cassandra.apache.org/doc/latest/configuration/cassandra_config_file.html#authenticator) is set to `AllowAllAuthenticator`) on server.

- `cdrs_tokio::authenticators::PasswordAuthenticator` that should be used if authentication is enabled on the server and [authenticator](http://cassandra.apache.org/doc/latest/configuration/cassandra_config_file.html#authenticator) is `PasswordAuthenticator`.

```rust
use cdrs_tokio::authenticators::PasswordAuthenticator;
let authenticator = PasswordAuthenticator::new("user", "pass");
```

If a node has a custom authentication strategy, corresponded `Authenticator` should be implemented by a developer and further used in `NodeTcpConfigBuilder`.

To figure out how a custom `Authenticator` should be implemented refer to [src/authenticators.rs](https://github.com/AlexPikalov/cdrs/blob/master/src/authenticators.rs).

### Reference

1. Cassandra cluster configuration https://docs.datastax.com/en/cassandra/3.0/cassandra/initialize/initTOC.html.

2. ScyllaDB cluster configuration https://docs.scylladb.com/operating-scylla/ (see Cluster Management section).
