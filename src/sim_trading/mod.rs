use crate::common::events::{Event, MarketDataEvent};
use crate::common::types::Exchange;
use crate::common::uds::OrderUpdate;
use crate::common::uds::{ExecutionType, OrderStatus, UDSMessage};
use crate::core::core::{CancelOrderRequest, EventProvider, ExchangeRequest, NewOrderRequest};
use crate::sim_broker::broker_events::BrokerOrder;
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
    fn on_new_timestamp(&mut self, ts: u64) -> Option<Vec<Event>>;
    fn on_new_market_data(&mut self, md: &MarketDataEvent) -> Option<Vec<Event>>;
}

pub struct SimulatedTrading<T: SimulatedTradingMarketDataProvider, B: SimulatedBroker> {
    md_provider: T,
    brokers: HashMap<Exchange, B>,
    pending_md_event: Option<MarketDataEvent>,
    broker_events_buffer: Vec<Event>,
    last_ts: u64,
}

impl<T: SimulatedTradingMarketDataProvider, B: SimulatedBroker> SimulatedTrading<T, B> {
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

    /*fn process_gateway_request(&mut self, gateway_request: ExchangeRequest) {
        match gateway_request {
            ExchangeRequest::NewOrder(request) => self.process_new_order(request),
            ExchangeRequest::CancelOrder(request) => self.process_cancel_request(request),
        }
    }

    fn process_new_order(&mut self, request: NewOrderRequest) {
        let wire_latency = self.wire_latency.get(&request.exchange).unwrap_or(&0);
        let receive_timestamp = self.last_ts + *wire_latency;
        let confirmation_timestamp = receive_timestamp + self.confirmation_latency;

        let NewOrderRequest {
            client_order_id,
            exchange,
            r#type,
            time_in_force,
            price,
            trigger_price,
            symbol,
            quantity,
            side,
        } = request;

        let broker_event = NewOrderEvent {
            receive_timestamp,
            confirmation_timestamp,
            order: BrokerOrder {
                exchange_order_id: None,
                client_order_id,
                exchange,
                r#type,
                time_in_force,
                price,
                trigger_price,
                symbol,
                side,
                quantity,
            },
        };

        self.pending_order_events
            .push(SimBrokerEvent::NewOrder(broker_event))
    }

    fn process_cancel_request(&mut self, request: CancelOrderRequest) {
        let wire_latency = self.wire_latency.get(&request.exchange).unwrap_or(&0);
        let receive_timestamp = self.last_ts + *wire_latency;
        let confirmation_timestamp = receive_timestamp + self.confirmation_latency;

        let CancelOrderRequest {
            client_order_id,
            exchange_order_id,
            exchange,
            symbol,
        } = request;

        let broker_event = NewOrderCancelEvent {
            receive_timestamp,
            confirmation_timestamp,
            client_order_id,
            exchange_order_id,
            exchange,
            symbol,
        };

        self.pending_order_events
            .push(SimBrokerEvent::NewOrderCancel(broker_event))
    }



    fn process_pending_order_event(&mut self) -> Option<Event> {
        let order_event = std::mem::replace(
            &mut self.pending_order_events[self.pending_order_events_next_index],
            SimBrokerEvent::None,
        );

        let uds_message = self.update_order_status(&order_event)?;

        self.pending_order_events_next_index += 1;
        Some(Event::UDSMessage(uds_message))
    }

    fn update_order_status(&mut self, order_event: &SimBrokerEvent) -> Option<UDSMessage> {
        let event_exchange = order_event.get_exchange();
        if !self.order_stores.contains_key(&event_exchange) {
            let store = ExchangeOrderStore::new();
            self.order_stores.insert(event_exchange.clone(), store);
        }

        let order_store = self.order_stores.get_mut(&event_exchange).unwrap();
        order_store.on_new_broker_event(order_event)
    }*/

    fn update_pending_md(&mut self) {
        if self.pending_md_event.is_some() {
            return;
        }

        self.pending_md_event = self.md_provider.next_event()
    }
}

impl<T: SimulatedTradingMarketDataProvider, B: SimulatedBroker> EventProvider
    for SimulatedTrading<T, B>
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
            if broker.get_name() == pending_md_event.get_exchange() {
                if let Some(new_events) = broker.on_new_market_data(pending_md_event) {
                    collected_broker_events.extend(new_events);
                }
            } else {
                if let Some(new_events) = broker.on_new_timestamp(pending_md_ts) {
                    collected_broker_events.extend(new_events);
                };
            }
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
