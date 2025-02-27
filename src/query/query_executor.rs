use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::cluster::{GetCompressor, GetConnection, ResponseCache};
use crate::error;
use crate::frame::{AsBytes, Frame};
use crate::query::{Query, QueryParams, QueryParamsBuilder, QueryValues};
use crate::transport::CDRSTransport;

use super::utils::{prepare_flags, send_frame};

#[async_trait]
pub trait QueryExecutor<
    T: CDRSTransport + Unpin + 'static,
    M: bb8::ManageConnection<Connection = Mutex<T>, Error = error::Error>,
>: GetConnection<T, M> + GetCompressor + ResponseCache + Sync
{
    async fn query_with_params_tw<Q: ToString + Send>(
        &self,
        query: Q,
        query_params: QueryParams,
        with_tracing: bool,
        with_warnings: bool,
    ) -> error::Result<Frame> {
        let query = Query {
            query: query.to_string(),
            params: query_params,
        };

        let flags = prepare_flags(with_tracing, with_warnings);

        let query_frame = Frame::new_query(query, flags);

        send_frame(self, query_frame.as_bytes(), query_frame.stream).await
    }

    /// Executes a query with default parameters:
    /// * TDB
    async fn query<Q: ToString + Send>(&self, query: Q) -> error::Result<Frame> {
        self.query_tw(query, false, false).await
    }

    /// Executes a query with ability to trace it and see warnings, and default parameters:
    /// * TBD
    async fn query_tw<Q: ToString + Send>(
        &self,
        query: Q,
        with_tracing: bool,
        with_warnings: bool,
    ) -> error::Result<Frame> {
        let query_params = QueryParamsBuilder::new().finalize();
        self.query_with_params_tw(query, query_params, with_tracing, with_warnings)
            .await
    }

    /// Executes a query with bounded values (either with or without names).
    async fn query_with_values<Q: ToString + Send, V: Into<QueryValues> + Send>(
        &self,
        query: Q,
        values: V,
    ) -> error::Result<Frame> {
        self.query_with_values_tw(query, values, false, false).await
    }

    /// Executes a query with bounded values (either with or without names)
    /// and ability to see warnings, trace a request and default parameters.
    async fn query_with_values_tw<Q: ToString + Send, V: Into<QueryValues> + Send>(
        &self,
        query: Q,
        values: V,
        with_tracing: bool,
        with_warnings: bool,
    ) -> error::Result<Frame> {
        let query_params_builder = QueryParamsBuilder::new();
        let query_params = query_params_builder.values(values.into()).finalize();
        self.query_with_params_tw(query, query_params, with_tracing, with_warnings)
            .await
    }

    /// Executes a query with query params without warnings and tracing.
    async fn query_with_params<Q: ToString + Send>(
        &self,
        query: Q,
        query_params: QueryParams,
    ) -> error::Result<Frame> {
        self.query_with_params_tw(query, query_params, false, false)
            .await
    }
}
