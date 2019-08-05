//! Tendermint node monitoring support

pub mod data;
pub mod message;
pub mod net_info;
pub mod status;

use self::{data::Data, net_info::NetInfo, status::Status};
use crate::{
    config::agent::{AgentConfig, CollectorAddr, HttpConfig},
    error::{Error, ErrorKind},
};
use message::Message;
use std::{
    thread,
    time::{Duration, Instant},
};
use tendermint::{net, rpc};

/// Default interval at which to poll a Tendermint node
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Default interval at which to provide a full node status report
pub const DEFAULT_FULL_REPORT_INTERVAL: Duration = Duration::from_secs(60);

/// Tendermint node monitor which performs various checks against the RPC
/// interface or other signal sources.
pub struct Monitor {
    /// RPC client
    rpc_client: rpc::Client,

    /// Node status monitor
    status: Status,

    /// Network information monitor
    net_info: NetInfo,

    /// Database directory monitor
    data: Data,

    /// Interval at which we poll the node
    poll_interval: Duration,

    /// Interval after which a full report of the current state is made
    full_report_interval: Duration,

    /// Last time a full report was made
    last_full_report: Instant,

    /// Collector address
    collector_addr: CollectorAddr,
}

impl Monitor {
    /// Create a new `Monitor`
    pub fn new(agent_config: &AgentConfig) -> Result<Self, Error> {
        let home_dir = &agent_config.node_home;
        let node_config = agent_config.load_tendermint_config()?;
        let rpc_client = rpc::Client::new(&node_config.rpc.laddr)?;
        let status = Status::new(&rpc_client)?;
        let data = Data::new(home_dir.join(&node_config.db_dir));
        let net_info = NetInfo::new(
            node_config.p2p.persistent_peers.clone(),
            node_config.p2p.private_peer_ids.clone(),
        );

        Ok(Self {
            rpc_client,
            status,
            net_info,
            data,
            poll_interval: DEFAULT_POLL_INTERVAL,
            full_report_interval: DEFAULT_FULL_REPORT_INTERVAL,
            last_full_report: Instant::now() - DEFAULT_FULL_REPORT_INTERVAL,
            collector_addr: agent_config.collector.clone(),
        })
    }

    /// Run the monitor
    pub fn run(&mut self) {
        loop {
            match self.poll() {
                Ok(msg) => {
                    if let Some(env) =
                        message::Envelope::new(self.status.node.network, self.status.node.id, msg)
                    {
                        self.report(env).unwrap();
                    }
                }
                Err(e) => {
                    status_err!("error polling node: {}", e);
                    break;
                }
            }

            thread::sleep(self.poll_interval);
        }
    }

    /// Poll the node, collecting messages about events we're interested in
    fn poll(&mut self) -> Result<Vec<Message>, Error> {
        let force = self.should_force();

        let mut messages = vec![];
        messages.extend(self.status.update(&self.rpc_client, force)?);
        messages.extend(self.net_info.update(&self.rpc_client, force)?);
        messages.extend(self.data.update(force)?);
        Ok(messages)
    }

    /// Determine if we need to force updates in order to generate a full report
    fn should_force(&mut self) -> bool {
        if self.last_full_report.elapsed() >= self.full_report_interval {
            self.last_full_report = Instant::now();
            true
        } else {
            false
        }
    }

    fn report(&self, msg: message::Envelope) -> Result<(), Error> {
        let url = match &self.collector_addr {
            CollectorAddr::Http(HttpConfig {
                addr: net::Address::Tcp { host, port, .. },
            }) => format!("http://{}:{}/collector", host, port),
            other => fail!(ErrorKind::Config, "unsupported collector: {:?}", other),
        };

        let client = reqwest::Client::new();
        let res = client
            .post(&url)
            .body(msg.to_json())
            .send()
            .map_err(|e| err!(ErrorKind::Report, "{}", e))?;

        res.error_for_status()
            .map_err(|e| err!(ErrorKind::Report, "{}", e))?;
        Ok(())
    }
}
