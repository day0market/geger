use super::market_data::{MarketDataEvent, Quote, Trade};
use super::types::{
    ClientOrderId, EventId, Exchange, ExchangeOrderId, ExchangeRequestID, ExecutionType,
    OrderStatus, OrderType, Side, Symbol, TimeInForce, Timestamp,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Event {
    NewMarketTrade(Trade),
    NewQuote(Quote),
    ResponseNewOrderAccepted(NewOrderAccepted),
    ResponseNewOrderRejected(NewOrderRejected),
    ResponseCancelOrderAccepted(CancelOrderAccepted),
    ResponseCancelOrderRejected(CancelOrderRejected),
    UDSOrderUpdate(OrderUpdate),
}

impl From<MarketDataEvent> for Event {
    fn from(md_event: MarketDataEvent) -> Self {
        match md_event {
            MarketDataEvent::NewMarketTrade(trade) => Self::NewMarketTrade(trade),
            MarketDataEvent::NewQuote(quote) => Self::NewQuote(quote),
        }
    }
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

    pub fn exchange_timestamp(&self) -> Timestamp {
        match self {
            Self::NewMarketTrade(t) => t.exchange_timestamp,
            Self::NewQuote(q) => q.exchange_timestamp,
            Self::ResponseNewOrderAccepted(r) => r.exchange_timestamp,
            Self::ResponseNewOrderRejected(r) => r.exchange_timestamp,
            Self::ResponseCancelOrderAccepted(r) => r.exchange_timestamp,
            Self::ResponseCancelOrderRejected(r) => r.exchange_timestamp,
            Self::UDSOrderUpdate(o) => o.exchange_timestamp,
        }
    }

    pub fn exchange(&self) -> Exchange {
        match self {
            Self::NewMarketTrade(t) => t.exchange.clone(),
            Self::NewQuote(q) => q.exchange.clone(),
            Self::ResponseNewOrderAccepted(r) => r.exchange.clone(),
            Self::ResponseNewOrderRejected(r) => r.exchange.clone(),
            Self::ResponseCancelOrderAccepted(r) => r.exchange.clone(),
            Self::ResponseCancelOrderRejected(r) => r.exchange.clone(),
            Self::UDSOrderUpdate(o) => o.exchange.clone(),
        }
    }

    pub fn symbol(&self) -> Exchange {
        match self {
            Self::NewMarketTrade(t) => t.symbol.clone(),
            Self::NewQuote(q) => q.symbol.clone(),
            Self::ResponseNewOrderAccepted(r) => r.symbol.clone(),
            Self::ResponseNewOrderRejected(r) => r.symbol.clone(),
            Self::ResponseCancelOrderAccepted(r) => r.symbol.clone(),
            Self::ResponseCancelOrderRejected(r) => r.symbol.clone(),
            Self::UDSOrderUpdate(o) => o.symbol.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewOrderAccepted {
    pub event_id: EventId,
    pub request_id: Option<ExchangeRequestID>,
    pub timestamp: Timestamp,
    pub exchange_timestamp: Timestamp,
    pub client_order_id: ClientOrderId,
    pub exchange_order_id: ExchangeOrderId,
    pub exchange: Exchange,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewOrderRejected {
    pub event_id: EventId,
    pub request_id: Option<ExchangeRequestID>,
    pub timestamp: Timestamp,
    pub exchange_timestamp: Timestamp,
    pub client_order_id: ClientOrderId,
    pub reason: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CancelOrderAccepted {
    pub event_id: EventId,
    pub request_id: Option<ExchangeRequestID>,
    pub timestamp: Timestamp,
    pub exchange_timestamp: Timestamp,
    pub client_order_id: ClientOrderId,
    pub exchange_order_id: ExchangeOrderId,
    pub exchange: Exchange,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CancelOrderRejected {
    pub event_id: EventId,
    pub request_id: Option<ExchangeRequestID>,
    pub timestamp: Timestamp,
    pub exchange_timestamp: Timestamp,
    pub client_order_id: ClientOrderId,
    pub exchange_order_id: Option<ExchangeOrderId>,
    pub reason: String,
    pub exchange: Exchange,
    pub symbol: Symbol,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderUpdate {
    pub event_id: EventId,
    pub timestamp: Timestamp,
    pub exchange_timestamp: Timestamp,
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
