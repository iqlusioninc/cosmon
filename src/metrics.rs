//!Module to process data into metrics for statsd. Mostly for pipeing to datadog.

use crate::error::Error;
use cadence::prelude::*;
use cadence::{BufferedUdpMetricSink, StatsdClient, DEFAULT_PORT};
use relayer_modules::ics04_channel::events as ChannelEvents;
use std::net::UdpSocket;
use tendermint::chain;
/// Send Statd metrics over UDP
#[derive(Debug)]
pub struct Metrics {
    client: StatsdClient,
}

impl Metrics {
    /// Create a new metrics client
    pub fn new(host: &str) -> Result<Metrics, Error> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        let host = (host, DEFAULT_PORT);
        let sink = BufferedUdpMetricSink::from(host, socket).unwrap();
        let client = StatsdClient::from_sink("sagan", sink);

        Ok(Self { client })
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

        self.client.incr(
            format!(
                "packer_send.{}.{}.{}.{}.{}",
                chain, src_channel, src_port, dst_channel, dst_port
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
        self.client.incr(
            format!(
                "packet_recieve.{}.{}.{}.{}.{}",
                chain, src_channel, src_port, dst_channel, dst_port
            )
            .as_ref(),
        )?;
        Ok(())
    }
}
