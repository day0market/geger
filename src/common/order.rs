use crate::common::types::Exchange;
use crate::common::uds::{OrderStatus, OrderType, Side, TimeInForce};
use crate::core::core::{ExchangeRequest, NewOrderRequest};

pub struct Order {
    pub(crate) create_ts: u64,
    pub(crate) update_ts: u64,
    pub(crate) exchange_order_id: Option<String>,
    pub(crate) client_order_id: String,
    pub(crate) exchange: Exchange,
    pub(crate) r#type: OrderType,
    pub(crate) time_in_force: TimeInForce,
    pub(crate) price: Option<f64>,
    pub(crate) trigger_price: Option<f64>,
    pub(crate) symbol: String,
    pub(crate) side: Side,
    pub(crate) quantity: f64,
    pub(crate) filled_quantity: Option<f64>,
    pub(crate) avg_fill_price: Option<f64>,
    pub(crate) status: OrderStatus,
}
