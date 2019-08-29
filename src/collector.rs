//! HTTP collector

use crate::{
    error::{Error, ErrorKind},
    message,
    prelude::*,
    response,
};
use abscissa_core::Runnable;
use std::{net::IpAddr, str::FromStr};
use tendermint::net;
use warp::{path, Filter};

/// HTTP service exposed by the collector
pub struct HttpServer {
    /// Bind address to listen on
    addr: IpAddr,

    /// Port to listen on
    port: u16,
}

impl HttpServer {
    /// Create a new HTTP collector
    pub fn new(listen_addr: &net::Address) -> Result<Self, Error> {
        match listen_addr {
            net::Address::Tcp { host, port, .. } => Ok(Self {
                addr: IpAddr::from_str(host).unwrap(),
                port: *port,
            }),
            other => fail!(ErrorKind::Config, "unsupported listen address: {}", other),
        }
    }
}

impl Runnable for HttpServer {
    /// Run the HTTP collector
    fn run(&self) {
        // GET /net/:network_id
        let network = warp::get2().and(path!("net" / String).map(|network_id| {
            let app = app_reader();
            let result = app
                .network(network_id)
                .map(|network| network.state())
                .ok_or_else(|| response::Error {});
            warp::reply::json(&response::Wrapper::from_result(result))
        }));

        // POST /collector
        let collector = warp::post2()
            .and(path("collector"))
            .and(warp::body::content_length_limit(1024 * 64))
            .and(warp::body::json())
            .map(|envelope: message::Envelope| {
                let mut app = app_writer();
                app.handle_message(envelope);
                warp::reply()
            });

        let routes = network.or(collector);

        warp::serve(routes).run((self.addr, self.port));
    }
}
