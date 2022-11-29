use crate::core::actions_context::ActionsContext;
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{error, info, warn};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

pub type Topic = String;

pub trait Message: Clone + Debug {
    fn get_topic(&self) -> Option<Topic>;
    fn is_event_loop_stopped_message(&self) -> bool;
    fn new_event_loop_stopped_message() -> Self;
}

pub trait MessageHandler<M: Message, MS: MessageSender<M>>: Debug + Send {
    fn on_new_message(&mut self, message: &M, actions_context: &ActionsContext<M, MS>);
    fn get_topics(&self) -> Vec<Topic>;
}

pub trait MessageProvider<M: Message>: Debug {
    fn next_message(&mut self) -> Option<M>;
}

pub trait MessageSender<M: Message>: Debug + Clone {
    fn send_message(&mut self, message: M) -> Result<(), String>;
}

pub struct MessageBus<
    M: Message,
    T: MessageProvider<M>,
    H: MessageHandler<M, MS>,
    MS: MessageSender<M>,
> {
    phantom: PhantomData<M>,
    message_provider: T,
    message_handlers_by_topic: HashMap<Topic, Vec<Arc<Mutex<H>>>>,
    message_handlers_topic_agnostic: Vec<Arc<Mutex<H>>>,
    actions_context: ActionsContext<M, MS>,
    terminate_on_event_loop_stop_message: bool,
}

impl<M: Message, T: MessageProvider<M>, H: MessageHandler<M, MS>, MS: MessageSender<M>>
    MessageBus<M, T, H, MS>
{
    pub fn new(
        message_provider: T,
        message_handlers: Vec<Arc<Mutex<H>>>,
        actions_context: ActionsContext<M, MS>,
        terminate_on_event_loop_stop_message: bool,
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
            actions_context,
            terminate_on_event_loop_stop_message,
        }
    }

    fn send_message_to_handler(&self, handler: &Arc<Mutex<H>>, message: &M) {
        match &mut handler.lock() {
            Ok(handler) => handler.on_new_message(message, &self.actions_context),
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
            warn!("received new message in loop: {:?}", &message);
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

            if message.is_event_loop_stopped_message() && self.terminate_on_event_loop_stop_message
            {
                return;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct CrossbeamMessageSender<M: Message> {
    sender: Sender<M>,
    receiver: Receiver<M>,
}

impl<M: Message> CrossbeamMessageSender<M> {
    pub fn receiver(&self) -> Receiver<M> {
        return self.receiver.clone();
    }

    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}

impl<M: Message> MessageSender<M> for CrossbeamMessageSender<M> {
    fn send_message(&mut self, message: M) -> Result<(), String> {
        match self.sender.send(message) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("{:?}", err)),
        }
    }
}

#[derive(Debug)]
pub struct CrossbeamMessageProvider<M: Message> {
    receiver: Receiver<M>,
}

impl<M: Message> CrossbeamMessageProvider<M> {
    pub fn new_with_receiver(receiver: Receiver<M>) -> Self {
        Self { receiver }
    }
}

impl<M: Message> MessageProvider<M> for CrossbeamMessageProvider<M> {
    fn next_message(&mut self) -> Option<M> {
        match self.receiver.recv() {
            Ok(val) => Some(val),
            Err(err) => {
                error!("failed to receive new message: {:?}", err);
                None
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SimpleMessage {
    pub topic: Option<Topic>,
    pub event_loop_stopped: bool,
    pub content: String,
}

impl SimpleMessage {
    pub fn new(topic: Option<Topic>, content: String) -> Self {
        Self {
            topic,
            content,
            event_loop_stopped: false,
        }
    }
}

impl Message for SimpleMessage {
    fn get_topic(&self) -> Option<Topic> {
        self.topic.clone()
    }

    fn is_event_loop_stopped_message(&self) -> bool {
        self.event_loop_stopped
    }

    fn new_event_loop_stopped_message() -> Self {
        Self {
            topic: None,
            content: "".to_string(),
            event_loop_stopped: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct LoggerMessageHandler {}

impl<M: Message, MS: MessageSender<M>> MessageHandler<M, MS> for LoggerMessageHandler {
    fn on_new_message(&mut self, message: &M, _actions_context: &ActionsContext<M, MS>) {
        info!("new message: {:?}", message)
    }

    fn get_topics(&self) -> Vec<Topic> {
        vec![]
    }
}

pub fn start_message_bus<
    M: Message + Send + 'static,
    T: MessageProvider<M> + Send + 'static,
    H: MessageHandler<M, MS> + Send + 'static,
    MS: MessageSender<M> + Send + 'static,
>(
    message_provider: T,
    message_handlers: Vec<Arc<Mutex<H>>>,
    actions_context: ActionsContext<M, MS>,
    terminate_on_event_loop_stop_message: bool,
) {
    let mut message_bus = MessageBus::new(
        message_provider,
        message_handlers,
        actions_context,
        terminate_on_event_loop_stop_message,
    );
    message_bus.run()
}
