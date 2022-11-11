_Geger_ is a framework for building trading systems. It's pretty much bare bone by design. Infrastructure may vary from algo to algo that's why I tried to keep it simple and flexible.

**Current status:**
* only backtesting mode
* execution on trades and quotes data
* multiple strategies on same core
* support multiple symbols and exchanges in single Actor (Strategy)
* wire and internal latency emulation per broker basis
* some test coverage
* simulation supports only GTC limit orders


## **Architecture overview**

It's event driven framework. You can implement you algo as Actor to handle all possible events and send request to exchanges(brokers).

General Event Path:

_Market Data Provider -> Environment -> Core -> Actors_

Sim Environment (which is the only implemented one) Event Path:

_Market Data Provider -> SimBrokers (which generate broker events) -> Core_



**Actor trait**

Every strategy have to implement Actor trait, but Actor might be more then strategy - it could be OMS, risk module, balance updater (you have to create all of this on your own)
```rust
pub trait Actor {
    fn on_event(&mut self, event: &Event, gw_router: &mut GatewayRouter);
}

```

**Event**
```rust
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum Event {
    NewMarketTrade(Trade),
    NewQuote(Quote),
    ResponseNewOrderAccepted(NewOrderAccepted),
    ResponseNewOrderRejected(NewOrderRejected),
    ResponseCancelOrderAccepted(CancelOrderAccepted),
    ResponseCancelOrderRejected(CancelOrderRejected),
    UDSOrderUpdate(OrderUpdate),
}
```

**Exchange requests** that you can send to gateway router

```rust
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ExchangeRequest {
    NewOrder(NewOrderRequest),
    CancelOrder(CancelOrderRequest),
}


#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct NewOrderRequest {
    pub request_id: ExchangeRequestID,
    pub client_order_id: ClientOrderId,
    pub exchange: Exchange,
    pub r#type: OrderType,
    pub time_in_force: TimeInForce,
    pub price: Option<f64>,
    pub trigger_price: Option<f64>,
    pub symbol: Symbol,
    pub quantity: f64,
    pub side: Side,
    pub creation_ts: Timestamp,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CancelOrderRequest {
    pub request_id: ExchangeRequestID,
    pub client_order_id: ClientOrderId,
    pub exchange_order_id: ExchangeOrderId,
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub creation_ts: Timestamp,
}
```


## **Quick start**

To start testing you need to implement 2 things:
* you strategy (Actor)
* market data provider, which for sure can be reused

```rust
pub trait SimulatedTradingMarketDataProvider {
    fn next_event(&mut self) -> Option<MarketDataEvent>;
}
```

see example strategy [here](examples/strategy.rs)


## Disclaimer
You will use this code on your own risk!

I EXPRESSLY DISCLAIMS ALL REPRESENTATIONS AND WARRANTIES, EXPRESS OR IMPLIED, WITH RESPECT TO THE CODE, INCLUDING THE WARRANTIES OF MERCHANTABILITY AND OF FITNESS FOR A PARTICULAR PURPOSE. UNDER NO CIRCUMSTANCES INCLUDING NEGLIGENCE SHALL I BE LIABLE FOR ANY DAMAGES, INCIDENTAL, SPECIAL, CONSEQUENTIAL OR OTHERWISE (INCLUDING WITHOUT LIMITATION DAMAGES FOR LOSS OF PROFITS, BUSINESS INTERRUPTION, LOSS OF INFORMATION OR OTHER PECUNIARY LOSS) THAT MAY RESULT FROM THE USE OF OR INABILITY TO USE THE CODE, EVEN IF I HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH DAMAGES.
