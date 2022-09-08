use crate::common::events::{Event, MarketDataEvent};
use crate::common::uds::UDSMessage;
use crate::core::core::{CancelOrderRequest, EventProvider, GatewayRequest, NewOrderRequest};
use crate::sim_broker::broker_events::SimBrokerEvent;
use crate::sim_broker::broker_events::{PendingCancelEvent, PendingOrderEvent};
use crossbeam_channel::Receiver;
use std::collections::HashMap;

pub trait SimulatedBrokerMarketDataProvider {
    fn next_event(&mut self) -> Option<MarketDataEvent>;
}

struct ExchangeOrder {}

struct ClosedExchangeOrder {}

struct ExchangeOrderStore {
    open_orders: HashMap<u64, ExchangeOrder>,
    pending_orders: HashMap<u64, ExchangeOrder>,
    done_orders: HashMap<u64, ClosedExchangeOrder>,
}

impl ExchangeOrderStore {
    fn new() -> Self {
        Self {
            open_orders: HashMap::new(),
            pending_orders: HashMap::new(),
            done_orders: HashMap::new(),
        }
    }

    fn on_new_broker_event(&mut self, event: &SimBrokerEvent) -> Option<UDSMessage> {
        None // TODO
    }
}

pub struct SimulatedBroker<T: SimulatedBrokerMarketDataProvider> {
    md_provider: T,
    gateway_request_receiver: Receiver<GatewayRequest>,
    pending_md_event: Option<MarketDataEvent>,
    pending_order_events: Vec<SimBrokerEvent>,
    last_exchange_order_id: u64,
    last_ts: u64,
    wire_latency: HashMap<String, u64>, // Exchange -> latency
    confirmation_latency: u64,
    pending_order_events_next_index: usize,
    order_stores: HashMap<String, ExchangeOrderStore>,
}

impl<T: SimulatedBrokerMarketDataProvider> SimulatedBroker<T> {
    pub fn new(
        md_provider: T,
        gateway_request_receiver: Receiver<GatewayRequest>,
        wire_latency: HashMap<String, u64>,
    ) -> Self {
        let order_stores = HashMap::new();
        Self {
            md_provider,
            gateway_request_receiver,
            pending_md_event: None,
            pending_order_events: vec![],
            last_exchange_order_id: 0,
            last_ts: 0,
            wire_latency,
            confirmation_latency: 0,
            pending_order_events_next_index: 0,
            order_stores,
        }
    }

    fn process_gateway_request(&mut self, gateway_request: GatewayRequest) {
        match gateway_request {
            GatewayRequest::NewOrder(request) => self.process_new_order(request),
            GatewayRequest::CancelOrder(request) => self.process_cancel_request(request),
        }
    }

    fn process_new_order(&mut self, request: NewOrderRequest) {
        let wire_latency = self.wire_latency.get(&request.exchange).unwrap_or(&0);
        let receive_timestamp = self.last_ts + *wire_latency;
        let confirmation_timestamp = receive_timestamp + self.confirmation_latency;
        let exchange_order_id = self.last_exchange_order_id;
        self.last_exchange_order_id += 1;
        let NewOrderRequest {
            client_order_id,
            exchange,
            r#type,
            time_in_force,
            price,
            trigger_price,
            symbol,
            quantity,
        } = request;

        let broker_event = PendingOrderEvent {
            receive_timestamp,
            confirmation_timestamp,
            exchange_order_id,
            client_order_id,
            exchange,
            r#type,
            time_in_force,
            price,
            trigger_price,
            symbol,
            quantity,
        };

        self.pending_order_events
            .push(SimBrokerEvent::PendingOrderEvent(broker_event))
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

        let broker_event = PendingCancelEvent {
            receive_timestamp,
            confirmation_timestamp,
            client_order_id,
            exchange_order_id,
            exchange,
            symbol,
        };

        self.pending_order_events
            .push(SimBrokerEvent::PendingCancelEvent(broker_event))
    }

    fn update_pending_md(&mut self) {
        if self.pending_md_event.is_some() {
            return;
        }

        self.pending_md_event = self.md_provider.next_event()
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
    }
}

impl<T: SimulatedBrokerMarketDataProvider> EventProvider for SimulatedBroker<T> {
    fn next_event(&mut self) -> Option<Event> {
        while let Ok(request) = self.gateway_request_receiver.try_recv() {
            self.process_gateway_request(request)
        }

        self.update_pending_md();

        match self
            .pending_order_events
            .get(self.pending_order_events_next_index)
        {
            Some(order_event) => match &self.pending_md_event {
                Some(val) => {
                    if val.get_timestamp() < order_event.get_timestamp() {
                        let md_event = self.pending_md_event.take()?;
                        Some(Event::MarketDataEvent(md_event))
                    } else {
                        self.process_pending_order_event()
                    }
                }
                None => self.process_pending_order_event(),
            },
            None => {
                let md_event = self.pending_md_event.take()?;
                Some(Event::MarketDataEvent(md_event))
            }
        }
    }
}
