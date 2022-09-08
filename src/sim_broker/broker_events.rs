use crate::common::uds::{OrderType, OrderUpdate, TimeInForce, UDSMessage};

pub enum SimBrokerEvent {
    PendingOrderEvent(PendingOrderEvent),
    PendingCancelEvent(PendingCancelEvent),
    ExecutionEvent,
    OrderConfirmationEvent,
    OrderRejectionEvent,
    OrderCancelEvent,
    CancelRejectedEvent,
    MarketDataEvent,
    None,
}

impl SimBrokerEvent {
    pub fn get_timestamp(&self) -> u64 {
        match &self {
            SimBrokerEvent::PendingOrderEvent(event) => event.confirmation_timestamp,
            SimBrokerEvent::PendingCancelEvent(event) => event.confirmation_timestamp,
            _ => 0u64, // TODO
        }
    }

    pub fn get_exchange(&self) -> String {
        match &self {
            SimBrokerEvent::PendingOrderEvent(event) => event.exchange.clone(),
            SimBrokerEvent::PendingCancelEvent(event) => event.exchange.clone(),
            _ => "".to_string(), // TODO
        }
    }
}

pub struct PendingOrderEvent {
    pub receive_timestamp: u64,
    pub confirmation_timestamp: u64,
    pub exchange_order_id: u64,
    pub client_order_id: String,
    pub exchange: String,
    pub r#type: OrderType,
    pub time_in_force: TimeInForce,
    pub price: Option<f64>,
    pub trigger_price: Option<f64>,
    pub symbol: String,
    pub quantity: f64,
}

pub struct PendingCancelEvent {
    pub receive_timestamp: u64,
    pub confirmation_timestamp: u64,
    pub client_order_id: String,
    pub exchange_order_id: String,
    pub exchange: String,
    pub symbol: String,
}
