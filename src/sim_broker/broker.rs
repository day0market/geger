use crate::common::events::{Event, MarketDataEvent};
use crate::common::uds::OrderUpdate;
use crate::common::uds::{ExecutionType, OrderStatus, UDSMessage};
use crate::core::core::{CancelOrderRequest, EventProvider, ExchangeRequest, NewOrderRequest};
use crate::sim_broker::broker_events::BrokerOrder;
use crate::sim_trading::SimulatedBroker;
use std::collections::HashMap;
use std::sync::mpsc;

struct SimBrokerExchangeRequest {
    ack_timestamp: u64,
    exchange_request: ExchangeRequest,
}

pub struct SimBroker {
    name: String,
    last_exchange_order_id: u64,
    last_ts: u64,
    open_orders: HashMap<u64, BrokerOrder>,
    done_orders: HashMap<u64, BrokerOrder>,
    order_id_mapping: HashMap<String, u64>,
    pending_requests: Vec<SimBrokerExchangeRequest>,
    incoming_request_receiver: mpsc::Receiver<ExchangeRequest>,
    wire_latency: u64,
}

impl SimBroker {
    pub fn new(name: String, incoming_request_receiver: mpsc::Receiver<ExchangeRequest>) -> Self {
        Self {
            name,
            last_ts: 0,
            last_exchange_order_id: 0,
            open_orders: HashMap::new(),
            done_orders: HashMap::new(),
            order_id_mapping: HashMap::new(),
            pending_requests: vec![],
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

    fn flush_event_buffer_on_ts(&mut self) -> Option<Vec<Event>> {
        // need to flush all exchange rquest events (gen responses, UDS messages) and flush generated UDS messages
        None // TODO
    }

    fn update_orders_on_md(&mut self, md: &MarketDataEvent) {
        // Update order state, generate UDS and put them into buffer
    }
}

impl SimulatedBroker for SimBroker {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn on_new_timestamp(&mut self, ts: u64) -> Option<Vec<Event>> {
        while let Ok(exchange_request) = self.incoming_request_receiver.try_recv() {
            let wrapped_request = SimBrokerExchangeRequest {
                ack_timestamp: self.last_ts + self.wire_latency,
                exchange_request,
            };
            self.pending_requests.push(wrapped_request);
        }
        self.last_ts = ts; // we need to set ts only after we receive all incoming requests
        self.flush_event_buffer_on_ts()
    }

    fn on_new_market_data(&mut self, md: &MarketDataEvent) -> Option<Vec<Event>> {
        let md_ts = md.get_timestamp();
        let events = self.on_new_timestamp(md_ts);
        self.update_orders_on_md(md);
        events
    }
}
