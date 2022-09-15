use crate::common::types::{
    ClientOrderId, Exchange, ExchangeOrderId, OrderType, Side, Symbol, TimeInForce,
};
use crossbeam_channel::{SendError, Sender};
use std::collections::HashMap;

pub enum ExchangeRequest {
    NewOrder(NewOrderRequest),
    CancelOrder(CancelOrderRequest), // TODO Alex: think about naming
}

pub struct NewOrderRequest {
    pub client_order_id: ClientOrderId,
    pub exchange: Exchange,
    pub r#type: OrderType,
    pub time_in_force: TimeInForce,
    pub price: Option<f64>,
    pub trigger_price: Option<f64>,
    pub symbol: Symbol,
    pub quantity: f64,
    pub side: Side,
}

pub struct CancelOrderRequest {
    pub client_order_id: ClientOrderId,
    pub exchange_order_id: ExchangeOrderId,
    pub exchange: Exchange,
    pub symbol: Symbol,
}

#[derive(Debug)]
pub enum GatewayRouterError {
    UnknownExchange,
    SendError(SendError<ExchangeRequest>),
}

pub struct GatewayRouter {
    senders: HashMap<Exchange, Sender<ExchangeRequest>>,
}

impl GatewayRouter {
    pub fn new(senders: HashMap<Exchange, Sender<ExchangeRequest>>) -> Self {
        Self { senders }
    }

    pub fn send_request(&mut self, request: ExchangeRequest) -> Result<(), GatewayRouterError> {
        match request {
            ExchangeRequest::NewOrder(r) => self.send_order(r),
            ExchangeRequest::CancelOrder(c) => self.cancel_order(c),
        }
    }

    pub fn send_order(&mut self, request: NewOrderRequest) -> Result<(), GatewayRouterError> {
        let sender = match self.senders.get(&request.exchange) {
            Some(val) => val,
            None => return Err(GatewayRouterError::UnknownExchange),
        };
        match sender.send(ExchangeRequest::NewOrder(request)) {
            Ok(_) => Ok(()),
            Err(err) => Err(GatewayRouterError::SendError(err)),
        }
    }

    pub fn cancel_order(&mut self, request: CancelOrderRequest) -> Result<(), GatewayRouterError> {
        let sender = match self.senders.get(&request.exchange) {
            Some(val) => val,
            None => return Err(GatewayRouterError::UnknownExchange),
        };
        match sender.send(ExchangeRequest::CancelOrder(request)) {
            Ok(_) => Ok(()),
            Err(err) => Err(GatewayRouterError::SendError(err)),
        }
    }
}
