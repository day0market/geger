use super::types::{
    ClientOrderId, Exchange, ExchangeOrderId, ExchangeRequestID, OrderType, Side, Symbol,
    TimeInForce, Timestamp,
};
use crossbeam_channel::{unbounded, Receiver, SendError, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ExchangeRequest {
    NewOrder(NewOrderRequest),
    CancelOrder(CancelOrderRequest),
}

impl ExchangeRequest {
    pub fn creation_ts(&self) -> Timestamp {
        match self {
            ExchangeRequest::NewOrder(r) => r.creation_ts,
            ExchangeRequest::CancelOrder(r) => r.creation_ts,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct NewOrderRequest {
    pub request_id: ExchangeRequestID,
    pub client_order_id: ClientOrderId,
    pub exchange: Exchange,
    pub r#type: OrderType,
    pub time_in_force: TimeInForce,
    pub price: Option<f64>,
    pub trigger_price: Option<f64>,
    pub symbol: Symbol,
    pub quantity: f64,
    pub side: Side,
    pub creation_ts: Timestamp,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CancelOrderRequest {
    pub request_id: ExchangeRequestID,
    pub client_order_id: ClientOrderId,
    pub exchange_order_id: ExchangeOrderId,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub creation_ts: Timestamp,
}

#[derive(Debug)]
pub enum GatewayRouterError {
    UnknownExchange,
    SendError(SendError<ExchangeRequest>),
}

#[derive(Clone, Debug)]
pub struct GatewayRouter {
    senders: HashMap<Exchange, Sender<ExchangeRequest>>,
    receivers: HashMap<Exchange, Receiver<ExchangeRequest>>,
}

impl GatewayRouter {
    pub fn new(exchanges: Vec<Exchange>) -> Self {
        let mut receivers = HashMap::new();
        let mut senders = HashMap::new();
        for exchange in exchanges {
            let (sender, receiver) = unbounded();
            senders.insert(exchange.clone(), sender);
            receivers.insert(exchange, receiver);
        }
        Self { senders, receivers }
    }

    pub fn receivers(&self) -> HashMap<Exchange, Receiver<ExchangeRequest>> {
        self.receivers.clone()
    }

    pub(crate) fn send_request(
        &mut self,
        request: ExchangeRequest,
    ) -> Result<(), GatewayRouterError> {
        match request {
            ExchangeRequest::NewOrder(r) => self.send_order(r),
            ExchangeRequest::CancelOrder(c) => self.cancel_order(c),
        }
    }

    pub(crate) fn send_order(
        &mut self,
        request: NewOrderRequest,
    ) -> Result<(), GatewayRouterError> {
        let sender = match self.senders.get(&request.exchange) {
            Some(val) => val,
            None => return Err(GatewayRouterError::UnknownExchange),
        };
        match sender.send(ExchangeRequest::NewOrder(request)) {
            Ok(_) => Ok(()),
            Err(err) => Err(GatewayRouterError::SendError(err)),
        }
    }

    pub(crate) fn cancel_order(
        &mut self,
        request: CancelOrderRequest,
    ) -> Result<(), GatewayRouterError> {
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
