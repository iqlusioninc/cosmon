//! Parser for Tendermint `config.toml` files

use crate::error::{Error, ErrorKind};
use abscissa_core::{FrameworkError, FrameworkErrorKind::ConfigError};
use serde::{de, de::Error as _, ser, Deserialize, Serialize};
use std::{
    fmt, fs,
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use tendermint::{net, node, Moniker};

/// Address prefix for TCP connections
const TCP_PREFIX: &str = "tcp://";

/// Address prefix for Unix socket connections
const UNIX_PREFIX: &str = "unix://";

/// Tendermint `config.toml` file
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TendermintConfig {
    /// TCP or UNIX socket address of the ABCI application,
    /// or the name of an ABCI application compiled in with the Tendermint binary.
    pub proxy_app: net::Address,

    /// A custom human readable name for this node
    pub moniker: Moniker,

    /// If this node is many blocks behind the tip of the chain, FastSync
    /// allows them to catchup quickly by downloading blocks in parallel
    /// and verifying their commits
    pub fast_sync: bool,

    /// Database backend: `leveldb | memdb | cleveldb`
    pub db_backend: DbBackend,

    /// Database directory
    pub db_dir: PathBuf,

    /// Output level for logging, including package level options
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_comma_separated_list"
    )]
    pub log_level: Vec<LogLevel>,

    /// Output format: 'plain' (colored text) or 'json'
    pub log_format: LogFormat,

    /// Path to the JSON file containing the initial validator set and other meta data
    pub genesis_file: PathBuf,

    /// Path to the JSON file containing the private key to use as a validator in the consensus protocol
    pub priv_validator_key_file: Option<PathBuf>,

    /// Path to the JSON file containing the last sign state of a validator
    pub priv_validator_state_file: PathBuf,

    /// TCP or UNIX socket address for Tendermint to listen on for
    /// connections from an external PrivValidator process
    #[serde(deserialize_with = "deserialize_optional_net_addr")]
    pub priv_validator_laddr: Option<net::Address>,

    /// Path to the JSON file containing the private key to use for node authentication in the p2p protocol
    pub node_key_file: PathBuf,

    /// Mechanism to connect to the ABCI application: socket | grpc
    pub abci: AbciMode,

    /// TCP or UNIX socket address for the profiling server to listen on
    #[serde(deserialize_with = "deserialize_optional_net_addr")]
    pub prof_laddr: Option<net::Address>,

    /// If `true`, query the ABCI app on connecting to a new peer
    /// so the app can decide if we should keep the connection or not
    pub filter_peers: bool,

    /// rpc server configuration options
    pub rpc: RpcConfig,

    /// peer to peer configuration options
    pub p2p: P2PConfig,

    /// mempool configuration options
    pub mempool: MempoolConfig,

    /// consensus configuration options
    pub consensus: ConsensusConfig,

    /// transactions indexer configuration options
    pub tx_index: TxIndexConfig,

    /// instrumentation configuration options
    pub instrumentation: InstrumentationConfig,
}

impl TendermintConfig {
    /// Parse Tendermint `config.toml`
    pub fn parse_toml<T: AsRef<str>>(toml_string: T) -> Result<Self, FrameworkError> {
        Ok(toml::from_str(toml_string.as_ref())?)
    }

    /// Load `config.toml` from a file
    pub fn load_toml_file<P>(path: &P) -> Result<Self, FrameworkError>
    where
        P: AsRef<Path>,
    {
        let toml_string = fs::read_to_string(path).map_err(|e| {
            err!(
                ConfigError,
                "couldn't open {}: {}",
                path.as_ref().display(),
                e
            )
        })?;

        Self::parse_toml(toml_string)
    }

    /// Is this configuration for a validator?
    pub fn is_validator(&self) -> bool {
        self.priv_validator_key_file.is_some() || self.priv_validator_laddr.is_some()
    }
}

/// Database backend
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum DbBackend {
    /// LevelDB backend
    #[serde(rename = "leveldb")]
    LevelDb,

    /// MemDB backend
    #[serde(rename = "memdb")]
    MemDb,

    /// CLevelDB backend
    #[serde(rename = "cleveldb")]
    CLevelDb,
}

/// Loglevel configuration
// TODO(tarcieri): parse and validate this string
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogLevel(String);

impl FromStr for LogLevel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(LogLevel(s.to_owned()))
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// Logging format
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum LogFormat {
    /// Plain (colored text)
    #[serde(rename = "plain")]
    Plain,

    /// JSON
    #[serde(rename = "json")]
    Json,
}

/// Mechanism to connect to the ABCI application: socket | grpc
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum AbciMode {
    /// Socket
    #[serde(rename = "socket")]
    Socket,

    /// GRPC
    #[serde(rename = "grpc")]
    Grpc,
}

/// Tendermint `config.toml` file's `[rpc]` section
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RpcConfig {
    /// TCP or UNIX socket address for the RPC server to listen on
    pub laddr: net::Address,

    /// A list of origins a cross-domain request can be executed from
    /// Default value `[]` disables cors support
    /// Use `["*"]` to allow any origin
    pub cors_allowed_origins: Vec<CorsOrigin>,

    /// A list of methods the client is allowed to use with cross-domain requests
    pub cors_allowed_methods: Vec<CorsMethod>,

    /// A list of non simple headers the client is allowed to use with cross-domain requests
    pub cors_allowed_headers: Vec<CorsHeader>,

    /// TCP or UNIX socket address for the gRPC server to listen on
    /// NOTE: This server only supports `/broadcast_tx_commit`
    #[serde(deserialize_with = "deserialize_optional_net_addr")]
    pub grpc_laddr: Option<net::Address>,

    /// Maximum number of simultaneous GRPC connections.
    /// Does not include RPC (HTTP&WebSocket) connections. See `max_open_connections`.
    pub grpc_max_open_connections: u64,

    /// Activate unsafe RPC commands like `/dial_seeds` and `/unsafe_flush_mempool`
    #[serde(rename = "unsafe")]
    pub unsafe_commands: bool,

    /// Maximum number of simultaneous connections (including WebSocket).
    /// Does not include gRPC connections. See `grpc_max_open_connections`.
    pub max_open_connections: u64,

    /// Maximum number of unique clientIDs that can `/subscribe`.
    pub max_subscription_clients: u64,

    /// Maximum number of unique queries a given client can `/subscribe` to.
    pub max_subscriptions_per_client: u64,

    /// How long to wait for a tx to be committed during `/broadcast_tx_commit`.
    pub timeout_broadcast_tx_commit: Timeout,

    /// The name of a file containing certificate that is used to create the HTTPS server.
    pub tls_cert_file: PathBuf,

    /// The name of a file containing matching private key that is used to create the HTTPS server.
    pub tls_key_file: PathBuf,
}

/// Origin hosts allowed with CORS requests to the RPC API
// TODO(tarcieri): parse and validate this string
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CorsOrigin(String);

impl fmt::Display for CorsOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// HTTP methods allowed with CORS requests to the RPC API
// TODO(tarcieri): parse and validate this string
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CorsMethod(String);

impl fmt::Display for CorsMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// HTTP headers allowed to be sent via CORS to the RPC API
// TODO(tarcieri): parse and validate this string
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CorsHeader(String);

impl fmt::Display for CorsHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// peer to peer configuration options
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct P2PConfig {
    /// Address to listen for incoming connections
    pub laddr: net::Address,

    /// Address to advertise to peers for them to dial
    /// If empty, will use the same port as the laddr,
    /// and will introspect on the listener or use UPnP
    /// to figure out the address.
    #[serde(deserialize_with = "deserialize_optional_net_addr")]
    pub external_address: Option<net::Address>,

    /// Comma separated list of seed nodes to connect to
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_net_addr_list"
    )]
    pub seeds: Vec<net::Address>,

    /// Comma separated list of nodes to keep persistent connections to
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_net_addr_list"
    )]
    pub persistent_peers: Vec<net::Address>,

    /// UPNP port forwarding
    pub upnp: bool,

    /// Path to address boo
    pub addr_book_file: PathBuf,

    /// Set `true` for strict address routability rules
    /// Set `false` for private or local networks
    pub addr_book_strict: bool,

    /// Maximum number of inbound peers
    pub max_num_inbound_peers: u64,

    /// Maximum number of outbound peers to connect to, excluding persistent peers
    pub max_num_outbound_peers: u64,

    /// Time to wait before flushing messages out on the connection
    pub flush_throttle_timeout: Timeout,

    /// Maximum size of a message packet payload, in bytes
    pub max_packet_msg_payload_size: u64,

    /// Rate at which packets can be sent, in bytes/second
    pub send_rate: TransferRate,

    /// Rate at which packets can be received, in bytes/second
    pub recv_rate: TransferRate,

    /// Set `true` to enable the peer-exchange reactor
    pub pex: bool,

    /// Seed mode, in which node constantly crawls the network and looks for
    /// peers. If another node asks it for addresses, it responds and disconnects.
    ///
    /// Does not work if the peer-exchange reactor is disabled.
    pub seed_mode: bool,

    /// Comma separated list of peer IDs to keep private (will not be gossiped to other peers)
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_comma_separated_list"
    )]
    pub private_peer_ids: Vec<node::Id>,

    /// Toggle to disable guard against peers connecting from the same ip.
    pub allow_duplicate_ip: bool,

    /// Handshake timeout
    pub handshake_timeout: Timeout,

    /// Timeout when dialing other peers
    pub dial_timeout: Timeout,
}

/// mempool configuration options
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MempoolConfig {
    /// Recheck enabled
    pub recheck: bool,

    /// Broadcast enabled
    pub broadcast: bool,

    /// WAL dir
    #[serde(deserialize_with = "deserialize_optional_value")]
    pub wal_dir: Option<PathBuf>,

    /// Maximum number of transactions in the mempool
    pub size: u64,

    /// Limit the total size of all txs in the mempool.
    /// This only accounts for raw transactions (e.g. given 1MB transactions and
    /// `max_txs_bytes`=5MB, mempool will only accept 5 transactions).
    pub max_txs_bytes: u64,

    /// Size of the cache (used to filter transactions we saw earlier) in transactions
    pub cache_size: u64,
}

/// consensus configuration options
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsensusConfig {
    /// Path to WAL file
    pub wal_file: PathBuf,

    /// Propose timeout
    pub timeout_propose: Timeout,

    /// Propose timeout delta
    pub timeout_propose_delta: Timeout,

    /// Prevote timeout
    pub timeout_prevote: Timeout,

    /// Prevote timeout delta
    pub timeout_prevote_delta: Timeout,

    /// Precommit timeout
    pub timeout_precommit: Timeout,

    /// Precommit timeout delta
    pub timeout_precommit_delta: Timeout,

    /// Commit timeout
    pub timeout_commit: Timeout,

    /// Make progress as soon as we have all the precommits (as if TimeoutCommit = 0)
    pub skip_timeout_commit: bool,

    /// EmptyBlocks mode
    pub create_empty_blocks: bool,

    /// Interval between empty blocks
    pub create_empty_blocks_interval: Timeout,

    /// Reactor sleep duration
    pub peer_gossip_sleep_duration: Timeout,

    /// Reactor query sleep duration
    pub peer_query_maj23_sleep_duration: Timeout,
}

/// transactions indexer configuration options
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TxIndexConfig {
    /// What indexer to use for transactions
    #[serde(default)]
    pub indexer: TxIndexer,

    /// Comma-separated list of tags to index (by default the only tag is `tx.hash`)
    // TODO(tarcieri): switch to `tendermint::abci::Tag`
    #[serde(
        serialize_with = "serialize_comma_separated_list",
        deserialize_with = "deserialize_comma_separated_list"
    )]
    pub index_tags: Vec<String>,

    /// When set to true, tells indexer to index all tags (predefined tags:
    /// `tx.hash`, `tx.height` and all tags from DeliverTx responses).
    pub index_all_tags: bool,
}

/// What indexer to use for transactions
#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TxIndexer {
    /// "null"
    // TODO(tarcieri): use an `Option` type here?
    #[serde(rename = "null")]
    Null,

    /// "kv" (default) - the simplest possible indexer, backed by key-value storage (defaults to levelDB; see DBBackend).
    #[serde(rename = "kv")]
    Kv,
}

impl Default for TxIndexer {
    fn default() -> TxIndexer {
        TxIndexer::Kv
    }
}

/// instrumentation configuration options
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstrumentationConfig {
    /// When `true`, Prometheus metrics are served under /metrics on
    /// PrometheusListenAddr.
    pub prometheus: bool,

    /// Address to listen for Prometheus collector(s) connections
    // TODO(tarcieri): parse to `tendermint::net::Addr`
    pub prometheus_listen_addr: String,

    /// Maximum number of simultaneous connections.
    pub max_open_connections: u64,

    /// Instrumentation namespace
    pub namespace: String,
}

/// Timeout value
#[derive(Copy, Clone, Debug)]
pub struct Timeout(Duration);

impl Deref for Timeout {
    type Target = Duration;

    fn deref(&self) -> &Duration {
        &self.0
    }
}

impl From<Duration> for Timeout {
    fn from(duration: Duration) -> Timeout {
        Timeout(duration)
    }
}

impl From<Timeout> for Duration {
    fn from(timeout: Timeout) -> Duration {
        timeout.0
    }
}

impl FromStr for Timeout {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Timeouts are either 'ms' or 's', and should always end with 's'
        if s.len() < 2 || !s.ends_with('s') {
            fail!(ErrorKind::Config, "invalid units");
        }

        let units = match s.chars().nth(s.len() - 2) {
            Some('m') => "ms",
            Some('0'...'9') => "s",
            _ => fail!(ErrorKind::Config, "invalid units"),
        };

        let numeric_part = s.chars().take(s.len() - units.len()).collect::<String>();

        let numeric_value = numeric_part
            .parse::<u64>()
            .map_err(|e| err!(ErrorKind::Config, e))?;

        let duration = match units {
            "s" => Duration::from_secs(numeric_value),
            "ms" => Duration::from_millis(numeric_value),
            _ => unreachable!(),
        };

        Ok(Timeout(duration))
    }
}

impl fmt::Display for Timeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ms", self.as_millis())
    }
}

impl<'de> Deserialize<'de> for Timeout {
    /// Parse `Timeout` from string ending in `s` or `ms`
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let string = String::deserialize(deserializer)?;
        string
            .parse()
            .map_err(|_| D::Error::custom(format!("invalid timeout value: {:?}", &string)))
    }
}

impl Serialize for Timeout {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

/// Rate at which bytes can be sent/received
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct TransferRate(u64);

impl TransferRate {
    /// Get the trasfer rate in bytes per second
    pub fn value(self) -> u64 {
        self.0
    }
}

/// Parse a network address, defaulting to `tcp://` if it has no URI prefix
fn parse_net_addr<'de, D, A>(addr: A) -> Result<net::Address, D::Error>
where
    D: de::Deserializer<'de>,
    A: AsRef<str>,
{
    let addr = addr.as_ref();

    if addr.starts_with(TCP_PREFIX) || addr.starts_with(UNIX_PREFIX) {
        addr.parse()
    } else {
        // Try adding the `tcp://` prefix if there isn't one
        let tcp_addr = format!("{}{}", TCP_PREFIX, addr);
        tcp_addr.parse()
    }
    .map_err(|_| D::Error::custom(format!("error parsing addr: {:?}", &addr)))
}

fn deserialize_optional_net_addr<'de, D>(deserializer: D) -> Result<Option<net::Address>, D::Error>
where
    D: de::Deserializer<'de>,
{
    if let Some(addr) = deserialize_optional_value::<D, String>(deserializer)? {
        Ok(Some(parse_net_addr::<D, String>(addr)?))
    } else {
        Ok(None)
    }
}

/// Deserialize a comma-separated `net::Address` list (sans `tcp://` prefix)
fn deserialize_net_addr_list<'de, D>(deserializer: D) -> Result<Vec<net::Address>, D::Error>
where
    D: de::Deserializer<'de>,
{
    deserialize_comma_separated_list(deserializer)?
        .iter()
        .map(|addr| parse_net_addr::<D, &String>(addr))
        .collect()
}

/// Deserialize `Option<T: FromStr>` where an empty string indicates `None`
fn deserialize_optional_value<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: de::Deserializer<'de>,
    T: FromStr,
{
    let string = String::deserialize(deserializer)?;

    if string.is_empty() {
        return Ok(None);
    }

    string
        .parse()
        .map(Some)
        .map_err(|_| D::Error::custom(format!("error parsing value: {:?}", &string)))
}

/// Deserialize a comma separated list of types that impl `FromStr` as a `Vec`
fn deserialize_comma_separated_list<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: de::Deserializer<'de>,
    T: FromStr,
{
    let mut result = vec![];
    let string = String::deserialize(deserializer)?;

    if string.is_empty() {
        return Ok(result);
    }

    for item in string.split(',') {
        result.push(item.parse().map_err(|_| {
            D::Error::custom(format!("error parsing comma-separated list: {:?}", &string))
        })?);
    }

    Ok(result)
}

/// Serialize a comma separated list types that impl `ToString`
fn serialize_comma_separated_list<S, T>(list: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
    T: ToString,
{
    let str_list = list.iter().map(|addr| addr.to_string()).collect::<Vec<_>>();
    str_list.join(",").serialize(serializer)
}
