use crate::core::events::Event;
use crate::core::market_data::MarketDataEvent;

use crate::core::core::EventProvider;
use crate::core::types::{Exchange, Timestamp};
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
    fn wire_latency(&self) -> Timestamp;
}

pub struct SimulatedEnvironment<T: SimulatedTradingMarketDataProvider, B: SimulatedBroker> {
    md_provider: T,
    brokers: HashMap<Exchange, B>,
    pending_md_event: Option<MarketDataEvent>,
    broker_events_buffer: Vec<Event>,
    default_latency: u64,
    no_more_md: bool,
    md_event_buffer: Vec<MarketDataEvent>,
    max_md_wire_latency: Timestamp,
}

impl<T: SimulatedTradingMarketDataProvider, B: SimulatedBroker> SimulatedEnvironment<T, B> {
    pub fn new(md_provider: T, default_latency: Option<u64>) -> Self {
        let brokers = HashMap::new();
        Self {
            md_provider,
            pending_md_event: None,
            brokers,
            broker_events_buffer: vec![],
            no_more_md: false,
            default_latency: default_latency.unwrap_or(0),
            md_event_buffer: vec![],
            max_md_wire_latency: 0,
        }
    }

    pub fn add_broker(&mut self, broker: B) -> Result<()> {
        let exchange = broker.exchange();
        if self.brokers.contains_key(&exchange) {
            return Err(SimTradingError::BrokerAlreadyExists);
        };
        let broker_wire_latency = broker.wire_latency();
        if broker_wire_latency > self.max_md_wire_latency {
            self.max_md_wire_latency = broker_wire_latency;
        }
        self.brokers.insert(exchange, broker);
        Ok(())
    }

    fn md_event_expected_received_ts(&self, md: &MarketDataEvent) -> Timestamp {
        match self.brokers.get(md.exchange().as_str()) {
            Some(broker) => broker.estimate_market_data_timestamp(md),
            None => {
                warn!("broker not found for md event exchange: {}", md.exchange());
                md.exchange_timestamp() + self.default_latency
            }
        }
    }

    fn update_pending_md(&mut self) {
        if self.pending_md_event.is_some() {
            return;
        }

        if self.md_event_buffer.len() == 0 {
            if self.no_more_md {
                return;
            }
            let event = self.md_provider.next_event();
            if event.is_none() {
                self.no_more_md = true;
                return;
            }
            let event = event.unwrap();
            self.md_event_buffer.push(event);
        }

        // expect that earliest event has max wire latency and min latency is 0
        // read all events with exchange ts <= earliest event + max latency
        // we expect that md is sorted by exchange ts, so first event in buffer should always have min exchange ts
        let max_exchange_ts_to_read =
            &self.md_event_buffer[0].exchange_timestamp() + self.max_md_wire_latency;

        loop {
            let event = self.md_provider.next_event();
            if event.is_none() {
                break;
            }
            let event = event.unwrap();
            let event_exchange_ts = event.exchange_timestamp();
            self.md_event_buffer.push(event);
            if event_exchange_ts > max_exchange_ts_to_read {
                break;
            }
        }

        let mut earliest_received_event_idx = 0;
        let mut earliest_receive_ts = self.md_event_expected_received_ts(&self.md_event_buffer[0]);
        for i in 0..self.md_event_buffer.len() {
            let expected_ts = self.md_event_expected_received_ts(&self.md_event_buffer[i]);
            if expected_ts < earliest_receive_ts {
                earliest_receive_ts = expected_ts;
                earliest_received_event_idx = i;
            }
        }

        let pending_event = self.md_event_buffer.remove(earliest_received_event_idx);
        self.pending_md_event = Some(pending_event);
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
            let expected_md_event_ts = self.md_event_expected_received_ts(pending_md_event);

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
