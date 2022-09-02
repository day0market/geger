use crate::common::events::{Event, MarketDataEvent};
use crate::core::core::EventProvider;

enum SimBrokerEvent {
    PendingOrderEvent,
    PendingCancelEvent,
    ExecutionEvent,
    OrderConfirmationEvent,
    OrderRejectionEvent,
    OrderCancelEvent,
    CancelRejectedEvent,
    MarketDataEvent,
}

pub trait SimulatedBrokerMarketDataProvider {
    fn next_event(&mut self) -> Option<MarketDataEvent>;
}
pub struct SimulatedBroker<T: SimulatedBrokerMarketDataProvider> {
    md_provider: T,
}

impl<T: SimulatedBrokerMarketDataProvider> SimulatedBroker<T> {
    pub fn new(md_provider: T) -> Self {
        Self { md_provider }
    }
}

impl<T: SimulatedBrokerMarketDataProvider> EventProvider for SimulatedBroker<T> {
    fn next_event(&mut self) -> Option<Event> {
        None
    }
}
