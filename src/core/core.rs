use crate::common::events::{Event, MarketDataEvent};
use crate::common::market_data::{Quote, Trade};
use crate::common::uds::{OrderType, Side, TimeInForce, UDSMessage};
use crossbeam_channel::{unbounded, Receiver, SendError, Sender};
use std::collections::HashMap;

use crate::common::types::Exchange;
use log::error;

pub trait EventProvider {
    fn next_event(&mut self) -> Option<Event>;
}

pub trait Actor {
    fn on_event(&mut self, event: &Event, gw_router: &mut GatewayRouter);
}

pub enum ExchangeRequest {
    NewOrder(NewOrderRequest),
    CancelOrder(CancelOrderRequest), // TODO Alex: think about naming
}

pub struct NewOrderRequest {
    pub client_order_id: String,
    pub exchange: String,
    pub r#type: OrderType,
    pub time_in_force: TimeInForce,
    pub price: Option<f64>,
    pub trigger_price: Option<f64>,
    pub symbol: String,
    pub quantity: f64,
    pub side: Side,
}

pub struct CancelOrderRequest {
    pub client_order_id: String,
    pub exchange_order_id: String,
    pub exchange: String,
    pub symbol: String,
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

pub struct Core<T: EventProvider, S: Actor> {
    event_provider: T,
    strategy: S,
    gateway_router: GatewayRouter,
}

impl<T: EventProvider, S: Actor> Core<T, S> {
    pub fn new(
        event_provider: T,
        strategy: S,
        gw_router_senders: HashMap<Exchange, Sender<ExchangeRequest>>,
    ) -> Self {
        let gateway_router = GatewayRouter::new(gw_router_senders);
        Self {
            event_provider,
            strategy,
            gateway_router,
        }
    }

    pub fn run(&mut self) {
        'event_loop: loop {
            let event = self.event_provider.next_event();
            if event.is_none() {
                break 'event_loop;
            }
            let event = event.unwrap();
            self.strategy.on_event(&event, &mut self.gateway_router);
        }
    }
}
