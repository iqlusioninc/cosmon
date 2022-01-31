//! HTTP collector

mod pager;
mod poller;
mod request;
mod response;
mod router;

pub use self::{
    pager::Pager,
    poller::Poller,
    request::{PollEvent, Request},
    response::Response,
    router::Router,
};

use crate::{
    config, message,
    network::{self, Network},
    prelude::*,
};
use std::{
    future::Future,
    pin::Pin,
    process,
    task::{Context, Poll},
};
use tower::Service;

/// Collector state
#[derive(Debug)]
pub struct Collector {
    /// Network states
    networks: Map<network::Id, Network>,
}

impl Collector {
    /// Initialize collector state
    pub fn new(config: &config::collector::Config) -> Result<Self, Error> {
        let mut networks = Map::default();

        for network in Network::from_config(&config.networks) {
            let network_id = network.id();

            if networks.insert(network_id.clone(), network).is_some() {
                // TODO(tarcieri): bubble up this error
                status_err!("duplicate network in config: {}", &network_id);
                process::exit(1);
            }
        }

        Ok(Self { networks })
    }

    /// Handle an incoming message
    fn handle_message(&mut self, msg: message::Envelope) -> Result<Response, Error> {
        // TODO(tarcieri): use `network::Id` in `message::Envelope`
        let network_id = network::Id::from(msg.network.clone());

        match self.networks.get_mut(&network_id) {
            Some(network) => {
                network.handle_message(msg);
            }
            None => {
                // TODO(tarcieri): bubble up this error?
                warn!("got message for unregistered network: {}", network_id);
            }
        }

        Ok(Response::Message)
    }

    /// Get network statue
    fn network_state(&self, network_id: &network::Id) -> Result<Response, Error> {
        match self.networks.get(network_id) {
            Some(network) => Ok(network.state().into()),
            None => {
                // TODO(tarcieri): 404 here
                panic!("unknown network ID! {}", network_id)
            }
        }
    }

    fn get_pager_events(&mut self) -> Result<Response, Error> {
        let mut events = Vec::new();

        for (_, network) in &mut self.networks {
            if let Some(event) = network.get_pager_events() {
                events.push(event);
            }
        }

        Ok(Response::PagerEvents(events))
    }

    /// Handle incoming poller info
    fn handle_poll_event(&mut self, event: PollEvent) -> Result<Response, Error> {
        self.networks
            .get_mut(&event.network_id)
            .expect("missing network") // TODO(tarcieri): don't panic
            .handle_poll_event(event);

        Ok(Response::PollEvent)
    }
}

impl Service<Request> for Collector {
    type Response = Response;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let result = match req {
            Request::Message(msg) => self.handle_message(msg),
            Request::NetworkState(id) => self.network_state(&id),
            Request::PagerEvents => self.get_pager_events(),
            Request::PollEvent(info) => self.handle_poll_event(info),
        };

        Box::pin(async { result })
    }
}
