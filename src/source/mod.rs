pub mod long_port;

use crate::common::Stock;
use async_trait::async_trait;
pub use long_port::LongPortSource;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

/// long port's token struct
#[derive(Debug, Deserialize)]
pub struct LongPortToken {
    pub app_key: String,
    pub app_secret: String,
    pub access_token: String,
}

/// config for watch stock list
#[derive(Debug, Deserialize)]
pub struct StockConfig {
    pub symbol: String,
    pub name: String,
    pub source: String,
}

/// config for init source
#[derive(Debug, Deserialize)]
pub struct Setting {
    pub long_port: Option<LongPortToken>,
    pub stock: Vec<StockConfig>,
}

#[async_trait]
pub trait Source: Send {
    /// init the source
    async fn init(config: &mut Setting) -> Result<Self, String>
    where
        Self: Sized;

    /// add a stock to subscribe
    async fn subscribe(&mut self, symbol: &str) -> Result<(), String>;

    /// add a stock to unsubscribe
    async fn unsubscribe(&mut self, symbol: &str) -> Result<(), String>;

    /// main loop for receive event
    async fn recv(&mut self) -> Option<Stock>;
}

pub struct SourceManager {
    source_map: HashMap<String, Arc<Mutex<dyn Source + Send + Sync>>>,
    join_set: JoinSet<(String, Option<Stock>)>,
}

impl SourceManager {
    pub async fn new(setting: &mut Setting) -> Self {
        let mut source_map = HashMap::new();
        if let Ok(long_port) = LongPortSource::init(setting).await {
            source_map.insert(
                "long_port".to_string(),
                Arc::new(Mutex::new(long_port)) as Arc<Mutex<dyn Source + Send + Sync>>,
            );
        }

        for stock in &setting.stock {
            if let Some(source) = source_map.get_mut(stock.source.as_str()) {
                source.lock().await.subscribe(&stock.symbol).await.unwrap();
            }
        }

        let mut join_set = JoinSet::new();
        for (key, value) in source_map.iter() {
            let arc = value.clone();
            let key = key.clone();
            join_set.spawn(async move {
                let mut source = arc.lock().await;
                (key, source.recv().await)
            });
        }

        Self { source_map, join_set }
    }

    pub async fn recv(&mut self) -> Option<Stock> {
        if let Some(result) = self.join_set.join_next().await {
            match result {
                Ok((key, res)) => {
                    if let Some(source) = self.source_map.get(&key) {
                        let arc = source.clone();
                        let key = key.clone();
                        self.join_set.spawn(async move {
                            let mut source = arc.lock().await;
                            (key, source.recv().await)
                        });
                    };
                    res
                },
                Err(_) => None
            }
        } else {
            None
        }
    }
}
