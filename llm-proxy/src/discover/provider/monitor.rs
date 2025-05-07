use std::collections::HashMap;

use futures::future::BoxFuture;
use meltdown::Token;
use tokio::sync::mpsc::Sender;
use tower::discover::Change;
use tracing::info;

use crate::{
    dispatcher::DispatcherService, error::runtime, types::router::RouterId,
};

/// Could monitor health from URLs like:
///
/// https://status.openai.com/proxy/status.openai.com
///
/// or more creative methods if required.
pub struct ProviderMonitor<K> {
    _tx: Sender<Change<K, DispatcherService>>,
}

impl<K> ProviderMonitor<K> {
    pub fn new(tx: Sender<Change<K, DispatcherService>>) -> Self {
        Self { _tx: tx }
    }
}

pub struct ProviderMonitors<K> {
    _txs: HashMap<RouterId, ProviderMonitor<K>>,
}

impl<K> ProviderMonitors<K> {
    pub fn new(txs: HashMap<RouterId, ProviderMonitor<K>>) -> Self {
        Self { _txs: txs }
    }
}

impl<K: Send + 'static> meltdown::Service for ProviderMonitors<K> {
    type Future = BoxFuture<'static, Result<(), runtime::RuntimeError>>;

    fn run(self, token: Token) -> Self::Future {
        Box::pin(async move {
            token.await;
            info!(name = "provider-monitor-task", "task shutdown successfully");
            Ok(())
        })
    }
}
