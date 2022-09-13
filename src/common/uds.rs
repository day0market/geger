#[derive(Debug)]
pub enum UDSMessage {
    //TODO
    OrderUpdate(OrderUpdate),
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

#[derive(Debug, Clone)]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Debug, Clone)]
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

pub enum CancelRequestStatus {
    Accepted,
    Rejected,
    AlreadyCanceled,
}

pub struct CancelResponse {
    pub client_order_id: Option<String>,
    pub exchange_order_id: Option<String>,
    pub status: CancelRequestStatus,
}
