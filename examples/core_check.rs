extern crate core;

use crossbeam_channel::unbounded;
use geger::common::events::Event;
use geger::common::market_data::{MarketDataEvent, Quote};
use geger::common::types::{Exchange, OrderType, Side, TimeInForce};
use geger::core::core::{Actor, Core, GatewayRouter, NewOrderRequest};
use geger::sim_broker::broker::SimBroker;
use geger::sim_environment::{SimulatedEnvironment, SimulatedTradingMarketDataProvider};
use log::error;
use std::collections::HashMap;
use std::fs;

const SIM_BROKER_NAME: &str = "test_broket";

struct FileMarketDataProvider {
    files: Vec<String>,
    events_buffer: Vec<Quote>,
    directory: String,
    idx: usize,
    file_idx: usize,
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

        let events_buffer = vec![];

        Self {
            files,
            events_buffer,
            directory: directory.to_string(),
            idx: 0,
            file_idx: 0,
        }
    }

    fn read_events_to_buffer(&mut self) -> Result<(), String> {
        println!("read event to buffer");
        if self.events_buffer.len() < self.idx + 1 && self.events_buffer.len() != 0 {
            self.idx += 1;
            return Ok(());
        }

        if self.files.len() == self.file_idx + 1 {
            return Err("no market data event".to_string());
        }

        let file_name = &self.files[self.file_idx];
        let file_path = format!("{}/{}", &self.directory, file_name);
        let contents = fs::read(file_path).unwrap();
        println!("content readed");
        self.file_idx += 1;
        let quotes = match rmp_serde::from_read_ref::<_, Vec<geger::common::market_data::Quote>>(
            &contents,
        ) {
            Ok(v) => v,
            Err(e) => {
                let err_msg = format!("failed deserialize quote {:?}", e);
                error!("{}", &err_msg);
                return Err(err_msg);
            }
        };

        self.events_buffer = quotes;
        self.idx = 0;
        Ok(())
    }
}

impl SimulatedTradingMarketDataProvider for FileMarketDataProvider {
    fn next_event(&mut self) -> Option<MarketDataEvent> {
        if let Err(err) = self.read_events_to_buffer() {
            println!("failed to read events: {}", err);
            return None;
        }
        let quote = &self.events_buffer[self.idx];
        Some(MarketDataEvent::NewQuote((*quote).clone()))
    }
}

struct SampleStrategy {
    has_open_order: bool,
}

impl SampleStrategy {
    fn new() -> Self {
        Self {
            has_open_order: false,
        }
    }

    fn on_quote(&mut self, quote: &Quote, gw_router: &mut GatewayRouter) {
        if !self.has_open_order {
            let request = NewOrderRequest {
                client_order_id: "1".to_string(),
                exchange: SIM_BROKER_NAME.to_string(),
                r#type: OrderType::LIMIT,
                time_in_force: TimeInForce::GTC,
                price: Some(quote.ask),
                trigger_price: None,
                symbol: quote.symbol.clone(),
                quantity: 1.0,
                side: Side::BUY,
            };
            if let Err(err) = gw_router.send_order(request) {
                panic!("{:?}", err)
            };
            self.has_open_order = true
        }
    }
}

impl Actor for SampleStrategy {
    fn on_event(&mut self, event: &Event, gw_router: &mut GatewayRouter) {
        match event {
            Event::NewQuote(quote) => self.on_quote(quote, gw_router),
            Event::UDSOrderUpdate(msg) => {
                println!("{:?}", &msg);
                panic!("uds arrived");
            }
            _ => {}
        }
    }
}

fn main() {
    let md_provider =
        FileMarketDataProvider::new("/Users/alex/Desktop/my_remote/all_book_tickers_msg");
    let strategy = SampleStrategy::new();
    let (gw_sender, gw_receiver) = unbounded();

    let sim_broker_name: Exchange = SIM_BROKER_NAME.to_string();
    let sim_broker = SimBroker::new(sim_broker_name.clone(), gw_receiver);
    let mut sim_trading = SimulatedEnvironment::new(md_provider);
    if let Err(err) = sim_trading.add_broker(sim_broker) {
        panic!("{:?}", err)
    };
    let mut gw_senders = HashMap::new();
    gw_senders.insert(sim_broker_name, gw_sender);
    let mut core = Core::new(sim_trading, strategy, gw_senders);

    core.run()
}
