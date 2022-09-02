use geger::common::events::MarketDataEvent;
use geger::core::core::Core;
use geger::sim_broker::broker::{SimulatedBroker, SimulatedBrokerMarketDataProvider};

struct FileMarketDataProvider {
    directory: String,
}

impl FileMarketDataProvider {
    fn new(directory: &str) -> Self {
        Self {
            directory: directory.to_string(),
        }
    }
}

impl SimulatedBrokerMarketDataProvider for FileMarketDataProvider {
    fn next_event(&mut self) -> Option<MarketDataEvent> {
        todo!()
    }
}

fn main() {
    let md_provider = FileMarketDataProvider::new("");
    let sim_broker = SimulatedBroker::new(md_provider);
    let mut core = Core::new(sim_broker);
    core.run()
}
