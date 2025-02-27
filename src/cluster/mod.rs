use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "rust-tls")]
mod config_rustls;
mod config_tcp;
mod generic_connection_pool;
mod keyspace_holder;
mod pager;
#[cfg(feature = "rust-tls")]
mod rustls_connection_pool;
pub mod session;
mod tcp_connection_pool;

#[cfg(feature = "rust-tls")]
pub use crate::cluster::config_rustls::{
    ClusterRustlsConfig, NodeRustlsConfig, NodeRustlsConfigBuilder,
};
pub use crate::cluster::config_tcp::{ClusterTcpConfig, NodeTcpConfig, NodeTcpConfigBuilder};
pub use crate::cluster::keyspace_holder::KeyspaceHolder;
pub use crate::cluster::pager::{ExecPager, PagerState, QueryPager, SessionPager};
#[cfg(feature = "rust-tls")]
pub use crate::cluster::rustls_connection_pool::{
    new_rustls_pool, RustlsConnectionPool, RustlsConnectionsManager,
};
pub use crate::cluster::tcp_connection_pool::{
    new_tcp_pool, startup, TcpConnectionPool, TcpConnectionsManager,
};
pub(crate) use generic_connection_pool::ConnectionPool;

use crate::compression::Compression;
use crate::error;
use crate::frame::{Frame, StreamId};
use crate::query::{BatchExecutor, ExecExecutor, PrepareExecutor, QueryExecutor};
use crate::transport::CDRSTransport;

/// `GetConnection` trait provides a unified interface for Session to get a connection
/// from a load balancer
#[async_trait]
pub trait GetConnection<
    T: CDRSTransport + Send + Sync + 'static,
    M: bb8::ManageConnection<Connection = Mutex<T>, Error = error::Error>,
>
{
    /// Returns connection from a load balancer.
    async fn get_connection(&self) -> Option<Arc<ConnectionPool<M>>>;
}

/// `GetCompressor` trait provides a unified interface for Session to get a compressor
/// for further decompressing received data.
pub trait GetCompressor {
    /// Returns actual compressor.
    fn get_compressor(&self) -> Compression;
}

/// `ResponseCache` caches responses to match them by their stream id to requests.
#[async_trait]
pub trait ResponseCache {
    async fn match_or_cache_response(&self, stream_id: StreamId, frame: Frame) -> Option<Frame>;
}

/// `CDRSSession` trait wrap ups whole query functionality. Use it only if whole query
/// machinery is needed and direct sub traits otherwise.
pub trait CDRSSession<
    T: CDRSTransport + Unpin + 'static,
    M: bb8::ManageConnection<Connection = Mutex<T>, Error = error::Error>,
>:
    GetCompressor
    + GetConnection<T, M>
    + QueryExecutor<T, M>
    + PrepareExecutor<T, M>
    + ExecExecutor<T, M>
    + BatchExecutor<T, M>
{
}
