use tokio::task::JoinHandle;
use tracing::{Instrument, Span, error, info};

pub fn worker<F: Future<Output = ()> + Send + 'static>(fut: F, span: Span) -> JoinHandle<()> {
    tokio::spawn(
        async move {
            if let Err(e) = tokio::spawn(
                async move {
                    info!("worker started");
                    fut.await;
                    info!("worker stopped");
                }
                .instrument(Span::current()),
            )
            .await
            {
                error!("worker panicked: {:?}", e);
            }
        }
        .instrument(span),
    )
}
