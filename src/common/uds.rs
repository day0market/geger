#[derive(Debug)]
pub enum UDSMessage {
    //TODO
    OrderUpdate(OrderUpdate),
}

#[derive(Debug)]
pub enum OrderType {
    MARKET,
    LIMIT,
    STOP,
    LIQUIDATION,
}

#[derive(Debug)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
    GTX,
}

#[derive(Debug)]
pub enum ExecutionType {
    NEW,
    CANCELED,
    CALCULATED, //Liquidation Execution
    EXPIRED,
    TRADE,
}

#[derive(Debug)]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Debug)]
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
    symbol: String,
    side: Side,
    client_order_id: Option<String>,
    exchange_order_id: Option<String>,
    order_type: Option<OrderType>,
    time_in_force: Option<TimeInForce>,
    original_qty: f64,
    original_price: f64,
    average_price: Option<f64>,
    stop_price: Option<f64>,
    execution_type: ExecutionType,
    order_status: OrderStatus,
    last_filled_qty: Option<f64>,
    accumulated_filled_qty: Option<f64>,
    last_filled_price: Option<f64>,
    last_trade_time: Option<f64>,
}
