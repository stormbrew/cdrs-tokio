use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::cluster::{GetCompressor, GetConnection, ResponseCache};
use crate::error;
use crate::frame::traits::AsBytes;
use crate::frame::Frame;
use crate::query::batch_query_builder::QueryBatch;
use crate::transport::CDRSTransport;

use super::utils::{prepare_flags, send_frame};

#[async_trait]
pub trait BatchExecutor<
    T: CDRSTransport + Unpin + 'static,
    M: bb8::ManageConnection<Connection = Mutex<T>, Error = error::Error>,
>: GetConnection<T, M> + GetCompressor + ResponseCache + Sync
{
    async fn batch_with_params_tw(
        &self,
        batch: QueryBatch,
        with_tracing: bool,
        with_warnings: bool,
    ) -> error::Result<Frame> {
        let flags = prepare_flags(with_tracing, with_warnings);

        let query_frame = Frame::new_req_batch(batch, flags);

        send_frame(self, query_frame.as_bytes(), query_frame.stream).await
    }

    async fn batch_with_params(&self, batch: QueryBatch) -> error::Result<Frame> {
        self.batch_with_params_tw(batch, false, false).await
    }
}
