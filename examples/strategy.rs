use crossbeam_channel::unbounded;
use geger::common::log::setup_log;
use geger::core::core::{Actor, Core};
use geger::core::events::Event;
use geger::core::gateway_router::{CancelOrderRequest, GatewayRouter, NewOrderRequest};
use geger::core::market_data::{MarketDataEvent, Quote};
use geger::core::types::{Exchange, OrderStatus, OrderType, Side, Symbol, TimeInForce, Timestamp};
use geger::sim::broker::SimBroker;
use geger::sim::environment::{SimulatedEnvironment, SimulatedTradingMarketDataProvider};
use log::{debug, error, LevelFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

const SIM_BROKER_EXCHANGE: &str = "test_exchange";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuoteDef {
    #[serde(rename(deserialize = "s"))]
    pub symbol: Symbol,

    #[serde(default)]
    pub exchange: Exchange,
    #[serde(rename(deserialize = "b"))]
    pub bid: f64,
    #[serde(rename(deserialize = "a"))]
    pub ask: f64,
    #[serde(rename(deserialize = "bs"))]
    pub bid_size: Option<f64>,
    #[serde(rename(deserialize = "as"))]
    pub ask_size: Option<f64>,
    #[serde(rename(deserialize = "e"))]
    pub exchange_timestamp: f64,
    #[serde(rename(deserialize = "r"))]
    pub received_timestamp: f64,
}

impl From<QuoteDef> for Quote {
    fn from(def: QuoteDef) -> Quote {
        let QuoteDef {
            symbol,
            exchange,
            bid,
            ask,
            bid_size,
            ask_size,
            exchange_timestamp,
            received_timestamp,
        } = def;
        Quote {
            event_id: None,
            symbol,
            exchange: SIM_BROKER_EXCHANGE.to_string(),
            bid,
            ask,
            bid_size,
            ask_size,
            exchange_timestamp: (exchange_timestamp * 1_000.0) as u64,
            received_timestamp: (received_timestamp * 1_000.0) as u64,
        }
    }
}

struct FileMarketDataProvider {
    files: Vec<String>,
    events_buffer: Vec<QuoteDef>,
    directory: String,
    idx: usize,
    file_idx: usize,
    last_ts: u64,
}

impl FileMarketDataProvider {
    fn new(directory: &str) -> Self {
        let mut files = vec![];
        let paths = fs::read_dir(directory).unwrap();
        for path in paths {
            let dir_entry = path.as_ref().unwrap();
            if dir_entry.file_type().unwrap().is_file() {
                files.push(dir_entry.file_name().into_string().unwrap());
            }
        }

        files.sort();

        let events_buffer = vec![];

        Self {
            files,
            events_buffer,
            directory: directory.to_string(),
            idx: 0,
            file_idx: 0,
            last_ts: 0,
        }
    }

    fn read_events_to_buffer(&mut self) -> Result<(), String> {
        self.events_buffer = vec![];
        self.idx = 0;

        if self.files.len() == self.file_idx + 1 {
            return Err("no market data event".to_string());
        }

        let file_name = &self.files[self.file_idx];
        let file_path = format!("{}/{}", &self.directory, file_name);
        let contents = fs::read(file_path.clone()).unwrap();
        println!("content {} loaded", &file_path);
        self.file_idx += 1;
        let quotes = match rmp_serde::from_read_ref::<_, Vec<QuoteDef>>(&contents) {
            Ok(v) => v,
            Err(e) => {
                let err_msg = format!("failed deserialize quote {:?}", e);
                error!("{}", &err_msg);
                return Err(err_msg);
            }
        };

        self.events_buffer = quotes;
        Ok(())
    }
}

impl SimulatedTradingMarketDataProvider for FileMarketDataProvider {
    fn next_event(&mut self) -> Option<MarketDataEvent> {
        let quote = match self.events_buffer.get(self.idx) {
            Some(val) => val,
            None => {
                if let Err(err) = self.read_events_to_buffer() {
                    debug!("failed to read events: {}", err);
                    return None;
                }
                match self.events_buffer.get(self.idx) {
                    Some(val) => val,
                    None => return None,
                }
            }
        };

        let event = MarketDataEvent::NewQuote((*quote).clone().into());
        if event.exchange_timestamp() < self.last_ts {
            panic!("incorrect sequence of events")
        };

        self.idx += 1;

        self.last_ts = event.exchange_timestamp();
        Some(event)
    }
}

struct SampleStrategy {
    has_open_order: bool,
    last_ts: Timestamp,
    seen_events: Vec<Event>,
}

impl SampleStrategy {
    fn new() -> Self {
        Self {
            has_open_order: false,
            last_ts: 0,
            seen_events: vec![],
        }
    }

    fn on_quote(&mut self, quote: &Quote, gw_router: &mut GatewayRouter) {
        if !self.has_open_order {
            let request = NewOrderRequest {
                request_id: "".to_string(),
                client_order_id: "1".to_string(),
                exchange: SIM_BROKER_EXCHANGE.to_string(),
                r#type: OrderType::LIMIT,
                time_in_force: TimeInForce::GTC,
                price: Some(quote.ask),
                trigger_price: None,
                symbol: quote.symbol.clone(),
                quantity: 1.0,
                side: Side::BUY,
                creation_ts: quote.received_timestamp,
            };
            debug!("new order request: {:?}", &request);
            if let Err(err) = gw_router.send_order(request) {
                panic!("{:?}", err)
            };
            self.has_open_order = true
        }
    }
}

impl Actor for SampleStrategy {
    fn on_event(&mut self, event: &Event, gw_router: &mut GatewayRouter) {
        if event.timestamp() < self.last_ts {
            panic!(
                "timestamp sequence is broken. event: {:#?} seen_events: {:#?}",
                event, self.seen_events
            )
        }

        self.seen_events.push((*event).clone());
        self.last_ts = event.timestamp();

        match event {
            Event::NewQuote(quote) => self.on_quote(quote, gw_router),
            Event::UDSOrderUpdate(msg) => match msg.order_status {
                OrderStatus::NEW => {
                    let request = CancelOrderRequest {
                        request_id: "".to_string(),
                        client_order_id: msg.client_order_id.as_ref().unwrap().clone(),
                        exchange_order_id: msg.exchange_order_id.as_ref().unwrap().clone(),
                        exchange: msg.exchange.clone(),
                        symbol: msg.symbol.clone(),
                        creation_ts: msg.exchange_timestamp,
                    };
                    debug!("new cancel request: {:?}", &request);
                    gw_router.cancel_order(request).unwrap();
                }
                OrderStatus::CANCELED => {
                    self.has_open_order = false;
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn main() {
    if let Err(err) = setup_log(Some(LevelFilter::Debug), None) {
        panic!("{:?}", err)
    }
    let md_provider = FileMarketDataProvider::new("data/examples/strategy");
    let strategy = SampleStrategy::new();
    let (gw_sender, gw_receiver) = unbounded();

    let sim_broker_name: Exchange = SIM_BROKER_EXCHANGE.to_string();
    let sim_broker = SimBroker::new(
        sim_broker_name.clone(),
        gw_receiver,
        true,
        Some(100),
        Some(2),
    );
    let mut sim_trading = SimulatedEnvironment::new(md_provider, None);
    if let Err(err) = sim_trading.add_broker(sim_broker) {
        panic!("{:?}", err)
    };
    let mut gw_senders = HashMap::new();
    gw_senders.insert(sim_broker_name, gw_sender);
    let mut core = Core::new(sim_trading, strategy, gw_senders);

    core.run()
}
