use crate::common::types::{Exchange, Symbol, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum MarketDataEvent {
    NewMarketTrade(Trade),
    NewQuote(Quote),
}

impl MarketDataEvent {
    pub fn timestamp(&self) -> Timestamp {
        match self {
            Self::NewQuote(Quote {
                received_timestamp, ..
            })
            | Self::NewMarketTrade(Trade {
                received_timestamp, ..
            }) => *received_timestamp,
        }
    }

    pub fn exchange(&self) -> Exchange {
        match self {
            Self::NewQuote(Quote { exchange, .. })
            | Self::NewMarketTrade(Trade { exchange, .. }) => exchange.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub received_timestamp: Timestamp, // TODO implement
    #[serde(default)]
    pub exchange: Exchange,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quote {
    #[serde(rename(deserialize = "s"))]
    pub symbol: Symbol,

    #[serde(default)]
    pub exchange: Exchange,
    #[serde(rename(deserialize = "b"))]
    pub bid: f64,
    #[serde(rename(deserialize = "a"))]
    pub ask: f64,
    #[serde(rename(deserialize = "bs"))]
    pub bid_size: Option<f64>,
    #[serde(rename(deserialize = "as"))]
    pub ask_size: Option<f64>,
    #[serde(rename(deserialize = "e"))]
    pub exchange_timestamp: Timestamp,
    #[serde(rename(deserialize = "r"))]
    pub received_timestamp: Timestamp,
}
