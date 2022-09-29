use crate::common::market_data::{MarketDataEvent, Quote, Trade};
use crate::common::types::{
    ClientOrderId, Exchange, ExchangeOrderId, ExecutionType, OrderStatus, OrderType, Side, Symbol,
    TimeInForce, Timestamp,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub enum Event {
    NewMarketTrade(Trade),
    NewQuote(Quote),
    ResponseNewOrderAccepted(NewOrderAccepted),
    ResponseNewOrderRejected(NewOrderRejected),
    ResponseCancelOrderAccepted(CancelOrderAccepted),
    ResponseCancelOrderRejected(CancelOrderRejected),
    UDSOrderUpdate(OrderUpdate),
}

impl Event {
    pub fn timestamp(&self) -> Timestamp {
        match self {
            Self::NewMarketTrade(t) => t.received_timestamp,
            Self::NewQuote(q) => q.received_timestamp,
            Self::ResponseNewOrderAccepted(r) => r.timestamp,
            Self::ResponseNewOrderRejected(r) => r.timestamp,
            Self::ResponseCancelOrderAccepted(r) => r.timestamp,
            Self::ResponseCancelOrderRejected(r) => r.timestamp,
            Self::UDSOrderUpdate(o) => o.timestamp,
        }
    }
}

impl From<MarketDataEvent> for Event {
    fn from(md_event: MarketDataEvent) -> Self {
        match md_event {
            MarketDataEvent::NewMarketTrade(trade) => Self::NewMarketTrade(trade),
            MarketDataEvent::NewQuote(quote) => Self::NewQuote(quote),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NewOrderAccepted {
    pub timestamp: Timestamp,
    pub client_order_id: ClientOrderId,
    pub exchange_order_id: ExchangeOrderId,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewOrderRejected {
    pub timestamp: Timestamp,
    pub client_order_id: ClientOrderId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderAccepted {
    pub timestamp: Timestamp,
    pub client_order_id: ClientOrderId,
    pub exchange_order_id: ExchangeOrderId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderRejected {
    pub timestamp: Timestamp,
    pub client_order_id: ClientOrderId,
    pub exchange_order_id: Option<ExchangeOrderId>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdate {
    pub timestamp: Timestamp,
    pub symbol: Symbol,
    pub exchange: Exchange,
    pub side: Side,
    pub client_order_id: Option<ClientOrderId>,
    pub exchange_order_id: Option<ExchangeOrderId>,
    pub order_type: Option<OrderType>,
    pub time_in_force: Option<TimeInForce>,
    pub original_qty: f64,
    pub original_price: Option<f64>,
    pub average_price: Option<f64>,
    pub stop_price: Option<f64>,
    pub execution_type: ExecutionType,
    pub order_status: OrderStatus,
    pub last_filled_qty: Option<f64>,
    pub accumulated_filled_qty: Option<f64>,
    pub last_filled_price: Option<f64>,
    pub last_trade_time: Option<Timestamp>,
}
