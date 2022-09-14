#[derive(Debug)]
pub enum UDSMessage {
    //TODO
    OrderUpdate(OrderUpdate),
}

impl UDSMessage {
    pub fn get_timestamp(&self) -> u64 {
        0 // TODO alex
    }
}

#[derive(Debug, Clone)]
pub enum OrderType {
    MARKET,
    LIMIT,
    STOP,
    LIQUIDATION,
}

#[derive(Debug, Clone)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
    GTX,
}

#[derive(Debug, Clone)]
pub enum ExecutionType {
    NEW,
    CANCELED,
    CALCULATED, //Liquidation Execution
    EXPIRED,
    TRADE,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    NEW,
    PARTIALLY_FILLED,
    FILLED,
    CANCELED,
    EXPIRED,
    NEW_INSURANCE, //Liquidation with Insurance Fund
    NEW_ADL,       // Counterparty Liquidation`
}

#[derive(Debug)]
pub struct OrderUpdate {
    pub timestamp: u64,
    pub symbol: String,
    pub side: Side,
    pub client_order_id: Option<String>,
    pub exchange_order_id: Option<String>,
    pub order_type: Option<OrderType>,
    pub time_in_force: Option<TimeInForce>,
    pub original_qty: f64,
    pub original_price: Option<f64>,
    pub average_price: Option<f64>,
    pub stop_price: Option<f64>,
    pub execution_type: ExecutionType,
    pub order_status: OrderStatus,
    pub last_filled_qty: Option<f64>,
    pub accumulated_filled_qty: Option<f64>,
    pub last_filled_price: Option<f64>,
    pub last_trade_time: Option<f64>,
}
