//! Remote addresses (`tcp://` or `unix://`)

use eyre::eyre;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Display},
    path::PathBuf,
    str::{self, FromStr},
};
use tendermint::node;
use url::Url;

/// URI prefix for TCP connections
pub const TCP_PREFIX: &str = "tcp://";

/// URI prefix for Unix socket connections
pub const UNIX_PREFIX: &str = "unix://";

/// Remote address (TCP or UNIX socket)
///
/// For TCP-based addresses, this supports both IPv4 and IPv6 addresses and
/// hostnames.
///
/// If the scheme is not supplied (i.e. `tcp://` or `unix://`) when parsing
/// from a string, it is assumed to be a TCP address.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Address {
    /// TCP connections
    Tcp {
        /// Remote peer ID
        peer_id: Option<node::Id>,

        /// Hostname or IP address
        host: String,

        /// Port
        port: u16,
    },

    /// UNIX domain sockets
    Unix {
        /// Path to a UNIX domain socket path
        path: PathBuf,
    },
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::from_str(&String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Tcp {
                peer_id: None,
                host,
                port,
            } => write!(f, "{}{}:{}", TCP_PREFIX, host, port),
            Address::Tcp {
                peer_id: Some(peer_id),
                host,
                port,
            } => write!(f, "{}{}@{}:{}", TCP_PREFIX, peer_id, host, port),
            Address::Unix { path } => write!(f, "{}{}", UNIX_PREFIX, path.display()),
        }
    }
}

impl FromStr for Address {
    type Err = eyre::Report;

    fn from_str(addr: &str) -> eyre::Result<Self> {
        let prefixed_addr = if addr.contains("://") {
            addr.to_owned()
        } else {
            // If the address has no scheme, assume it's TCP
            format!("{}{}", TCP_PREFIX, addr)
        };
        let url = Url::parse(&prefixed_addr)?;
        match url.scheme() {
            "tcp" => Ok(Self::Tcp {
                peer_id: if !url.username().is_empty() {
                    Some(url.username().parse()?)
                } else {
                    None
                },
                host: url
                    .host_str()
                    .ok_or_else(|| eyre!("invalid TCP address (missing host): {}", addr))?
                    .to_owned(),
                port: url
                    .port()
                    .ok_or_else(|| eyre!("invalid TCP address (missing port): {}", addr))?,
            }),
            "unix" => Ok(Self::Unix {
                path: PathBuf::from(url.path()),
            }),
            _ => Err(eyre!("invalid address scheme: {:?}", addr)),
        }
    }
}

impl Serialize for Address {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tendermint::node;
    //use crate::node;

    const EXAMPLE_TCP_ADDR: &str =
        "tcp://abd636b766dcefb5322d8ca40011ec2cb35efbc2@35.192.61.41:26656";
    const EXAMPLE_TCP_ADDR_WITHOUT_ID: &str = "tcp://35.192.61.41:26656";
    const EXAMPLE_UNIX_ADDR: &str = "unix:///tmp/node.sock";
    const EXAMPLE_TCP_IPV6_ADDR: &str =
        "tcp://abd636b766dcefb5322d8ca40011ec2cb35efbc2@[2001:0000:3238:DFE1:0063:0000:0000:FEFB]:26656";

    #[test]
    fn parse_tcp_addr() {
        let tcp_addr_without_prefix = &EXAMPLE_TCP_ADDR[TCP_PREFIX.len()..];

        for tcp_addr in &[EXAMPLE_TCP_ADDR, tcp_addr_without_prefix] {
            match tcp_addr.parse::<Address>().unwrap() {
                Address::Tcp {
                    peer_id,
                    host,
                    port,
                } => {
                    assert_eq!(
                        peer_id.unwrap(),
                        "abd636b766dcefb5322d8ca40011ec2cb35efbc2"
                            .parse::<node::Id>()
                            .unwrap()
                    );
                    assert_eq!(host, "35.192.61.41");
                    assert_eq!(port, 26656);
                }
                other => panic!("unexpected address type: {:?}", other),
            }
        }
    }

    #[test]
    fn parse_tcp_addr_without_id() {
        let addr = EXAMPLE_TCP_ADDR_WITHOUT_ID.parse::<Address>().unwrap();
        let addr_without_prefix = EXAMPLE_TCP_ADDR_WITHOUT_ID[TCP_PREFIX.len()..]
            .parse::<Address>()
            .unwrap();
        for addr in &[addr, addr_without_prefix] {
            match addr {
                Address::Tcp {
                    peer_id,
                    host,
                    port,
                } => {
                    assert!(peer_id.is_none());
                    assert_eq!(host, "35.192.61.41");
                    assert_eq!(*port, 26656);
                }
                other => panic!("unexpected address type: {:?}", other),
            }
        }
    }

    #[test]
    fn parse_unix_addr() {
        let addr = EXAMPLE_UNIX_ADDR.parse::<Address>().unwrap();
        match addr {
            Address::Unix { path } => {
                assert_eq!(path.to_str().unwrap(), "/tmp/node.sock");
            }
            other => panic!("unexpected address type: {:?}", other),
        }
    }

    #[test]
    fn parse_tcp_ipv6_addr() {
        let addr = EXAMPLE_TCP_IPV6_ADDR.parse::<Address>().unwrap();
        let addr_without_prefix = EXAMPLE_TCP_IPV6_ADDR[TCP_PREFIX.len()..]
            .parse::<Address>()
            .unwrap();
        for addr in &[addr, addr_without_prefix] {
            match addr {
                Address::Tcp {
                    peer_id,
                    host,
                    port,
                } => {
                    assert_eq!(
                        peer_id.unwrap(),
                        "abd636b766dcefb5322d8ca40011ec2cb35efbc2"
                            .parse::<node::Id>()
                            .unwrap()
                    );
                    // The parser URL strips the leading zeroes and converts to lowercase hex
                    assert_eq!(host, "[2001:0:3238:dfe1:63::fefb]");
                    assert_eq!(*port, 26656);
                }
                other => panic!("unexpected address type: {:?}", other),
            }
        }
    }
}
