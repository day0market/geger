use crate::common::market_data::{Quote, Trade};
use crate::common::uds::UDSMessage;

#[derive(Debug)]
pub enum Event {
    MarketDataEvent(MarketDataEvent),
    UDSMessage(UDSMessage),
}

#[derive(Debug)]
pub enum MarketDataEvent {
    Trade(Trade),
    Quote(Quote),
}
