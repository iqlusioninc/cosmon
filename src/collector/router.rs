//! Collector HTTP request router

use super::{Request, Response};
use crate::{config, message, prelude::*, response};
use std::convert::Infallible;
use tower::{util::ServiceExt, Service};
use warp::Filter;

/// HTTP request router
#[derive(Clone)]
pub struct Router {
    /// Address to listen on
    addr: ([u8; 4], u16),

    /// Protocol to listen on
    protocol: config::collector::listen::Protocol,
}

impl Router {
    /// Initialize the router from the config
    pub fn new(config: &config::collector::Config) -> Self {
        let addr = (config.listen.addr.octets(), config.listen.port);
        let protocol = config.listen.protocol;

        Self { addr, protocol }
    }

    /// Route incoming requests
    pub async fn run<S>(self, collector: S)
    where
        S: Service<Request, Response = Response, Error = BoxError> + Send + Sync + Clone + 'static,
        S::Future: Send,
    {
        let addr = self.addr;
        let protocol = self.protocol;
        let collector = warp::any().map(move || collector.clone());

        // GET /net/:network_id
        let network = warp::get()
            .and(warp::path!("net" / String))
            .and(collector.clone())
            .and_then(network_get);

        // POST /collector
        let collector = warp::post()
            .and(warp::path("collector"))
            .and(warp::path::end())
            .and(warp::body::json())
            .and(collector.clone())
            .and_then(collector_post);

        let routes = network.or(collector);

        match protocol {
            config::collector::listen::Protocol::Http => warp::serve(routes).run(addr).await,
        }
    }
}

/// `GET /net/:network_id`: handle incoming requests to get network state
pub async fn network_get<S>(
    network_id: String,
    mut service: S,
) -> Result<impl warp::Reply, Infallible>
where
    S: Service<Request, Response = Response, Error = BoxError> + Send + Clone + 'static,
{
    let result = service
        .ready()
        .await
        .expect("service not ready")
        .call(Request::NetworkState(network_id.into()))
        .await
        .map(|resp| match resp {
            Response::NetworkState(state) => state,
            other => panic!("unexpected response to request: {:?}", other),
        });

    Ok(warp::reply::json(&response::Wrapper::from_result(result)))
}

/// `POST /collector`: handle incoming messages sent to the collector
///
/// This endpoint is intended to be triggered by the sagan agent
pub async fn collector_post<S>(
    msg: message::Envelope,
    mut service: S,
) -> Result<impl warp::Reply, Infallible>
where
    S: Service<Request, Response = Response, Error = BoxError> + Send + Sync + Clone + 'static,
    S::Future: Send,
{
    let result = service
        .ready()
        .await
        .expect("service not ready")
        .call(msg.into())
        .await;

    if let Err(err) = result {
        // TODO(tarcieri): report errors back to the caller?
        warn!("error handling agent message: {}", err);
    }

    Ok(warp::reply())
}
