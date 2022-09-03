use crate::common::events::{Event, MarketDataEvent};
use crate::common::market_data::{Quote, Trade};
use crate::common::uds::UDSMessage;

pub trait EventProvider {
    fn next_event(&mut self) -> Option<Event>;
}

pub trait Strategy {
    fn on_trade(&mut self, trade: &Trade);
    fn on_quote(&mut self, quote: &Quote);
    fn on_uds(&mut self, uds: &UDSMessage);
}

pub struct Core<T: EventProvider> {
    event_provider: T,
}

impl<T: EventProvider> Core<T> {
    pub fn new(event_provider: T) -> Self {
        Self { event_provider }
    }

    pub fn run(&mut self) {
        'event_loop: loop {
            let event = self.event_provider.next_event();
            if event.is_none() {
                println!("event is none");
                break 'event_loop;
            }
            let event = event.unwrap();
            println!("received event {:?}", &event);
            match event {
                Event::MarketDataEvent(event) => self.process_md_event(event),
                Event::UDSMessage(event) => self.process_uds(event),
            }
        }
    }

    fn process_md_event(&mut self, event: MarketDataEvent) {}

    fn process_uds(&mut self, event: UDSMessage) {}
}
