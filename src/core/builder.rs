use crate::core::actions_context::ActionsContext;
use crate::core::event_loop::{Actor, EventLoop, EventProvider};
use crate::core::gateway_router::{ExchangeRequest, GatewayRouter};
use crate::core::message_bus::{CrossbeamMessageSender, Message, MessageSender, SimpleMessage};
use crate::core::types::Exchange;
use crate::sim::broker::SimBroker;
use crate::sim::environment::{SimulatedEnvironment, SimulatedTradingMarketDataProvider};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

pub struct Builder<S: Actor<M, MS>, M: Message, MS: MessageSender<M>> {
    phantom: PhantomData<M>,
    phantom2: PhantomData<MS>,
    actors: Vec<Arc<Mutex<S>>>,
    exchanges: Vec<Exchange>,
}

impl<S: Actor<M, MS>, M: Message, MS: MessageSender<M>> Builder<S, M, MS> {
    pub fn new() -> Self {
        Self {
            phantom: Default::default(),
            phantom2: Default::default(),
            actors: vec![],
            exchanges: vec![],
        }
    }
    pub fn add_actor(&mut self, actor: Arc<Mutex<S>>) {
        self.actors.push(actor);
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

        let actions_context = ActionsContext::new(gateway_router, message_sender);
        let mut event_loop = EventLoop::new(event_provider, self.actors, actions_context);

        event_loop.run()
    }
}

impl<S: Actor<SimpleMessage, CrossbeamMessageSender<SimpleMessage>>>
    Builder<S, SimpleMessage, CrossbeamMessageSender<SimpleMessage>>
{
    pub fn run_with_event_provider_default_messaging<T: EventProvider>(self, event_provider: T) {
        let gateway_router = GatewayRouter::new(self.exchanges.clone());
        let message_sender = CrossbeamMessageSender::new();
        let actions_context = ActionsContext::new(gateway_router, message_sender);
        let mut event_loop = EventLoop::new(event_provider, self.actors, actions_context);
        event_loop.run()
    }

    pub fn run_with_sim_environment<T: SimulatedTradingMarketDataProvider>(
        self,
        md_provider: T,
        default_latency: Option<u64>,
        latencies_config: HashMap<Exchange, (Option<u64>, Option<u64>)>,
    ) {
        let gateway_router = GatewayRouter::new(self.exchanges.clone());
        let mut sim_trading = SimulatedEnvironment::new(md_provider, default_latency);

        for (exch, gw_receiver) in gateway_router.receivers() {
            let (wire_latency, internal_latency) = match latencies_config.get(&exch) {
                Some(val) => val.clone(),
                None => (None, None),
            };
            let sim_broker = SimBroker::new(
                exch.clone(),
                gw_receiver,
                true,
                wire_latency,
                internal_latency,
            ); // todo alex use broker config
            if let Err(err) = sim_trading.add_broker(sim_broker) {
                panic!("{:?}", err)
            };
        }

        let message_sender = CrossbeamMessageSender::new();
        let actions_context = ActionsContext::new(gateway_router, message_sender);
        let mut event_loop = EventLoop::new(sim_trading, self.actors, actions_context);
        event_loop.run()
    }
}