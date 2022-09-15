use crate::common::market_data::{MarketDataEvent, Quote, Trade};
use crate::common::types::{
    ClientOrderId, ExchangeOrderId, ExecutionType, OrderStatus, OrderType, Side, Symbol,
    TimeInForce, Timestamp,
};

#[derive(Debug)]
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
        0 // TODO
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

#[derive(Debug)]
pub struct NewOrderAccepted {
    pub timestamp: Timestamp,
    pub client_order_id: ClientOrderId,
    pub exchange_order_id: ExchangeOrderId,
}

#[derive(Debug)]
pub struct NewOrderRejected {
    pub timestamp: Timestamp,
    pub client_order_id: ClientOrderId,
    pub reason: String,
}

#[derive(Debug)]
pub struct CancelOrderAccepted {
    pub timestamp: Timestamp,
    pub client_order_id: ClientOrderId,
    pub exchange_order_id: ExchangeOrderId,
}

#[derive(Debug)]
pub struct CancelOrderRejected {
    pub timestamp: Timestamp,
    pub client_order_id: ClientOrderId,
    pub exchange_order_id: Option<ExchangeOrderId>,
    pub reason: String,
}

#[derive(Debug)]
pub struct OrderUpdate {
    pub timestamp: Timestamp,
    pub symbol: Symbol,
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
    pub last_trade_time: Option<f64>,
}
