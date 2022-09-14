use crate::common::events::{Event, MarketDataEvent};
use crate::common::types::Exchange;
use crate::common::uds::OrderUpdate;
use crate::common::uds::{ExecutionType, OrderStatus, UDSMessage};
use crate::core::core::{CancelOrderRequest, EventProvider, ExchangeRequest, NewOrderRequest};
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
    fn get_name(&self) -> String;
    fn on_new_timestamp(&mut self, ts: u64) -> Vec<Event>;
    fn on_new_market_data(&mut self, md: &MarketDataEvent) -> Vec<Event>;
}

pub struct SimulatedEnvironment<T: SimulatedTradingMarketDataProvider, B: SimulatedBroker> {
    md_provider: T,
    brokers: HashMap<Exchange, B>,
    pending_md_event: Option<MarketDataEvent>,
    broker_events_buffer: Vec<Event>,
    last_ts: u64,
}

impl<T: SimulatedTradingMarketDataProvider, B: SimulatedBroker> SimulatedEnvironment<T, B> {
    pub fn new(md_provider: T) -> Self {
        let brokers = HashMap::new();
        Self {
            md_provider,
            pending_md_event: None,
            last_ts: 0,
            brokers,
            broker_events_buffer: vec![],
        }
    }

    pub fn add_broker(&mut self, broker: B) -> Result<()> {
        let name = broker.get_name();
        if self.brokers.contains_key(&name) {
            return Err(SimTradingError::BrokerAlreadyExists);
        };
        self.brokers.insert(name, broker);
        Ok(())
    }

    fn update_pending_md(&mut self) {
        if self.pending_md_event.is_some() {
            return;
        }

        self.pending_md_event = self.md_provider.next_event()
    }
}

impl<T: SimulatedTradingMarketDataProvider, B: SimulatedBroker> EventProvider
    for SimulatedEnvironment<T, B>
{
    fn next_event(&mut self) -> Option<Event> {
        if self.broker_events_buffer.len() > 0 {
            let event = self.broker_events_buffer.remove(0); // TODO alex optimize

            return Some(event);
        }

        self.update_pending_md();
        if self.pending_md_event.is_none() {
            return None;
        }

        let pending_md_event = self.pending_md_event.as_ref().unwrap();
        let pending_md_ts = pending_md_event.get_timestamp();

        let mut collected_broker_events = vec![];

        for (_, broker) in self.brokers.iter_mut() {
            let new_events = if broker.get_name() == pending_md_event.get_exchange() {
                broker.on_new_market_data(pending_md_event)
            } else {
                broker.on_new_timestamp(pending_md_ts)
            };
            collected_broker_events.extend(new_events);
        }

        if collected_broker_events.len() > 0 {
            collected_broker_events.sort_by(|a, b| a.get_timestamp().cmp(&b.get_timestamp()));
            let event = collected_broker_events.remove(0);
            self.broker_events_buffer = collected_broker_events;
            Some(event)
        } else {
            let md_event = self.pending_md_event.take().unwrap();
            Some(Event::MarketDataEvent(md_event))
        }
    }
}
