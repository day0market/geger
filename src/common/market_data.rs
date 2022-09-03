use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trade {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quote {
    #[serde(rename(deserialize = "s"))]
    pub symbol: String,
    #[serde(rename(deserialize = "b"))]
    pub bid: f64,
    #[serde(rename(deserialize = "a"))]
    pub ask: f64,
    #[serde(rename(deserialize = "bs"))]
    pub bid_size: Option<f64>,
    #[serde(rename(deserialize = "as"))]
    pub ask_size: Option<f64>,
    #[serde(rename(deserialize = "e"))]
    pub exchange_timestamp: f64,
    #[serde(rename(deserialize = "r"))]
    pub received_timestamp: f64,
}
