use crate::common::Stock;
use crate::source::{Setting, Source};
use longport::quote::PushEventDetail::Quote;
use longport::quote::{PushEvent, SubFlags};
use longport::QuoteContext;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedReceiver;

pub struct LongPortSource {
    quote_ctx: QuoteContext,
    quote_receiver: UnboundedReceiver<PushEvent>,
}

#[async_trait]
impl Source for LongPortSource {
    async fn init(config: &mut Setting) -> Result<Self, String> {
        if let Some(long_port_config) = config.long_port.take() {
            let mut base_config = longport::Config::new(
                long_port_config.app_key,
                long_port_config.app_secret,
                long_port_config.access_token,
            );
            base_config = base_config.enable_overnight();
            let (ctx, receiver) = QuoteContext::try_new(Arc::new(base_config))
                .await
                .map_err(|e| e.to_string())?;
            Ok(Self {
                quote_ctx: ctx,
                quote_receiver: receiver,
            })
        } else {
            Err("Long Port TOKEN is nil".into())
        }
    }

    async fn subscribe(&mut self, symbol: &str) -> Result<(), String> {
        self.quote_ctx.subscribe([symbol], SubFlags::QUOTE, true).await.map_err(|e| e.to_string())
    }

    async fn unsubscribe(&mut self, symbol: &str) -> Result<(), String> {
        self.quote_ctx.unsubscribe([symbol], SubFlags::QUOTE).await.map_err(|e| e.to_string())
    }

    async fn recv(&mut self) -> Option<Stock> {
        if let Some(event) = self.quote_receiver.recv().await {
            translate_stock(event)
        } else {
            None
        }
    }
}

fn translate_stock(event: PushEvent) -> Option<Stock> {
    if let Quote(quote) = event.detail {
        Some(Stock {
            symbol: event.symbol,
            timestamp: quote.timestamp,
            last_done: quote.last_done,
            open: quote.open,
            high: quote.high,
            low: quote.low,
        })
    } else {
        None
    }
}
