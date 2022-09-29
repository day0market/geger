use crossbeam_channel::unbounded;
use geger::common::events::{Event, NewOrderAccepted, OrderUpdate};
use geger::common::market_data::{MarketDataEvent, Quote};
use geger::common::types::{ClientOrderId, OrderStatus, OrderType, Side, TimeInForce, Timestamp};
use geger::core::core::{Actor, Core};
use geger::core::gateway_router::{CancelOrderRequest, GatewayRouter, NewOrderRequest};
use geger::sim_broker::broker::SimBroker;
use geger::sim_environment::{SimulatedEnvironment, SimulatedTradingMarketDataProvider};
use json_comments::StripComments;
use log::info;
use serde_json;
use std::collections::HashMap;
use std::{env, fs};

const TRADE_SYMBOL: &str = "test_ok";
const TRADE_EXCHANGE: &str = "test_exchange_ok";
const NON_TRADE_EXCHANGE: &str = "test_exchange_not_ok";
const FIXTURE_PATH: &str = "data/test/test_sim_environment_md.json";

struct TestEventSequenceMDProvider {
    md_events: Vec<MarketDataEvent>,
    event_idx: usize,
}

impl TestEventSequenceMDProvider {
    fn new() -> Self {
        let mut md_events = Self::get_md_events_from_fixture();
        Self {
            md_events,
            event_idx: 0,
        }
    }

    fn get_md_events_from_fixture() -> Vec<MarketDataEvent> {
        let cwd = env::current_dir().unwrap();
        let fixture_path = cwd.join(FIXTURE_PATH);
        let data = fs::read_to_string(fixture_path.clone()).unwrap();
        let stripped = StripComments::new(data.as_bytes());
        let mut md_events = serde_json::from_reader(stripped).unwrap();

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
    open_order: Option<ClientOrderId>,
    open_order_ts: Option<Timestamp>,
    collected_events: Vec<Event>,
    max_order_age: u64,
}

impl TestStrategy {
    fn new() -> Self {
        Self {
            last_event_ts: 0,
            last_client_order_id: 1,
            open_order: None,
            open_order_ts: None,
            collected_events: vec![],
            max_order_age: 200,
        }
    }
    fn on_order_accepted(&mut self, event: &NewOrderAccepted) {
        assert_eq!(event.exchange_order_id, event.client_order_id);
        info!("{} order accepted", event.client_order_id)
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
            other => unreachable!(),
        }
    }

    fn on_quote(&mut self, event: &Quote, gw_router: &mut GatewayRouter) {
        if self.open_order.is_some() {
            return;
        }

        if event.exchange.as_str() != TRADE_EXCHANGE || event.symbol.as_str() != TRADE_SYMBOL {
            return;
        }

        let new_order_request = NewOrderRequest {
            client_order_id: self.last_client_order_id.to_string(),
            exchange: TRADE_EXCHANGE.to_string(),
            r#type: OrderType::LIMIT,
            time_in_force: TimeInForce::GTC,
            price: Some(event.ask),
            trigger_price: None,
            symbol: TRADE_SYMBOL.to_string(),
            quantity: 1.0,
            side: Side::BUY,
        };
        self.open_order = Some(self.last_client_order_id.to_string());
        self.open_order_ts = Some(event.received_timestamp);
        self.last_client_order_id += 1;

        gw_router.send_order(new_order_request).unwrap();
    }
}

impl Actor for TestStrategy {
    fn on_event(&mut self, event: &Event, gw_router: &mut GatewayRouter) {
        if self.last_event_ts > event.timestamp() {
            panic!("wrong timestamp sequence")
        }
        self.last_event_ts = event.timestamp();
        match event {
            Event::NewQuote(_) | Event::NewMarketTrade(_) => {
                if let Some(order_ts) = self.open_order_ts {
                    if event.timestamp() - order_ts > self.max_order_age {
                        let clio = self.open_order.as_ref().unwrap().clone();
                        let cancel_request = CancelOrderRequest {
                            client_order_id: clio.clone(),
                            exchange_order_id: clio,
                            exchange: TRADE_EXCHANGE.to_string(),
                            symbol: TRADE_SYMBOL.to_string(),
                        };

                        gw_router.cancel_order(cancel_request).unwrap();
                    }
                };
            }
            _ => {}
        }

        match event {
            Event::NewQuote(q) => self.on_quote(q, gw_router),
            Event::UDSOrderUpdate(u) => self.on_uds(u),
            Event::ResponseNewOrderAccepted(e) => self.on_order_accepted(e),
            _ => {}
        };

        self.collected_events.push(event.clone());
    }
}

#[test]
fn check_event_sequence() {
    let provider = TestEventSequenceMDProvider::new();
    let strategy = TestStrategy::new();
    let mut sim_trading = SimulatedEnvironment::new(provider);

    let (gw_sender_ok, gw_receiver_ok) = unbounded();
    let sim_broker_ok = SimBroker::new(TRADE_EXCHANGE.to_string(), gw_receiver_ok, true);

    let (gw_sender_not_ok, gw_receiver_not_ok) = unbounded();
    let sim_broker_not_ok =
        SimBroker::new(NON_TRADE_EXCHANGE.to_string(), gw_receiver_not_ok, true);

    if let Err(err) = sim_trading.add_broker(sim_broker_ok) {
        panic!("{:?}", err)
    };

    if let Err(err) = sim_trading.add_broker(sim_broker_not_ok) {
        panic!("{:?}", err)
    };

    let mut gw_senders = HashMap::new();
    gw_senders.insert(TRADE_EXCHANGE.to_string(), gw_sender_ok);
    gw_senders.insert(NON_TRADE_EXCHANGE.to_string(), gw_sender_not_ok);

    let mut core = Core::new(sim_trading, strategy, gw_senders);

    core.run();

    let strategy = core.get_strategy();
    let data = serde_json::to_vec(&strategy.collected_events).unwrap();
    fs::write("tests/collected_events.json", data).unwrap();
    println!("{:#?}", strategy.collected_events);
}
