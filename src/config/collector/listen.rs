//! Listen config.

use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

/// Default port number (Sagan's number: 7E22)
pub const DEFAULT_PORT: u16 = 7322;

/// Listen config: controls where the collector listens for connections
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// IPv4 address to listen on
    // TODO(tarcieri): IPv6
    pub addr: Ipv4Addr,

    /// Port to listen on
    pub port: u16,

    /// Protocol to listen on
    pub protocol: Protocol,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: Ipv4Addr::new(127, 0, 0, 1),
            port: DEFAULT_PORT,
            protocol: Protocol::default(),
        }
    }
}

/// Protocol to listen on
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Protocol {
    /// Plaintext HTTP
    // TODO(tarcieri): HTTPS, gRPC
    #[serde(rename = "http")]
    Http,
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::Http
    }
}
