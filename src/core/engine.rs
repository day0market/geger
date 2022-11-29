use crate::core::actions_context::ActionsContext;
use crate::core::event_loop::{start_event_loop, Actor, EventLoop, EventProvider};
use crate::core::gateway_router::GatewayRouter;
use crate::core::message_bus::{
    start_message_bus, CrossbeamMessageProvider, CrossbeamMessageSender, LoggerMessageHandler,
    Message, MessageHandler, MessageSender, SimpleMessage,
};
use crate::core::types::{Exchange, Latency};
use crate::sim::broker::{SimBroker, SimBrokerConfig};
use crate::sim::environment::{SimulatedEnvironment, SimulatedTradingMarketDataProvider};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Engine<
    S: Actor<M, MS>,
    M = SimpleMessage,
    MS = CrossbeamMessageSender<SimpleMessage>,
    H = LoggerMessageHandler,
> where
    H: MessageHandler<M, MS>,
    M: Message,
    MS: MessageSender<M>,
{
    phantom: PhantomData<M>,
    phantom2: PhantomData<MS>,
    actors: Vec<Arc<Mutex<S>>>,
    message_handlers: Vec<Arc<Mutex<H>>>,
    exchanges: Vec<Exchange>,
}

impl<S: Actor<M, MS>, M: Message, MS: MessageSender<M>, H: MessageHandler<M, MS>>
    Engine<S, M, MS, H>
{
    pub fn new() -> Self {
        Self {
            phantom: Default::default(),
            phantom2: Default::default(),
            actors: vec![],
            exchanges: vec![],
            message_handlers: vec![],
        }
    }
    pub fn add_actor(&mut self, actor: Arc<Mutex<S>>) {
        self.actors.push(actor);
    }

    pub fn add_message_handler(&mut self, message_handler: Arc<Mutex<H>>) {
        self.message_handlers.push(message_handler);
    }

    pub fn add_exchange(&mut self, exchange: Exchange) {
        self.exchanges.push(exchange)
    }

    pub fn run_with_event_provider_custom_messaging<T: EventProvider>(
        self,
        event_provider: T,
        message_sender: MS,
    ) {
        let gateway_router = GatewayRouter::new(self.exchanges.clone());

        let actions_context = ActionsContext::new_with_sender(gateway_router, message_sender);
        let mut event_loop = EventLoop::new(event_provider, self.actors, actions_context);

        event_loop.run()
    }
}

impl<
        S: Actor<SimpleMessage, CrossbeamMessageSender<SimpleMessage>> + 'static,
        H: MessageHandler<SimpleMessage, CrossbeamMessageSender<SimpleMessage>> + 'static,
    > Engine<S, SimpleMessage, CrossbeamMessageSender<SimpleMessage>, H>
{
    pub fn run_with_event_provider<T: EventProvider>(self, event_provider: T, run_messaging: bool) {
        let gateway_router = GatewayRouter::new(self.exchanges.clone());
        let message_sender = CrossbeamMessageSender::new();
        let actions_context = match run_messaging {
            true => ActionsContext::new_with_sender(gateway_router, message_sender),
            false => ActionsContext::new(gateway_router),
        };

        let mut event_loop = EventLoop::new(event_provider, self.actors, actions_context);
        event_loop.run()
    }

    pub fn execute_in_sim_environment<T: SimulatedTradingMarketDataProvider + Send + 'static>(
        self,
        md_provider: T,
        default_latency: Option<Latency>,
        sim_broker_configs: HashMap<Exchange, SimBrokerConfig>,
        run_messaging: bool,
    ) {
        let gateway_router = GatewayRouter::new(self.exchanges.clone());
        let mut sim_env = SimulatedEnvironment::new(md_provider, default_latency);

        for (exchange, gw_receiver) in gateway_router.receivers() {
            let conf = match sim_broker_configs.get(&exchange) {
                Some(val) => val.clone(),
                None => SimBrokerConfig::default(),
            };

            let sim_broker = SimBroker::new(exchange.clone(), gw_receiver, conf);
            if let Err(err) = sim_env.add_broker(sim_broker) {
                panic!("{:?}", err)
            };
        }

        let (actions_context, message_receiver) = match run_messaging {
            true => {
                let message_sender = CrossbeamMessageSender::new();
                let receiver = message_sender.receiver();
                (
                    ActionsContext::new_with_sender(gateway_router, message_sender),
                    Some(receiver),
                )
            }
            false => (ActionsContext::new(gateway_router), None),
        };

        let mut threads = vec![];

        {
            let actions_context = actions_context.clone();
            let actors = self.actors.clone();
            threads.push(
                thread::Builder::new()
                    .name("run_event_loop".to_string())
                    .spawn(move || start_event_loop(sim_env, actors, actions_context)),
            );
        }

        if run_messaging {
            let message_provider =
                CrossbeamMessageProvider::new_with_receiver(message_receiver.unwrap());
            let message_handlers = self.message_handlers.clone();
            let actions_context = actions_context.clone();
            threads.push(
                thread::Builder::new()
                    .name("run_message_bus".to_string())
                    .spawn(move || {
                        start_message_bus(message_provider, message_handlers, actions_context, true)
                    }),
            );
        }

        for th in threads {
            th.unwrap().join().unwrap()
        }
    }
}
