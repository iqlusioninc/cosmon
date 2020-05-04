//! `sagan.toml` Collector configuration settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tendermint::net;

/// Collector config settings from `sagan.toml`
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CollectorConfig {
    /// Address to bind to
    pub listen_addr: net::Address,

    /// Networks this collector is collecting information about
    pub networks: NetworkConfig,

    /// Host ip for the StatsD Deserver
    pub statsd: String,

    /// Prefix on metrics sent to statsd
    pub metrics_prefix: String,

    /// Team details
    pub teams: Option<Vec<Team>>,
}

impl CollectorConfig {
    ///Build look up indexes from the teams
    pub fn build_hashmaps(
        &self,
    ) -> Option<(
        HashMap<String, String>,
        HashMap<String, String>,
        HashMap<String, String>,
    )> {
        if let Some(ref teams) = self.teams {
            let mut address_to_team = HashMap::new();
            let mut channel_id_to_team = HashMap::new();
            let mut client_id_to_team = HashMap::new();

            for team in teams {
                address_to_team.insert(team.address.clone(), team.name.clone());
                channel_id_to_team.insert(team.channel_id.clone(), team.name.clone());
                client_id_to_team.insert(team.client_id.clone(), team.name.clone());
            }

            return Some((address_to_team, channel_id_to_team, client_id_to_team));
        }
        None
    }
}

/// Types of networks this collector is collecting information about
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NetworkConfig {
    /// Tendermint networks
    #[serde(default)]
    pub tendermint: Vec<tendermint::chain::Id>,
}

/// Team Details
#[derive(Clone, Debug, Deserialize, Serialize)]

pub struct Team {
    ///Team name
    pub name: String,
    /// Team Cosmos Address
    pub address: String,
    /// Team Channel Id
    pub channel_id: String,
    ///Team client_id
    pub client_id: String,
}
