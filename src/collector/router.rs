//! Collector HTTP request router

use super::state::State;
use crate::{config, message, prelude::*, response};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::Mutex;
use warp::Filter;

/// Storage cell for collector state
type StateCell = Arc<Mutex<State>>;

/// HTTP request router
#[derive(Clone)]
pub struct Router {
    /// Address to listen on
    addr: ([u8; 4], u16),

    /// Protocol to listen on
    protocol: config::collector::listen::Protocol,

    /// Collector state
    state: StateCell,
}

impl Router {
    /// Initialize the router from the config
    pub fn new(config: &config::collector::Config) -> Result<Router, Error> {
        let addr = (config.listen.addr.octets(), config.listen.port);
        let protocol = config.listen.protocol;
        let state = Arc::new(Mutex::new(State::new(&config)?));

        Ok(Self {
            addr,
            protocol,
            state,
        })
    }

    /// Route incoming requests
    pub async fn run(self) {
        let addr = self.addr;
        let protocol = self.protocol;
        let state = warp::any().map(move || self.state.clone());

        // GET /net/:network_id
        let network = warp::get()
            .and(warp::path!("net" / String))
            .and(state.clone())
            .and_then(network_get);

        // POST /collector
        let collector = warp::post()
            .and(warp::path("collector"))
            .and(warp::path::end())
            .and(warp::body::json())
            .and(state.clone())
            .and_then(collector_post);

        let routes = network.or(collector);

        match protocol {
            config::collector::listen::Protocol::Http => warp::serve(routes).run(addr).await,
        }
    }
}

/// `GET /net/:network_id`: handle incoming requests to get network state
pub async fn network_get(
    network_id: String,
    state_cell: StateCell,
) -> Result<impl warp::Reply, Infallible> {
    let state = state_cell.lock().await;
    let result = state
        .network(&network_id.into())
        .map(|network| network.state())
        .ok_or(response::Error {});

    Ok(warp::reply::json(&response::Wrapper::from_result(result)))
}

/// `POST /collector`: handle incoming messages sent to the collector
///
/// This endpoint is intended to be triggered by the sagan agent
pub async fn collector_post(
    msg: message::Envelope,
    state_cell: StateCell,
) -> Result<impl warp::Reply, Infallible> {
    let mut state = state_cell.lock().await;
    state.handle_message(msg);
    Ok(warp::reply())
}
