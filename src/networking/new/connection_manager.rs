use anyhow::Result;
use std::future::Future;

pub trait ConnectionManager {
    fn serve(self) -> impl Future<Output = Result<()>> + Send + Sync;
}
