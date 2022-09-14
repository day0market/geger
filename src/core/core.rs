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

pub trait Strategy {
    fn on_trade(&mut self, trade: &Trade, gw_router: &mut GatewayRouter);
    fn on_quote(&mut self, quote: &Quote, gw_router: &mut GatewayRouter);
    fn on_uds(&mut self, uds: &UDSMessage, gw_router: &mut GatewayRouter);
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

pub struct Core<T: EventProvider, S: Strategy> {
    event_provider: T,
    strategy: S,
    gateway_router: GatewayRouter,
}

impl<T: EventProvider, S: Strategy> Core<T, S> {
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
                println!("event is none");
                break 'event_loop;
            }
            let event = event.unwrap();
            //println!("received event {:?}", &event);
            match event {
                Event::MarketDataEvent(event) => self.process_md_event(event),
                Event::UDSMessage(event) => self.process_uds(event),
                Event::ExchangeResponse(event) => {
                    todo!()
                }
            }
        }
    }

    fn process_md_event(&mut self, event: MarketDataEvent) {
        match event {
            MarketDataEvent::Quote(quote) => {
                self.strategy.on_quote(&quote, &mut self.gateway_router)
            }
            MarketDataEvent::Trade(trade) => {
                self.strategy.on_trade(&trade, &mut self.gateway_router)
            }
        }
    }

    fn process_uds(&mut self, event: UDSMessage) {
        self.strategy.on_uds(&event, &mut self.gateway_router)
    }
}
