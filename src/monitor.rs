//! Tendermint node monitoring support

pub mod data;
pub mod message;
pub mod net_info;
pub mod status;

use self::{data::Data, net_info::NetInfo, status::Status};
use crate::error::Error;
use message::Message;
use std::{
    path::Path,
    thread,
    time::{Duration, Instant},
};
use tendermint::{config::TendermintConfig, rpc};

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
}

impl Monitor {
    /// Create a new `Monitor`
    pub fn new(home_dir: impl AsRef<Path>, config: &TendermintConfig) -> Result<Self, Error> {
        let rpc_client = rpc::Client::new(&config.rpc.laddr)?;
        let home_dir = home_dir.as_ref();
        let status = Status::new(&rpc_client)?;
        let data = Data::new(home_dir.join(&config.db_dir));
        let net_info = NetInfo::new(
            config.p2p.persistent_peers.clone(),
            config.p2p.private_peer_ids.clone(),
        );

        Ok(Self {
            rpc_client,
            status,
            net_info,
            data,
            poll_interval: DEFAULT_POLL_INTERVAL,
            full_report_interval: DEFAULT_FULL_REPORT_INTERVAL,
            last_full_report: Instant::now() - DEFAULT_FULL_REPORT_INTERVAL,
        })
    }

    /// Run the monitor
    pub fn run(&mut self) {
        loop {
            match self.poll() {
                Ok(msg) => {
                    if let Some(env) = message::Envelope::new(self.status.node.id, msg) {
                        println!("{}", env.to_json());
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
}
