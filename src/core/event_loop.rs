use super::events::Event;
use super::gateway_router::{ExchangeRequest, GatewayRouter};
use crossbeam_channel::Sender;
use std::collections::HashMap;

use crate::core::types::Exchange;

pub trait EventProvider {
    fn next_event(&mut self) -> Option<Event>;
}

pub trait Actor {
    fn on_event(&mut self, event: &Event, gw_router: &mut GatewayRouter);
}

pub struct EventLoop<T: EventProvider, S: Actor> {
    event_provider: T,
    actors: Vec<S>,
    gateway_router: GatewayRouter,
}

impl<T: EventProvider, S: Actor> EventLoop<T, S> {
    pub fn new(
        event_provider: T,
        actors: Vec<S>,
        gw_router_senders: HashMap<Exchange, Sender<ExchangeRequest>>,
    ) -> Self {
        let gateway_router = GatewayRouter::new(gw_router_senders);
        Self {
            event_provider,
            actors,
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
            for actor in &mut self.actors {
                actor.on_event(&event, &mut self.gateway_router);
            }
        }
    }

    pub fn get_actors(&self) -> &Vec<S> {
        &self.actors
    }
}
