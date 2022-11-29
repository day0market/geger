use super::events::Event;
use crate::core::actions_context::ActionsContext;
use crate::core::message_bus::{Message, MessageSender};
use log::error;
use std::sync::{Arc, Mutex};

pub trait EventProvider {
    fn next_event(&mut self) -> Option<Event>;
}

pub trait Actor<M: Message, MS: MessageSender<M>>: Send {
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
        let term_message = M::new_event_loop_stopped_message();
        if let Err(err) = self.actions_context.send_message(term_message) {
            error!("failed to send action: {:?}", err)
        }
    }

    pub fn get_actors(&self) -> &Vec<Arc<Mutex<S>>> {
        &self.actors
    }
}

pub fn start_event_loop<
    T: EventProvider + Send + 'static,
    S: Actor<M, MS> + Send + 'static,
    M: Message + Send + 'static,
    MS: MessageSender<M> + Send + 'static,
>(
    event_provider: T,
    actors: Vec<Arc<Mutex<S>>>,
    actions_context: ActionsContext<M, MS>,
) {
    let mut event_loop = EventLoop::new(event_provider, actors, actions_context);
    event_loop.run()
}
