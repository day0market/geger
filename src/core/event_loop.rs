use super::events::Event;
use super::gateway_router::{ExchangeRequest, GatewayRouter};
use crossbeam_channel::Sender;
use log::error;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::types::Exchange;

pub trait EventProvider {
    fn next_event(&mut self) -> Option<Event>;
}

pub trait Actor {
    fn on_event(&mut self, event: &Event, gw_router: &mut GatewayRouter);
}

pub struct EventLoop<T: EventProvider, S: Actor> {
    event_provider: T,
    actors: Vec<Arc<Mutex<S>>>,
    gateway_router: GatewayRouter,
}

impl<T: EventProvider, S: Actor> EventLoop<T, S> {
    pub fn new(
        event_provider: T,
        actors: Vec<Arc<Mutex<S>>>,
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
                match &mut actor.lock() {
                    Ok(actor) => {
                        actor.on_event(&event, &mut self.gateway_router);
                    }
                    Err(err) => {
                        error!("failed to process event because of mutex lock error: {:?}. event: {:?}", err, &event)
                    }
                }
            }
        }
    }

    pub fn get_actors(&self) -> &Vec<Arc<Mutex<S>>> {
        &self.actors
    }
}
