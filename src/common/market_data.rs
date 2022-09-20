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

    pub fn symbol(&self) -> Symbol {
        match self {
            Self::NewQuote(q) => q.symbol.clone(),
            Self::NewMarketTrade(t) => t.symbol.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub symbol: Symbol,
    pub exchange: Exchange,

    pub last_price: f64,
    pub last_size: f64,

    pub exchange_timestamp: Timestamp,
    pub received_timestamp: Timestamp,
}

#[derive(Debug, Serialize, Clone)]
pub struct Quote {
    pub symbol: Symbol,
    pub exchange: Exchange,

    pub bid: f64,
    pub ask: f64,
    pub bid_size: Option<f64>,
    pub ask_size: Option<f64>,

    pub exchange_timestamp: Timestamp,
    pub received_timestamp: Timestamp,
}
