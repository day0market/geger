use crate::common::order::Order;
use crate::common::uds::{OrderStatus, OrderType};
use crate::core::core::NewOrderRequest;

#[derive(Debug)]
pub enum Error {
    UnreachableStatus,
}

impl Order {
    pub(crate) fn new_order_from_exchange_request(request: &NewOrderRequest, ts: u64) -> Self {
        let (price, trigger_price) = match request.r#type {
            OrderType::LIMIT => (request.price, None),
            OrderType::MARKET => (None, None),
            OrderType::STOP => (None, request.trigger_price),
            _ => unimplemented!(),
        };

        Self {
            create_ts: ts,
            update_ts: ts,
            exchange_order_id: None,
            client_order_id: request.client_order_id.clone(),
            exchange: request.exchange.clone(),
            r#type: request.r#type.clone(),
            time_in_force: request.time_in_force.clone(),
            price,
            trigger_price,
            symbol: request.symbol.clone(),
            side: request.side.clone(),
            quantity: request.quantity,
            filled_quantity: None,
            avg_fill_price: None,
            status: OrderStatus::NEW,
        }
    }

    pub(crate) fn set_confirmed_by_exchange(
        &mut self,
        exchange_order_id: String,
        confirmation_ts: u64,
    ) -> Result<(), Error> {
        if self.status != OrderStatus::NEW {
            Err(Error::UnreachableStatus)
        } else {
            self.update_ts = self.update_ts.max(confirmation_ts);
            self.exchange_order_id = Some(exchange_order_id);
            Ok(())
        }
    }

    pub(crate) fn cancel(&mut self, cancel_ts: u64) -> Result<(), Error> {
        if self.status == OrderStatus::FILLED || self.exchange_order_id.is_none() {
            Err(Error::UnreachableStatus)
        } else {
            self.update_ts = self.update_ts.max(cancel_ts);
            Ok(())
        }
    }
}
