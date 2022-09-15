use crate::common::events::Event;
use crate::common::types::{ClientOrderId, ExchangeOrderId, OrderType, Side, Symbol, TimeInForce};
use crate::core::gateway_router::{ExchangeRequest, GatewayRouter};
use crossbeam_channel::{SendError, Sender};
use std::collections::HashMap;

use crate::common::types::Exchange;

pub trait EventProvider {
    fn next_event(&mut self) -> Option<Event>;
}

pub trait Actor {
    fn on_event(&mut self, event: &Event, gw_router: &mut GatewayRouter);
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
