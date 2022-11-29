use crate::core::actions_context::ActionsContext;
use crate::core::event_loop::{start_event_loop, Actor, EventProvider};
use crate::core::gateway_router::{ExchangeRequest, GatewayRouter};
use crate::core::message_bus::{
    start_message_bus, CrossbeamMessageProvider, CrossbeamMessageSender, LoggerMessageHandler,
    Message, MessageHandler, MessageProvider, MessageSender, SimpleMessage,
};
use crate::core::types::{Exchange, Latency};
use crate::sim::broker::{SimBroker, SimBrokerConfig};
use crate::sim::environment::{SimulatedEnvironment, SimulatedTradingMarketDataProvider};
use crossbeam_channel::Receiver;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{io, thread};

pub struct EngineExecutionInfo {
    pub threads: Vec<io::Result<JoinHandle<()>>>,
    pub exchange_requests_receivers: HashMap<Exchange, Receiver<ExchangeRequest>>,
}

#[derive(Debug)]
pub enum EngineError {
    MissedParameter(String),
    Initialization(String),
}

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

impl<
        S: Actor<M, MS> + 'static,
        M: Message + Send + 'static,
        MS: MessageSender<M> + Send + 'static,
        H: MessageHandler<M, MS> + 'static,
    > Engine<S, M, MS, H>
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

    pub fn run_with_event_provider_custom_messaging<
        T: EventProvider + Send + 'static,
        MP: MessageProvider<M> + Send + 'static,
    >(
        self,
        event_provider: T,
        message_sender: MS,
        message_provider: MP,
    ) -> Result<EngineExecutionInfo, EngineError> {
        let gateway_router = GatewayRouter::new(self.exchanges.clone());
        let exchange_requests_receivers = gateway_router.receivers();
        let actions_context = ActionsContext::new_with_sender(gateway_router, message_sender);
        let threads = self.start_threads(
            event_provider,
            actions_context,
            Some(message_provider),
            true,
            None,
        )?;

        Ok(EngineExecutionInfo {
            threads,
            exchange_requests_receivers,
        })
    }

    fn start_threads<T: EventProvider + Send + 'static, MP: MessageProvider<M> + Send + 'static>(
        &self,
        event_provider: T,
        actions_context: ActionsContext<M, MS>,
        message_provider: Option<MP>,
        run_messaging: bool,
        terminate_messaging_on_event_loop_stop: Option<bool>,
    ) -> Result<Vec<io::Result<JoinHandle<()>>>, EngineError> {
        if run_messaging && message_provider.is_none() {
            return Err(EngineError::MissedParameter(
                "message provider is empty".into(),
            ));
        }

        let mut threads = vec![];
        {
            let actions_context = actions_context.clone();
            let actors = self.actors.clone();
            threads.push(
                thread::Builder::new()
                    .name("event_loop_thread".to_string())
                    .spawn(move || start_event_loop(event_provider, actors, actions_context)),
            );
        }

        if run_messaging {
            let message_provider = message_provider.unwrap();
            let message_handlers = self.message_handlers.clone();
            let actions_context = actions_context.clone();
            threads.push(
                thread::Builder::new()
                    .name("message_bus_thread".to_string())
                    .spawn(move || {
                        start_message_bus(
                            message_provider,
                            message_handlers,
                            actions_context,
                            terminate_messaging_on_event_loop_stop.unwrap_or(false),
                        )
                    }),
            );
        };

        Ok(threads)
    }
}

impl<
        S: Actor<SimpleMessage, CrossbeamMessageSender<SimpleMessage>> + 'static,
        H: MessageHandler<SimpleMessage, CrossbeamMessageSender<SimpleMessage>> + 'static,
    > Engine<S, SimpleMessage, CrossbeamMessageSender<SimpleMessage>, H>
{
    fn create_actions_context_with_default_message_provider(
        &self,
        run_messaging: bool,
    ) -> (
        ActionsContext<SimpleMessage, CrossbeamMessageSender<SimpleMessage>>,
        Option<CrossbeamMessageProvider<SimpleMessage>>,
    ) {
        let gateway_router = GatewayRouter::new(self.exchanges.clone());
        match run_messaging {
            true => {
                let message_sender = CrossbeamMessageSender::new();
                let receiver = message_sender.receiver();
                (
                    ActionsContext::new_with_sender(gateway_router, message_sender),
                    Some(CrossbeamMessageProvider::new_with_receiver(receiver)),
                )
            }
            false => (ActionsContext::new(gateway_router), None),
        }
    }
    pub fn start_with_event_provider<T: EventProvider + Send + 'static>(
        self,
        event_provider: T,
        run_messaging: bool,
    ) -> Result<EngineExecutionInfo, EngineError> {
        let (actions_context, message_provider) =
            self.create_actions_context_with_default_message_provider(run_messaging);

        let exchange_requests_receivers = actions_context.exchange_requests_receivers();

        let threads = self.start_threads(
            event_provider,
            actions_context,
            message_provider,
            run_messaging,
            None,
        )?;

        Ok(EngineExecutionInfo {
            threads,
            exchange_requests_receivers,
        })
    }

    pub fn execute_with_sim_environment<T: SimulatedTradingMarketDataProvider + Send + 'static>(
        self,
        md_provider: T,
        default_latency: Option<Latency>,
        sim_broker_configs: HashMap<Exchange, SimBrokerConfig>,
        run_messaging: bool,
    ) -> Result<EngineExecutionInfo, EngineError> {
        let (actions_context, message_provider) =
            self.create_actions_context_with_default_message_provider(run_messaging);

        let mut sim_env = SimulatedEnvironment::new(md_provider, default_latency);

        for (exchange, gw_receiver) in actions_context.exchange_requests_receivers() {
            let conf = match sim_broker_configs.get(&exchange) {
                Some(val) => val.clone(),
                None => SimBrokerConfig::default(),
            };

            let sim_broker = SimBroker::new(exchange.clone(), gw_receiver, conf);
            if let Err(err) = sim_env.add_broker(sim_broker) {
                panic!("{:?}", err)
            };
        }

        let threads = self.start_threads(
            sim_env,
            actions_context,
            message_provider,
            run_messaging,
            Some(true),
        )?;

        Ok(EngineExecutionInfo {
            threads,
            exchange_requests_receivers: Default::default(),
        })
    }
}
