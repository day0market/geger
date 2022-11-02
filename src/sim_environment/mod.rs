use crate::common::events::Event;
use crate::common::market_data::MarketDataEvent;

use crate::common::types::{Exchange, Timestamp};
use crate::core::core::EventProvider;
use log::warn;
use std::collections::HashMap;

#[derive(Debug)]
pub enum SimTradingError {
    BrokerAlreadyExists,
}

type Result<T> = std::result::Result<T, SimTradingError>;

pub trait SimulatedTradingMarketDataProvider {
    fn next_event(&mut self) -> Option<MarketDataEvent>;
}

pub trait SimulatedBroker {
    fn exchange(&self) -> Exchange;
    fn on_new_timestamp(&mut self, ts: Timestamp) -> Vec<Event>;
    fn on_new_market_data(&mut self, md: &MarketDataEvent) -> Vec<Event>;
    fn estimate_market_data_timestamp(&self, md: &MarketDataEvent) -> Timestamp;
}

pub struct SimulatedEnvironment<T: SimulatedTradingMarketDataProvider, B: SimulatedBroker> {
    md_provider: T,
    brokers: HashMap<Exchange, B>,
    pending_md_event: Option<MarketDataEvent>,
    broker_events_buffer: Vec<Event>,
    last_ts: u64,
    default_latency: u64,
    no_more_md: bool,
}

impl<T: SimulatedTradingMarketDataProvider, B: SimulatedBroker> SimulatedEnvironment<T, B> {
    pub fn new(md_provider: T, default_latency: Option<u64>) -> Self {
        let brokers = HashMap::new();
        Self {
            md_provider,
            pending_md_event: None,
            last_ts: 0,
            brokers,
            broker_events_buffer: vec![],
            no_more_md: false,
            default_latency: default_latency.unwrap_or(0),
        }
    }

    pub fn add_broker(&mut self, broker: B) -> Result<()> {
        let exchange = broker.exchange();
        if self.brokers.contains_key(&exchange) {
            return Err(SimTradingError::BrokerAlreadyExists);
        };
        self.brokers.insert(exchange, broker);
        Ok(())
    }

    fn update_pending_md(&mut self) {
        if self.no_more_md || self.pending_md_event.is_some() {
            return;
        }

        self.pending_md_event = self.md_provider.next_event();
        if self.pending_md_event.is_none() {
            self.no_more_md = true
        }
    }

    fn feed_market_data_event_to_brokers(&mut self, pending_md_event: MarketDataEvent) {
        let pending_md_ts = pending_md_event.exchange_timestamp();
        for (_, broker) in self.brokers.iter_mut() {
            let new_events = if broker.exchange() == pending_md_event.exchange() {
                broker.on_new_market_data(&pending_md_event)
            } else {
                broker.on_new_timestamp(pending_md_ts)
            };
            self.broker_events_buffer.extend(new_events);
            self.broker_events_buffer
                .sort_by(|a, b| a.timestamp().cmp(&b.timestamp()));
        }
    }
}

impl<T: SimulatedTradingMarketDataProvider, B: SimulatedBroker> EventProvider
    for SimulatedEnvironment<T, B>
{
    fn next_event(&mut self) -> Option<Event> {
        loop {
            self.update_pending_md();

            if self.broker_events_buffer.len() == 0 {
                if self.no_more_md {
                    return None;
                }
                let pending_md_event = self.pending_md_event.take().unwrap();
                self.feed_market_data_event_to_brokers(pending_md_event);
                continue;
            }

            if self.no_more_md {
                if self.broker_events_buffer.len() > 0 {
                    let event = self.broker_events_buffer.remove(0); // TODO alex optimize
                    return Some(event);
                }
                return None;
            }
            let pending_md_event = self.pending_md_event.as_ref().unwrap();
            let expected_md_event_ts = match self.brokers.get(pending_md_event.exchange().as_str())
            {
                Some(broker) => broker.estimate_market_data_timestamp(pending_md_event),
                None => {
                    warn!(
                        "broker not found for md event exchange: {}",
                        pending_md_event.exchange()
                    );
                    pending_md_event.exchange_timestamp() + self.default_latency
                }
            };

            let earliest_broker_event = &self.broker_events_buffer[0];
            if earliest_broker_event.timestamp() > expected_md_event_ts {
                let pending_md_event = self.pending_md_event.take().unwrap();
                self.feed_market_data_event_to_brokers(pending_md_event);
            } else {
                let event = self.broker_events_buffer.remove(0); // TODO alex optimize
                return Some(event);
            }
        }
    }
}
