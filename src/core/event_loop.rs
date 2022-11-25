use super::events::Event;
use crate::core::actions_context::ActionsContext;
use crate::core::message_bus::{Message, MessageSender};
use log::error;
use std::sync::{Arc, Mutex};

pub trait EventProvider {
    fn next_event(&mut self) -> Option<Event>;
}

pub trait Actor<M: Message, MS: MessageSender<M>> {
    fn on_event(&mut self, event: &Event, actions_context: &mut ActionsContext<M, MS>);
}

pub struct EventLoop<T: EventProvider, S: Actor<M, MS>, M: Message, MS: MessageSender<M>> {
    event_provider: T,
    actors: Vec<Arc<Mutex<S>>>,
    actions_context: ActionsContext<M, MS>,
}

impl<T: EventProvider, S: Actor<M, MS>, M: Message, MS: MessageSender<M>> EventLoop<T, S, M, MS> {
    pub fn new(
        event_provider: T,
        actors: Vec<Arc<Mutex<S>>>,
        actions_context: ActionsContext<M, MS>,
    ) -> Self {
        Self {
            event_provider,
            actors,
            actions_context,
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
                        actor.on_event(&event, &mut self.actions_context);
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
