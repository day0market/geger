use crate::core::gateway_router::{
    CancelOrderRequest, ExchangeRequest, GatewayRouter, GatewayRouterError, NewOrderRequest,
};
use crate::core::message_bus::{Message, MessageSender};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct ActionsContext<M: Message, T: MessageSender<M>> {
    phantom: PhantomData<M>,
    gw_router: GatewayRouter,
    message_sender: T,
}

impl<M: Message, T: MessageSender<M>> ActionsContext<M, T> {
    pub fn new(gw_router: GatewayRouter, message_sender: T) -> Self {
        Self {
            phantom: Default::default(),
            gw_router,
            message_sender,
        }
    }

    pub fn send_exchange_request(
        &mut self,
        request: ExchangeRequest,
    ) -> Result<(), GatewayRouterError> {
        self.gw_router.send_request(request)
    }

    pub fn send_order(&mut self, request: NewOrderRequest) -> Result<(), GatewayRouterError> {
        self.gw_router.send_order(request)
    }

    pub fn cancel_order(&mut self, request: CancelOrderRequest) -> Result<(), GatewayRouterError> {
        self.gw_router.cancel_order(request)
    }

    pub fn send_message(&mut self, message: M) -> Result<(), String> {
        self.message_sender.send_message(message)
    }
}
