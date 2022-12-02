use crate::core::gateway_router::{
    CancelOrderRequest, ExchangeRequest, GatewayRouter, GatewayRouterError, NewOrderRequest,
};
use crate::core::message_bus::{CrossbeamMessageSender, Message, MessageSender, SimpleMessage};
use crate::core::types::Exchange;
use crossbeam_channel::Receiver;
use log::warn;
use std::collections::HashMap;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum ActionError {
    GatewayRouterError(GatewayRouterError),
    SendMessageError(String),
    ActionNotSupported(String),
}

impl From<GatewayRouterError> for ActionError {
    fn from(err: GatewayRouterError) -> Self {
        Self::GatewayRouterError(err)
    }
}

#[derive(Clone, Debug)]
pub struct ActionsContext<M: Message, T: MessageSender<M>> {
    phantom: PhantomData<M>,
    gw_router: GatewayRouter,
    message_sender: Option<T>,
}

impl<M: Message, T: MessageSender<M>> ActionsContext<M, T> {
    pub fn exchange_requests_receivers(&self) -> HashMap<Exchange, Receiver<ExchangeRequest>> {
        self.gw_router.receivers()
    }
    pub fn new_with_sender(gw_router: GatewayRouter, message_sender: T) -> Self {
        Self {
            phantom: Default::default(),
            gw_router,
            message_sender: Some(message_sender),
        }
    }

    pub fn send_exchange_request(
        &mut self,
        request: ExchangeRequest,
    ) -> Result<(), GatewayRouterError> {
        self.gw_router.send_request(request)
    }

    pub fn send_order(&mut self, request: NewOrderRequest) -> Result<(), ActionError> {
        self.gw_router.send_order(request)?;
        Ok(())
    }

    pub fn cancel_order(&mut self, request: CancelOrderRequest) -> Result<(), ActionError> {
        self.gw_router.cancel_order(request)?;
        Ok(())
    }

    pub fn send_message(&mut self, message: M) -> Result<(), ActionError> {
        warn!("send new message: {:?}", &message);
        match &mut self.message_sender {
            Some(val) => match val.send_message(message) {
                Ok(_) => Ok(()),
                Err(err) => Err(ActionError::SendMessageError(err)),
            },
            None => Err(ActionError::ActionNotSupported(
                "send_message is not supported".into(),
            )),
        }
    }
}

impl ActionsContext<SimpleMessage, CrossbeamMessageSender<SimpleMessage>> {
    pub fn new(gw_router: GatewayRouter) -> Self {
        Self {
            phantom: Default::default(),
            gw_router,
            message_sender: None,
        }
    }
}
