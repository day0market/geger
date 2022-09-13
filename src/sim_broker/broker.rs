use crate::common::events::{Event, MarketDataEvent};
use crate::common::uds::OrderUpdate;
use crate::common::uds::{ExecutionType, OrderStatus, UDSMessage};
use crate::core::core::{CancelOrderRequest, EventProvider, ExchangeRequest, NewOrderRequest};
use crate::sim_broker::broker_events::BrokerOrder;
use crate::sim_trading::SimulatedBroker;
use rmp::Marker::{False, True};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;

struct SimBrokerExchangeRequest {
    ack_timestamp: u64,
    request_id: u64,
    exchange_request: ExchangeRequest,
}

pub struct SimBroker {
    name: String,
    last_exchange_order_id: u64,
    last_request_id: u64,
    last_generated_event_id: u64,
    last_ts: u64,
    open_orders: HashMap<u64, BrokerOrder>,
    done_orders: HashMap<u64, BrokerOrder>,
    order_id_mapping: HashMap<String, u64>,
    pending_requests: HashMap<u64, SimBrokerExchangeRequest>,
    incoming_request_receiver: mpsc::Receiver<ExchangeRequest>,
    generated_events: HashMap<u64, Event>,
    wire_latency: u64,
}

impl SimBroker {
    pub fn new(name: String, incoming_request_receiver: mpsc::Receiver<ExchangeRequest>) -> Self {
        Self {
            name,
            last_ts: 0,
            last_exchange_order_id: 0,
            last_request_id: 0,
            last_generated_event_id: 0,
            open_orders: HashMap::new(),
            done_orders: HashMap::new(),
            order_id_mapping: HashMap::new(),
            pending_requests: HashMap::new(),
            generated_events: HashMap::new(),
            incoming_request_receiver,
            wire_latency: 0,
        }
    }

    /*fn on_new_broker_event(&mut self, event: &SimBrokerEvent) -> Option<UDSMessage> {
        match event {
            SimBrokerEvent::NewOrder(event) => self.on_new_order_event(event),
            SimBrokerEvent::NewOrderCancel(event) => self.on_cancel_event(event),
            SimBrokerEvent::MarketDataEvent() => todo!(),
            _ => None,
        }
    }

    fn on_new_order_event(&mut self, event: &NewOrderEvent) -> Option<UDSMessage> {
        // order rejected update is default
        let mut order_update = OrderUpdate {
            symbol: event.order.symbol.clone(),
            side: event.order.side.clone(),
            client_order_id: Some(event.order.client_order_id.clone()),
            exchange_order_id: None,
            order_type: Some(event.order.r#type.clone()),
            time_in_force: Some(event.order.time_in_force.clone()),
            original_qty: event.order.quantity,
            original_price: event.order.price,
            average_price: None,
            stop_price: None, // TODO populate when order is stop
            execution_type: ExecutionType::NEW,
            order_status: OrderStatus::CANCELED,
            last_filled_qty: None,
            accumulated_filled_qty: None,
            last_filled_price: None,
            last_trade_time: None,
        };

        if self
            .order_id_mapping
            .contains_key(&event.order.client_order_id)
        {
            return Some(UDSMessage::OrderUpdate(order_update));
        }

        self.last_exchange_order_id += 1;
        order_update.exchange_order_id = Some(self.last_exchange_order_id.to_string());
        order_update.order_status = OrderStatus::NEW;

        self.order_id_mapping.insert(
            event.order.client_order_id.clone(),
            self.last_exchange_order_id,
        );
        self.open_orders
            .insert(self.last_exchange_order_id, event.order.clone());

        Some(UDSMessage::OrderUpdate(order_update))
    }

    fn on_cancel_event(&mut self, event: &NewOrderCancelEvent) -> Option<UDSMessage> {
        todo!()
    }*/

    fn execute_requests_after_ts(&mut self, ts: u64) {
        if self.pending_requests.len() == 0 {
            return;
        }
        let keys: Vec<u64> = self.pending_requests.keys().map(|&k| k).collect();
        for req_id in keys {
            if &self.pending_requests[&req_id].ack_timestamp > &ts {
                continue;
            }

            let wrapped_request = match self.pending_requests.remove(&req_id) {
                Some(val) => val,
                None => unreachable!(),
            };

            match &wrapped_request.exchange_request {
                ExchangeRequest::NewOrder(request) => self.on_new_order_request(request),
                ExchangeRequest::CancelOrder(request) => self.on_cancel_order_requests(request),
            };
        }
    }

    fn update_orders_on_md(&mut self, md: &MarketDataEvent) {
        // Update order state, generate UDS and put them into buffer
    }
    fn on_new_order_request(&self, request: &NewOrderRequest) {
        todo!()
    }

    fn on_cancel_order_requests(&self, request: &CancelOrderRequest) {
        todo!()
    }

    fn get_generated_events_before_ts(&mut self, ts: u64) -> Vec<Event> {
        let mut events = vec![];
        if self.generated_events.len() == 0 {
            return events;
        }

        let event_ids: Vec<u64> = self.generated_events.keys().map(|&k| k).collect();
        for event_id in event_ids {
            if &self.generated_events[&event_id].get_timestamp() > &ts {
                continue;
            }

            let event = match self.generated_events.remove(&event_id) {
                Some(event) => event,
                None => unreachable!(),
            };

            events.push(event);
        }

        events
    }

    fn process_requests_on_new_ts(&mut self, ts: u64) {
        while let Ok(exchange_request) = self.incoming_request_receiver.try_recv() {
            self.last_request_id += 1;
            let wrapped_request = SimBrokerExchangeRequest {
                ack_timestamp: self.last_ts + self.wire_latency,
                request_id: self.last_request_id,
                exchange_request,
            };

            self.pending_requests
                .insert(self.last_request_id, wrapped_request);
        }
        self.execute_requests_after_ts(ts);
    }
}

impl SimulatedBroker for SimBroker {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn on_new_timestamp(&mut self, ts: u64) -> Vec<Event> {
        self.process_requests_on_new_ts(ts);
        let events = self.get_generated_events_before_ts(ts);
        self.last_ts = ts;
        events
    }

    fn on_new_market_data(&mut self, md: &MarketDataEvent) -> Vec<Event> {
        let md_ts = md.get_timestamp();
        self.process_requests_on_new_ts(md_ts);
        self.update_orders_on_md(md);
        let events = self.get_generated_events_before_ts(md_ts);
        self.last_ts = ts;
        events
    }
}
