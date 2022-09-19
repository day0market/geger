use crate::common::events::{
    CancelOrderAccepted, CancelOrderRejected, Event, NewOrderAccepted, NewOrderRejected,
    OrderUpdate,
};
use crate::common::market_data::MarketDataEvent;
use crate::common::order::Order;
use crate::common::types::{Exchange, ExecutionType, OrderStatus, Timestamp};
use crate::core::gateway_router::{CancelOrderRequest, ExchangeRequest, NewOrderRequest};
use crate::sim_environment::SimulatedBroker;
use crossbeam_channel::Receiver;
use std::collections::HashMap;

type InternalID = u64;

struct SimBrokerExchangeRequest {
    ack_timestamp: Timestamp,
    request_id: InternalID,
    exchange_request: ExchangeRequest,
}

pub struct SimBroker {
    exchange: Exchange,
    last_exchange_order_id: InternalID,
    last_request_id: InternalID,
    last_generated_event_id: InternalID,
    last_ts: Timestamp,
    open_orders: HashMap<InternalID, Order>,
    done_orders: HashMap<InternalID, Order>,
    order_id_mapping: HashMap<String, InternalID>,
    pending_requests: HashMap<InternalID, SimBrokerExchangeRequest>,
    incoming_request_receiver: Receiver<ExchangeRequest>,
    generated_events: HashMap<InternalID, Event>,

    wire_latency: Timestamp,
    internal_latency: Timestamp,
}

impl SimBroker {
    pub fn new(exchange: Exchange, incoming_request_receiver: Receiver<ExchangeRequest>) -> Self {
        Self {
            exchange,
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
            wire_latency: 5,
            internal_latency: 2,
        }
    }

    fn execute_requests_after_ts(&mut self, ts: Timestamp) {
        if self.pending_requests.len() == 0 {
            return;
        }
        let keys: Vec<InternalID> = self.pending_requests.keys().map(|&k| k).collect();
        for req_id in keys {
            if &self.pending_requests[&req_id].ack_timestamp > &ts {
                continue;
            }

            let wrapped_request = match self.pending_requests.remove(&req_id) {
                Some(val) => val,
                None => unreachable!(),
            };

            match &wrapped_request.exchange_request {
                ExchangeRequest::NewOrder(request) => {
                    self.on_new_order_request(request, wrapped_request.ack_timestamp)
                }
                ExchangeRequest::CancelOrder(request) => {
                    self.on_cancel_order_requests(request, wrapped_request.ack_timestamp)
                }
            };
        }
    }

    fn update_orders_on_md(&mut self, md: &MarketDataEvent) {
        // Update order state, generate UDS and put them into buffer
    }

    fn on_new_order_request(&mut self, request: &NewOrderRequest, ts: Timestamp) {
        if self.order_id_mapping.contains_key(&request.client_order_id) {
            let order_rejected = NewOrderRejected {
                timestamp: ts,
                client_order_id: request.client_order_id.clone(),
                reason: "duplicate client order id".to_string(),
            };
            self.add_generated_event(Event::ResponseNewOrderRejected(order_rejected));
            return;
        }

        self.last_exchange_order_id += 1;
        let exchange_order_id = self.last_exchange_order_id;
        let exchange_order_id_str = self.last_exchange_order_id.to_string();
        let order_accepted = NewOrderAccepted {
            timestamp: ts,
            client_order_id: request.client_order_id.to_string(),
            exchange_order_id: exchange_order_id_str.clone(),
        };
        self.add_generated_event(Event::ResponseNewOrderAccepted(order_accepted));

        let order_update = OrderUpdate {
            exchange: request.exchange.clone(),
            timestamp: ts + self.internal_latency,
            symbol: request.symbol.clone(),
            side: request.side.clone(),
            client_order_id: Some(request.client_order_id.clone()),
            exchange_order_id: Some(exchange_order_id_str.clone()),
            order_type: Some(request.r#type.clone()),
            time_in_force: Some(request.time_in_force.clone()),
            original_qty: request.quantity,
            original_price: request.price,
            average_price: None,
            stop_price: None,
            execution_type: ExecutionType::NEW,
            order_status: OrderStatus::NEW,
            last_filled_qty: None,
            accumulated_filled_qty: None,
            last_filled_price: None,
            last_trade_time: None,
        };

        self.add_generated_event(Event::UDSOrderUpdate(order_update));
        let mut order = Order::new_order_from_exchange_request(request, ts);
        if let Err(err) = order.set_confirmed_by_exchange(exchange_order_id_str.clone(), ts) {
            panic!("failed to confirm order: {:?}", err)
        };

        self.open_orders.insert(exchange_order_id, order);
        self.order_id_mapping
            .insert(request.client_order_id.clone(), exchange_order_id);
    }

    fn add_generated_event(&mut self, event: Event) {
        self.last_generated_event_id += 1;
        self.generated_events
            .insert(self.last_generated_event_id, event);
    }

    fn on_cancel_order_requests(&mut self, request: &CancelOrderRequest, ts: Timestamp) {
        let exchange_order_id = match request.exchange_order_id.parse::<InternalID>() {
            Ok(val) => val,
            Err(_) => {
                let cancel_rejected = CancelOrderRejected {
                    timestamp: ts,
                    client_order_id: request.client_order_id.clone(),
                    exchange_order_id: Some(request.exchange_order_id.clone()),
                    reason: "invalid exchange order id".to_string(),
                };
                self.add_generated_event(Event::ResponseCancelOrderRejected(cancel_rejected));
                return;
            }
        };

        let mut order = match self.open_orders.remove(&exchange_order_id) {
            Some(order) => order,
            None => {
                let cancel_rejected = CancelOrderRejected {
                    timestamp: ts,
                    client_order_id: request.client_order_id.clone(),
                    exchange_order_id: Some(request.exchange_order_id.clone()),
                    reason: "order not found".to_string(),
                };
                self.add_generated_event(Event::ResponseCancelOrderRejected(cancel_rejected));
                return;
            }
        };

        if let Err(err) = &order.cancel(ts) {
            panic!("failed to cancel order: {:?}", err)
        };

        let exchange_order_id_str = exchange_order_id.to_string();

        let cancel_accepted = CancelOrderAccepted {
            timestamp: ts,
            client_order_id: order.client_order_id.clone(),
            exchange_order_id: exchange_order_id_str.clone(),
        };
        self.add_generated_event(Event::ResponseCancelOrderAccepted(cancel_accepted));
        let order_update = OrderUpdate {
            exchange: request.exchange.clone(),
            timestamp: ts + self.internal_latency,
            symbol: order.symbol.clone(),
            side: order.side.clone(),
            client_order_id: Some(order.client_order_id.clone()),
            exchange_order_id: Some(exchange_order_id_str),
            order_type: Some(order.r#type.clone()),
            time_in_force: Some(order.time_in_force.clone()),
            original_qty: order.quantity,
            original_price: order.price,
            average_price: order.avg_fill_price,
            stop_price: order.trigger_price,
            execution_type: ExecutionType::CANCELED,
            order_status: OrderStatus::CANCELED,
            last_filled_qty: None,
            accumulated_filled_qty: order.filled_quantity,
            last_filled_price: None,
            last_trade_time: None,
        };
        self.add_generated_event(Event::UDSOrderUpdate(order_update));

        self.done_orders.insert(exchange_order_id, order);
    }

    fn get_generated_events_before_ts(&mut self, ts: Timestamp) -> Vec<Event> {
        let mut events = vec![];
        if self.generated_events.len() == 0 {
            return events;
        }

        let event_ids: Vec<u64> = self.generated_events.keys().map(|&k| k).collect();
        for event_id in event_ids {
            if &self.generated_events[&event_id].timestamp() > &ts {
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

    fn process_requests_on_new_ts(&mut self, ts: Timestamp) {
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
    fn exchange(&self) -> Exchange {
        self.exchange.clone()
    }

    fn on_new_timestamp(&mut self, ts: Timestamp) -> Vec<Event> {
        self.process_requests_on_new_ts(ts);
        let events = self.get_generated_events_before_ts(ts);
        self.last_ts = ts;
        events
    }

    fn on_new_market_data(&mut self, md: &MarketDataEvent) -> Vec<Event> {
        let md_ts = md.timestamp();
        self.process_requests_on_new_ts(md_ts);
        self.update_orders_on_md(md);
        let events = self.get_generated_events_before_ts(md_ts);
        self.last_ts = md_ts;
        events
    }
}
