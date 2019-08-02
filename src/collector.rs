//! HTTP collector

use crate::monitor::message;
use std::{net::IpAddr, str::FromStr};
use tendermint::net;
use warp::Filter;

/// Run the HTTP collector
pub fn run(listen_addr: &net::Address) {
    let addr = match listen_addr {
        net::Address::Tcp { host, port, .. } => (IpAddr::from_str(host).unwrap(), *port),
        other => panic!("unsupported listen addr: {:?}", other),
    };

    // POST /collector
    let collector = warp::post2()
        .and(warp::path("collector"))
        .and(warp::body::content_length_limit(1024 * 64))
        .and(warp::body::json())
        .map(|envelope: message::Envelope| {
            dbg!(envelope);
            warp::reply()
        });

    warp::serve(collector).run(addr);
}
