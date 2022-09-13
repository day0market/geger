use crate::common::uds::{OrderType, OrderUpdate, Side, TimeInForce, UDSMessage};

/*pub enum SimBrokerEvent {
    NewOrder(NewOrderEvent),
    NewOrderCancel(NewOrderCancelEvent),
    ExecutionEvent,
    OrderCancelEvent,
    MarketDataEvent,
    None,
}

impl SimBrokerEvent {
    pub fn get_timestamp(&self) -> u64 {
        match &self {
            SimBrokerEvent::NewOrder(event) => event.confirmation_timestamp,
            SimBrokerEvent::NewOrderCancel(event) => event.confirmation_timestamp,
            _ => 0u64, // TODO
        }
    }

    pub fn get_exchange(&self) -> String {
        match &self {
            SimBrokerEvent::NewOrder(event) => event.exchange.clone(),
            SimBrokerEvent::NewOrderCancel(event) => event.exchange.clone(),
            _ => "".to_string(), // TODO
        }
    }
}

pub struct NewOrderEvent {
    pub receive_timestamp: u64,
    pub confirmation_timestamp: u64,
    pub order: BrokerOrder,
}

#[derive(Clone)]
pub struct NewOrderCancelEvent {
    pub receive_timestamp: u64,
    pub confirmation_timestamp: u64,
    pub client_order_id: String,
    pub exchange_order_id: String,
    pub exchange: String,
    pub symbol: String,
}*/

pub struct BrokerOrder {
    pub exchange_order_id: Option<u64>,
    pub client_order_id: String,
    pub exchange: String,
    pub r#type: OrderType,
    pub time_in_force: TimeInForce,
    pub price: Option<f64>,
    pub trigger_price: Option<f64>,
    pub symbol: String,
    pub side: Side,
    pub quantity: f64,
}
