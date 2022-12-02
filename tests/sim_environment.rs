extern crate core;

use geger::core::actions_context::ActionsContext;
use geger::core::event_loop::Actor;
use geger::core::events::{Event, NewOrderAccepted, OrderUpdate};
use geger::core::gateway_router::{CancelOrderRequest, ExchangeRequest, NewOrderRequest};
use geger::core::market_data::{MarketDataEvent, Quote};

use geger::core::engine::Engine;
use geger::core::message_bus::{LoggerMessageHandler, Message, MessageSender};
use geger::core::types::{ClientOrderId, OrderStatus, OrderType, Side, TimeInForce, Timestamp};
use geger::sim::broker::SimBrokerConfig;
use geger::sim::environment::SimulatedTradingMarketDataProvider;
use json_comments::StripComments;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{env, fs};

const TRADE_SYMBOL: &str = "test_ok";
const TRADE_EXCHANGE: &str = "test_exchange_ok";
const NON_TRADE_EXCHANGE: &str = "test_exchange_not_ok";
const SINGLE_SYMBOL_MD: &str = "data/test/sim_environment/md_single_symbol.json";
const MULTIPLE_EXCHANGE_SYMBOL_MD: &str =
    "data/test/sim_environment/md_multiple_symbols_exchanges.json";
const EXPECTED_COLLECTED_EVENTS_PATH: &str = "data/test/sim_environment/expected_events.json";

enum MyActors {
    Strategy(TestStrategy),
}

impl<M: Message, MS: MessageSender<M>> Actor<M, MS> for MyActors {
    fn on_event(&mut self, event: &Event, actions_context: &mut ActionsContext<M, MS>) {
        match self {
            MyActors::Strategy(s) => s.on_event(event, actions_context),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
enum TestStrategyCollectedEvent {
    Event(Event),
    ExchangeRequest(ExchangeRequest),
}

fn get_expected_events_from_fixture() -> Vec<TestStrategyCollectedEvent> {
    let cwd = env::current_dir().unwrap();
    let fixture_path = cwd.join(EXPECTED_COLLECTED_EVENTS_PATH);
    let data = fs::read_to_string(fixture_path.clone()).unwrap();
    let stripped = StripComments::new(data.as_bytes());
    let md_events = serde_json::from_reader(stripped).unwrap();

    md_events
}

struct TestEventSequenceMDProvider {
    md_events: Vec<MarketDataEvent>,
    event_idx: usize,
}

impl TestEventSequenceMDProvider {
    fn new(fixture_path: &str) -> Self {
        let md_events = Self::get_md_events_from_fixture(fixture_path);
        Self {
            md_events,
            event_idx: 0,
        }
    }

    fn get_md_events_from_fixture(fixture_path: &str) -> Vec<MarketDataEvent> {
        let cwd = env::current_dir().unwrap();
        let fixture_path = cwd.join(fixture_path);
        let data = fs::read_to_string(fixture_path.clone()).unwrap();
        let stripped = StripComments::new(data.as_bytes());
        let md_events = serde_json::from_reader(stripped).unwrap();

        md_events
    }
}

impl SimulatedTradingMarketDataProvider for TestEventSequenceMDProvider {
    fn next_event(&mut self) -> Option<MarketDataEvent> {
        match self.md_events.get(self.event_idx) {
            Some(val) => {
                self.event_idx += 1;
                Some((*val).clone())
            }
            None => None,
        }
    }
}

struct TestStrategy {
    last_event_ts: Timestamp,
    last_client_order_id: u64, // should be same as exchange id
    last_request_id: u64,
    open_order: Option<ClientOrderId>,
    open_order_ts: Option<Timestamp>,
    collected_events: Vec<TestStrategyCollectedEvent>,
    max_order_age: u64,
    previous_side: Side,
}

impl TestStrategy {
    fn new() -> Self {
        Self {
            last_event_ts: 0,
            last_client_order_id: 1,
            last_request_id: 0,
            open_order: None,
            open_order_ts: None,
            collected_events: vec![],
            max_order_age: 200,
            previous_side: Side::SELL,
        }
    }
    fn on_order_accepted(&mut self, event: &NewOrderAccepted) {
        assert_eq!(event.exchange_order_id, event.client_order_id);
        info!("{} order accepted", event.client_order_id)
    }

    fn get_request_id(&mut self) -> String {
        self.last_request_id += 1;
        self.last_request_id.to_string()
    }

    fn on_uds(&mut self, event: &OrderUpdate) {
        match &event.order_status {
            OrderStatus::NEW => {
                info!("{:?} order update NEW", event.client_order_id)
            }
            OrderStatus::FILLED => {
                info!("{:?} order update FILLED", event.client_order_id);
                self.open_order = None;
                self.open_order_ts = None;
            }
            OrderStatus::CANCELED => {
                info!("{:?} order update CANCELED", event.client_order_id);
                self.open_order = None;
                self.open_order_ts = None;
            }
            other => unreachable!("{:?}", other),
        }
    }

    fn on_quote<M: Message, MS: MessageSender<M>>(
        &mut self,
        event: &Quote,
        actions_context: &mut ActionsContext<M, MS>,
    ) {
        if self.open_order.is_some() {
            return;
        }

        if event.exchange.as_str() != TRADE_EXCHANGE || event.symbol.as_str() != TRADE_SYMBOL {
            return;
        }

        let (side, price) = match &self.previous_side {
            Side::SELL => (Side::BUY, event.ask),
            Side::BUY => (Side::SELL, event.bid),
        };

        self.previous_side = side.clone();

        let new_order_request = NewOrderRequest {
            request_id: self.get_request_id(),
            client_order_id: self.last_client_order_id.to_string(),
            exchange: TRADE_EXCHANGE.to_string(),
            r#type: OrderType::LIMIT,
            time_in_force: TimeInForce::GTC,
            price: Some(price),
            trigger_price: None,
            symbol: TRADE_SYMBOL.to_string(),
            quantity: 1.0,
            side,
            creation_ts: event.received_timestamp,
        };

        self.open_order = Some(self.last_client_order_id.to_string());
        self.open_order_ts = Some(event.received_timestamp);

        info!(
            "open_order_ts: {} new order request: {:?}",
            event.received_timestamp, &new_order_request
        );
        self.last_client_order_id += 1;
        self.collected_events
            .push(TestStrategyCollectedEvent::ExchangeRequest(
                ExchangeRequest::NewOrder(new_order_request.clone()),
            ));
        actions_context.send_order(new_order_request).unwrap();
    }
}

impl<M: Message, MS: MessageSender<M>> Actor<M, MS> for TestStrategy {
    fn on_event(&mut self, event: &Event, actions_context: &mut ActionsContext<M, MS>) {
        if self.last_event_ts > event.timestamp() {
            panic!("wrong timestamp sequence")
        }
        info!(
            "test strategy received exchange_ts: {} received_ts: {} new event: {:?}",
            event.exchange_timestamp(),
            event.timestamp(),
            event
        );

        self.last_event_ts = event.timestamp();

        // due to map iteration event sequence and event_id may vary.
        // here is some dirty hack to set all event_id to empty and check only event existence in assetion
        let mut collected_event = event.clone();
        match &mut collected_event {
            Event::ResponseNewOrderAccepted(i) => {
                i.event_id = "".to_string();
            }
            Event::ResponseNewOrderRejected(i) => {
                i.event_id = "".to_string();
            }
            Event::ResponseCancelOrderAccepted(i) => {
                i.event_id = "".to_string();
            }
            Event::ResponseCancelOrderRejected(i) => {
                i.event_id = "".to_string();
            }
            Event::UDSOrderUpdate(i) => {
                i.event_id = "".to_string();
            }
            _ => {}
        }
        self.collected_events
            .push(TestStrategyCollectedEvent::Event(collected_event));

        if event.symbol() == TRADE_SYMBOL && event.exchange() == TRADE_EXCHANGE {
            match event {
                Event::NewQuote(_) | Event::NewMarketTrade(_) => {
                    if let Some(order_ts) = self.open_order_ts {
                        let order_age = event.timestamp() - order_ts;
                        info!("order_age: {}", order_age);
                        if order_age > self.max_order_age {
                            let clio = self.open_order.as_ref().unwrap().clone();
                            let cancel_request = CancelOrderRequest {
                                request_id: self.get_request_id(),
                                client_order_id: clio.clone(),
                                exchange_order_id: clio,
                                exchange: TRADE_EXCHANGE.to_string(),
                                symbol: TRADE_SYMBOL.to_string(),
                                creation_ts: event.timestamp(),
                            };

                            info!("hit max order age. cancel request: {:?}", &cancel_request);
                            self.collected_events.push(
                                TestStrategyCollectedEvent::ExchangeRequest(
                                    ExchangeRequest::CancelOrder(cancel_request.clone()),
                                ),
                            );
                            actions_context.cancel_order(cancel_request).unwrap();
                        }
                    };
                }
                _ => {}
            }
        }

        match event {
            Event::NewQuote(q) => self.on_quote(q, actions_context),
            Event::UDSOrderUpdate(u) => self.on_uds(u),
            Event::ResponseNewOrderAccepted(e) => self.on_order_accepted(e),
            _ => {}
        };
    }
}

#[test]
fn check_event_sequence_single_exchange_symbol() {
    let expected_collected_events = get_expected_events_from_fixture();

    let md_provider = TestEventSequenceMDProvider::new(SINGLE_SYMBOL_MD);
    let strategy = MyActors::Strategy(TestStrategy::new());
    let arc_strategy = Arc::new(Mutex::new(strategy));
    let message_handler = Arc::new(Mutex::new(LoggerMessageHandler::default()));

    let mut sim_broker_configs = HashMap::new();
    sim_broker_configs.insert(
        TRADE_EXCHANGE.to_string(),
        SimBrokerConfig::new(true, Some(100), Some(5)),
    );
    sim_broker_configs.insert(
        NON_TRADE_EXCHANGE.to_string(),
        SimBrokerConfig::new(true, Some(50), Some(10)),
    );

    let mut engine = Engine::new();
    engine.add_exchange(TRADE_EXCHANGE.to_string());
    engine.add_exchange(NON_TRADE_EXCHANGE.to_string());
    engine.add_actor(arc_strategy.clone());
    engine.add_message_handler(message_handler);
    let execution_info = engine
        .execute_with_sim_environment(md_provider, None, sim_broker_configs, true)
        .unwrap();

    for th in execution_info.threads {
        th.unwrap().join().unwrap()
    }

    let lock = arc_strategy.lock().unwrap();
    let strategy = match *lock {
        MyActors::Strategy(ref s) => s,
    };
    //let data = serde_json::to_vec(&strategy.collected_events).unwrap();
    //fs::write("tests/collected_events.json", data).unwrap();
    assert_eq!(
        &strategy.collected_events.len(),
        &expected_collected_events.len()
    );

    for i in 0..strategy.collected_events.len() {
        let collected = &strategy.collected_events[i];
        let found = &expected_collected_events.contains(collected);
        assert!(found);
    }
}

#[test]
fn check_event_sequence_multiple_exchanges_symbols() {
    let expected_collected_events = get_expected_events_from_fixture();
    let expected_md_events =
        TestEventSequenceMDProvider::get_md_events_from_fixture(MULTIPLE_EXCHANGE_SYMBOL_MD);

    let md_provider = TestEventSequenceMDProvider::new(MULTIPLE_EXCHANGE_SYMBOL_MD);
    let strategy = MyActors::Strategy(TestStrategy::new());
    let arc_strategy = Arc::new(Mutex::new(strategy));
    let message_handler = Arc::new(Mutex::new(LoggerMessageHandler::default()));

    let mut sim_broker_configs = HashMap::new();
    sim_broker_configs.insert(
        TRADE_EXCHANGE.to_string(),
        SimBrokerConfig::new(true, Some(100), Some(5)),
    );
    sim_broker_configs.insert(
        NON_TRADE_EXCHANGE.to_string(),
        SimBrokerConfig::new(true, Some(50), Some(10)),
    );

    let mut engine = Engine::new();
    engine.add_exchange(TRADE_EXCHANGE.to_string());
    engine.add_exchange(NON_TRADE_EXCHANGE.to_string());
    engine.add_actor(arc_strategy.clone());
    engine.add_message_handler(message_handler);
    let execution_info = engine
        .execute_with_sim_environment(md_provider, None, sim_broker_configs, true)
        .unwrap();

    for th in execution_info.threads {
        th.unwrap().join().unwrap()
    }

    let lock = arc_strategy.lock().unwrap();
    let strategy = match *lock {
        MyActors::Strategy(ref s) => s,
    };
    //let data = serde_json::to_vec(&strategy.collected_events).unwrap();
    //fs::write("tests/collected_events_multiple.json", data).unwrap();

    for i in 0..expected_collected_events.len() {
        let expected = &expected_collected_events[i];
        let found = &strategy.collected_events.contains(expected);

        assert!(found, "expected: {:?}", &expected);
    }

    let mut collected_md_events = vec![];
    for event in &strategy.collected_events {
        match event {
            TestStrategyCollectedEvent::Event(e) => match e {
                Event::NewQuote(q) => {
                    collected_md_events.push(MarketDataEvent::NewQuote(q.clone()));
                }
                Event::NewMarketTrade(t) => {
                    collected_md_events.push(MarketDataEvent::NewMarketTrade(t.clone()));
                }
                _ => {}
            },
            _ => {}
        }
    }

    assert_eq!(collected_md_events.len(), expected_md_events.len())
}
