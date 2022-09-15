use crate::common::types::{
    ClientOrderId, Exchange, ExchangeOrderId, OrderStatus, OrderType, Side, Symbol, TimeInForce,
    Timestamp,
};

pub struct Order {
    pub(crate) create_ts: Timestamp,
    pub(crate) update_ts: Timestamp,
    pub(crate) exchange_order_id: Option<ExchangeOrderId>,
    pub(crate) client_order_id: ClientOrderId,
    pub(crate) exchange: Exchange,
    pub(crate) r#type: OrderType,
    pub(crate) time_in_force: TimeInForce,
    pub(crate) price: Option<f64>,
    pub(crate) trigger_price: Option<f64>,
    pub(crate) symbol: Symbol,
    pub(crate) side: Side,
    pub(crate) quantity: f64,
    pub(crate) filled_quantity: Option<f64>,
    pub(crate) avg_fill_price: Option<f64>,
    pub(crate) status: OrderStatus,
}
