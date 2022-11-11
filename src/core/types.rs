use serde::{Deserialize, Serialize};

pub type Exchange = String;
pub type ClientOrderId = String;
pub type ExchangeOrderId = String;
pub type Symbol = String;
pub type Timestamp = u64;
pub type EventId = String;
pub type ExchangeRequestID = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    MARKET,
    LIMIT,
    STOP,
    LIQUIDATION,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
    GTX,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionType {
    NEW,
    CANCELED,
    CALCULATED, //Liquidation Execution
    EXPIRED,
    TRADE,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum OrderStatus {
    NEW,
    PARTIALLY_FILLED,
    FILLED,
    CANCELED,
    EXPIRED,
    NEW_INSURANCE, //Liquidation with Insurance Fund
    NEW_ADL,       // Counterparty Liquidation`
}
