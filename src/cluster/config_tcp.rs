use std::sync::Arc;
use std::time::Duration;

use crate::authenticators::Authenticator;

/// Cluster configuration that holds per node TCP configs
pub struct ClusterTcpConfig(pub Vec<NodeTcpConfig>);

/// Single node TCP connection config.
#[derive(Clone)]
pub struct NodeTcpConfig {
    pub addr: String,
    pub authenticator: Arc<dyn Authenticator + Send + Sync>,
    pub max_size: u32,
    pub min_idle: Option<u32>,
    pub max_lifetime: Option<Duration>,
    pub idle_timeout: Option<Duration>,
    pub connection_timeout: Duration,
}

/// Builder structure that helps to configure TCP connection for node.
pub struct NodeTcpConfigBuilder {
    addr: String,
    authenticator: Arc<dyn Authenticator + Send + Sync>,
    max_size: Option<u32>,
    min_idle: Option<u32>,
    max_lifetime: Option<Duration>,
    idle_timeout: Option<Duration>,
    connection_timeout: Option<Duration>,
}

impl NodeTcpConfigBuilder {
    const DEFAULT_MAX_SIZE: u32 = 10;
    const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

    pub fn new<S: ToString>(
        addr: S,
        authenticator: Arc<dyn Authenticator + Send + Sync>,
    ) -> NodeTcpConfigBuilder {
        NodeTcpConfigBuilder {
            addr: addr.to_string(),
            authenticator,
            max_size: None,
            min_idle: None,
            max_lifetime: None,
            idle_timeout: None,
            connection_timeout: None,
        }
    }

    /// Sets the maximum number of connections managed by the pool.
    /// Defaults to 10.
    pub fn max_size(mut self, size: u32) -> Self {
        self.max_size = Some(size);
        self
    }

    /// Sets the minimum idle connection count maintained by the pool.
    /// If set, the pool will try to maintain at least this many idle
    /// connections at all times, while respecting the value of `max_size`.
    /// Defaults to None (equivalent to the value of `max_size`).
    pub fn min_idle(mut self, min_idle: Option<u32>) -> Self {
        self.min_idle = min_idle;
        self
    }

    /// Sets the maximum lifetime of connections in the pool.
    /// If set, connections will be closed after existing for at most 30 seconds beyond this duration.
    /// If a connection reaches its maximum lifetime while checked out it will be closed when it is returned to the pool.
    /// Defaults to 30 minutes.
    pub fn max_lifetime(mut self, max_lifetime: Option<Duration>) -> Self {
        self.max_lifetime = max_lifetime;
        self
    }

    /// Sets the idle timeout used by the pool.
    /// If set, connections will be closed after sitting idle for at most 30 seconds beyond this duration.
    /// Defaults to 10 minutes.
    pub fn idle_timeout(mut self, idle_timeout: Option<Duration>) -> Self {
        self.idle_timeout = idle_timeout;
        self
    }

    /// Sets the connection timeout used by the pool.
    /// Defaults to 30 seconds.
    pub fn connection_timeout(mut self, connection_timeout: Duration) -> Self {
        self.connection_timeout = Some(connection_timeout);
        self
    }

    /// Sets new authenticator.
    pub fn authenticator(mut self, authenticator: Arc<dyn Authenticator + Send + Sync>) -> Self {
        self.authenticator = authenticator;
        self
    }

    /// Finalizes building process and returns `NodeSslConfig`
    pub fn build(self) -> NodeTcpConfig {
        NodeTcpConfig {
            addr: self.addr,
            authenticator: self.authenticator,

            max_size: self.max_size.unwrap_or(Self::DEFAULT_MAX_SIZE),
            min_idle: self.min_idle,
            max_lifetime: self.max_lifetime,
            idle_timeout: self.idle_timeout,
            connection_timeout: self
                .connection_timeout
                .unwrap_or(Self::DEFAULT_CONNECTION_TIMEOUT),
        }
    }
}
