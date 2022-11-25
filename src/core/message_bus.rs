use crate::core::gateway_router::GatewayRouter;
use crossbeam_channel::{Receiver, Sender};
use log::error;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

pub type Topic = String;

pub trait Message: Debug {
    fn get_topic(&self) -> Option<Topic>;
    fn is_stop_message(&self) -> bool;
}

pub trait MessageHandler<M: Message>: Debug {
    fn on_new_message(&mut self, message: &M, gw_router: &GatewayRouter);
    fn get_topics(&self) -> Vec<Topic>;
}

pub trait MessageProvider<M: Message> {
    fn next_message(&mut self) -> Option<M>;
}

pub trait MessageSender<M: Message> {
    fn send_message(&mut self, message: M) -> Result<(), String>;
}

pub struct MessageBus<M: Message, T: MessageProvider<M>, H: MessageHandler<M>> {
    phantom: PhantomData<M>,
    message_provider: T,
    message_handlers_by_topic: HashMap<Topic, Vec<Arc<Mutex<H>>>>,
    message_handlers_topic_agnostic: Vec<Arc<Mutex<H>>>,
    gw_router: GatewayRouter,
}

impl<M: Message, T: MessageProvider<M>, H: MessageHandler<M>> MessageBus<M, T, H> {
    pub fn new(
        message_provider: T,
        message_handlers: Vec<Arc<Mutex<H>>>,
        gw_router: GatewayRouter,
    ) -> Self {
        let mut message_handlers_by_topic: HashMap<Topic, Vec<Arc<Mutex<H>>>> = HashMap::new();
        let mut message_handlers_topic_agnostic = vec![];

        for handler in message_handlers {
            let topics = match handler.lock() {
                Ok(lock) => lock.get_topics(),
                Err(err) => {
                    panic!("failed to lock message handler: {:?}", err)
                }
            };
            if topics.is_empty() {
                message_handlers_topic_agnostic.push(handler)
            } else {
                for topic in topics {
                    let entry = message_handlers_by_topic.entry(topic).or_default();
                    entry.push(handler.clone())
                }
            }
        }
        Self {
            phantom: Default::default(),
            message_provider,
            message_handlers_by_topic,
            message_handlers_topic_agnostic,
            gw_router,
        }
    }

    fn send_message_to_handler(&self, handler: &Arc<Mutex<H>>, message: &M) {
        match &mut handler.lock() {
            Ok(handler) => handler.on_new_message(message, &self.gw_router),
            Err(err) => {
                error!(
                    "failed to lock handler mutex: {:?}. handler: {:?} message:{:?}",
                    err, &handler, &message
                )
            }
        }
    }
    pub fn run(&mut self) {
        'message_loop: loop {
            let message = self.message_provider.next_message();
            if message.is_none() {
                break 'message_loop;
            }

            let message = message.unwrap();
            let topic = message.get_topic();
            for handler in self.message_handlers_topic_agnostic.iter() {
                self.send_message_to_handler(handler, &message);
            }

            match topic {
                Some(topic) => {
                    if let Some(handlers) = self.message_handlers_by_topic.get(topic.as_str()) {
                        for handler in handlers {
                            self.send_message_to_handler(handler, &message);
                        }
                    }
                }
                None => {
                    for (_, handlers) in self.message_handlers_by_topic.iter() {
                        for handler in handlers {
                            self.send_message_to_handler(handler, &message);
                        }
                    }
                }
            };
        }
    }
}

pub struct CrossbeamMessageSender<M: Message> {
    sender: Sender<M>,
}

impl<M: Message> MessageSender<M> for CrossbeamMessageSender<M> {
    fn send_message(&mut self, message: M) -> Result<(), String> {
        match self.sender.send(message) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("{:?}", err)),
        }
    }
}

pub struct CrossbeamMessageProvider<M: Message> {
    receiver: Receiver<M>,
}

impl<M: Message> MessageProvider<M> for CrossbeamMessageProvider<M> {
    fn next_message(&mut self) -> Option<M> {
        match self.receiver.recv() {
            Ok(val) => {
                if val.is_stop_message() {
                    None
                } else {
                    Some(val)
                }
            }
            Err(err) => {
                error!("failed to receive new message: {:?}", err);
                None
            }
        }
    }
}