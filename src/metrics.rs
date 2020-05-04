//!Module to process data into metrics for statsd. Mostly for pipeing to datadog.

use crate::error::Error;
use cadence::prelude::*;
use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
use relayer_modules::ics02_client::events as ClientEvents;
use relayer_modules::ics04_channel::events as ChannelEvents;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::SystemTime;
use tendermint::chain;

/// Send Statd metrics over UDP
#[derive(Debug)]
pub struct Metrics {
    client: StatsdClient,

    /// Metric Prefix
    pub prefix: String,

    /// Map from Channel ID to team
    pub teamchannels: Option<HashMap<String, String>>,

    /// Map from Address to team
    pub teamaddresses: Option<HashMap<String, String>>,
}
impl Metrics {
    /// Create a new metrics client
    pub fn new(
        host: &str,
        prefix: String,
        teamchannels: Option<HashMap<String, String>>,
        teamaddresses: Option<HashMap<String, String>>,
    ) -> Result<Metrics, Error> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        let host = (host, DEFAULT_PORT);
        let sink = UdpMetricSink::from(host, socket).unwrap();
        let client = StatsdClient::from_sink("sagan", sink);
        client
            .time(
                &format!("{}.collector.start", &prefix),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            )
            .unwrap();
        Ok(Self {
            prefix,
            client,
            teamchannels,
            teamaddresses,
        })
    }
    ///heartbeat metric
    pub fn heartbeat(&mut self) {
        self.client
            .incr(&format!("{}.heartbeat", self.prefix))
            .unwrap();
    }

    /// Send a metric for each packet send event
    pub fn packet_send_event(
        &mut self,
        chain: chain::Id,
        event: ChannelEvents::SendPacket,
    ) -> Result<(), Error> {
        let missing_src_channel = "packet_src_channel_missing".to_owned();
        let src_channel: &String = event
            .data
            .get("packet_src_channel")
            .map(|data| data.get(0))
            .unwrap_or(Some(&missing_src_channel))
            .unwrap();
        let missing_src_port = "packet_src_port_missing".to_owned();
        let src_port: &String = event
            .data
            .get("packet_src_port")
            .map(|data| data.get(0))
            .unwrap_or(Some(&missing_src_port))
            .unwrap();
        let missing_dst_channel = "packet_dst_channel_missing".to_owned();
        let dst_channel: &String = event
            .data
            .get("packet_dst_channel")
            .map(|data| data.get(0))
            .unwrap_or(Some(&missing_dst_channel))
            .unwrap();
        let missing_dst_port = "packet_dst_port_missing".to_owned();
        let dst_port: &String = event
            .data
            .get("packet_dst_port")
            .map(|data| data.get(0))
            .unwrap_or(Some(&missing_dst_port))
            .unwrap();

        let missing_sender = "sender_missing".to_owned();
        let message_sender = event
            .data
            .get("sender")
            .map(|data| data.get(0))
            .unwrap_or(Some(&missing_sender))
            .unwrap();

        self.client.incr(
            format!(
                "{}.packer_send.{}.{}.{}.{}.{}.{}",
                self.prefix, message_sender, chain, src_channel, src_port, dst_channel, dst_port
            )
            .as_ref(),
        )?;
        Ok(())
    }

    ///Send a metric for packet recieve event
    pub fn packet_recieve_event(
        &mut self,
        chain: chain::Id,
        event: ChannelEvents::RecievePacket,
    ) -> Result<(), Error> {
        let missing_src_channel = "packet_src_channel_missing".to_owned();
        let src_channel: &String = event
            .data
            .get("packet_src_channel")
            .map(|data| data.get(0))
            .unwrap_or(Some(&missing_src_channel))
            .unwrap();
        let missing_src_port = "packet_src_port_missing".to_owned();
        let src_port: &String = event
            .data
            .get("packet_src_port")
            .map(|data| data.get(0))
            .unwrap_or(Some(&missing_src_port))
            .unwrap();
        let missing_dst_channel = "packet_dst_channel_missing".to_owned();
        let dst_channel: &String = event
            .data
            .get("packet_dst_channel")
            .map(|data| data.get(0))
            .unwrap_or(Some(&missing_dst_channel))
            .unwrap();
        let missing_dst_port = "packet_dst_port_missing".to_owned();
        let dst_port: &String = event
            .data
            .get("packet_dst_port")
            .map(|data| data.get(0))
            .unwrap_or(Some(&missing_dst_port))
            .unwrap();
        let missing_sender = "sender_missing".to_owned();
        let message_sender = event
            .data
            .get("sender")
            .map(|data| data.get(0))
            .unwrap_or(Some(&missing_sender))
            .unwrap();

        self.client.incr(
            format!(
                "{}.packet_recieve.{}.{}.{}.{}.{}.{}",
                self.prefix, chain, message_sender, src_channel, src_port, dst_channel, dst_port
            )
            .as_ref(),
        )?;
        Ok(())
    }
    ///Send a metric for update client event
    pub fn update_client_event(
        &mut self,
        chain: chain::Id,
        event: ClientEvents::UpdateClient,
    ) -> Result<(), Error> {
        let missing_client_id = "client_id_missing".to_owned();
        let client_id = event
            .data
            .get("client_id")
            .map(|data| data.get(0))
            .unwrap_or(Some(&missing_client_id))
            .unwrap();

        let missing_sender = "sender_missing".to_owned();
        let message_sender = event
            .data
            .get("sender")
            .map(|data| data.get(0))
            .unwrap_or(Some(&missing_sender))
            .unwrap();

        self.client.incr(
            format!(
                "{}.packet_recieve.{}.{}.{}",
                self.prefix, chain, message_sender, client_id
            )
            .as_ref(),
        )?;
        Ok(())
    }
}
