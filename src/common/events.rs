use crate::common::market_data::{Quote, Trade};
use crate::common::uds::UDSMessage;

#[derive(Debug)]
pub enum Event {
    MarketDataEvent(MarketDataEvent),
    UDSMessage(UDSMessage),
    ExchangeResponse(ExchangeResponse),
}

impl Event {
    pub fn get_timestamp(&self) -> u64 {
        match self {
            Event::MarketDataEvent(event) => event.get_timestamp(),
            Event::UDSMessage(event) => event.get_timestamp(),
            Event::ExchangeResponse(event) => event.get_timestamp(),
        }
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

#[derive(Debug)]
pub enum ExchangeResponse {
    NewOrderAccepted(NewOrderAccepted),
    NewOrderRejected(NewOrderRejected),
    CancelOrderAccepted(CancelOrderAccepted),
    CancelOrderRejected(CancelOrderRejected),
}

impl ExchangeResponse {
    pub fn get_timestamp(&self) -> u64 {
        //TODO alex
        0
    }
}

#[derive(Debug)]
pub struct NewOrderAccepted {
    pub timestamp: u64,
    pub client_order_id: String,
    pub exchange_order_id: String,
}

#[derive(Debug)]
pub struct NewOrderRejected {
    pub timestamp: u64,
    pub client_order_id: String,
    pub reason: String,
}

#[derive(Debug)]
pub struct CancelOrderAccepted {
    pub timestamp: u64,
    pub client_order_id: String,
    pub exchange_order_id: String,
}

#[derive(Debug)]
pub struct CancelOrderRejected {
    pub timestamp: u64,
    pub client_order_id: String,
    pub exchange_order_id: Option<String>,
    pub reason: String,
}
