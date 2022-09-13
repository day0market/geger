use crate::common::market_data::{Quote, Trade};
use crate::common::uds::UDSMessage;

#[derive(Debug)]
pub enum Event {
    MarketDataEvent(MarketDataEvent),
    UDSMessage(UDSMessage),
    ExchangeResponse,
}

impl Event {
    pub fn get_timestamp(&self) -> u64 {
        0
    }
}

#[derive(Debug)]
pub enum MarketDataEvent {
    Trade(Trade),
    Quote(Quote),
}

impl MarketDataEvent {
    pub fn get_timestamp(&self) -> u64 {
        return 0; // TODO
    }
    pub fn get_exchange(&self) -> &str {
        return ""; // TODO
    }
}
