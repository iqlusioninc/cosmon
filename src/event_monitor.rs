//!
use crate::{
    config::agent::{AgentConfig, CollectorAddr, HttpConfig},
    error::{Error, ErrorKind},
    message::{Envelope, Message},
};
use tendermint::{chain, net, node, rpc::event_listener, Error as TMError};

use abscissa_core::prelude::trace;
use tokio::sync::mpsc::{Receiver, Sender};

use relayer_modules::events::IBCEvent;

use tokio::time::delay_for;

use std::time::Duration;

/// Connect to a tendermint node and recieve push events over a websocket and filter them for the collector.
pub struct EventMonitor {
    /// Websocket to collect events from
    event_listener: event_listener::EventListener,
    /// Channel endpoint for the agent to push events to
    event_out_queue: Sender<Vec<IBCEvent>>,

    // Node Address for reconnection
    node_addr: net::Address,
}

impl EventMonitor {
    /// Constructor for the event listener. Connect to node and subscribe to the queries.
    pub async fn new(
        agent_config: &AgentConfig,
        event_out_queue: Sender<Vec<IBCEvent>>,
    ) -> Result<Self, Error> {
        match agent_config.load_tendermint_config()? {
            Some(node_config) => {
                let mut event_listener =
                    event_listener::EventListener::connect(node_config.rpc.laddr.clone()).await?;
                    event_listener.subscribe(event_listener::EventSubscription::TransactionSubscription).await?;
                
                Ok(EventMonitor {
                    event_listener,
                    event_out_queue,
                    node_addr: node_config.rpc.laddr.clone(),
                })
            }
            None => {
                let mut event_listener =
                    event_listener::EventListener::connect(agent_config.rpc.clone()).await?;
                    event_listener.subscribe(event_listener::EventSubscription::TransactionSubscription).await?;
                
                Ok(EventMonitor {
                    event_listener,
                    event_out_queue,
                    node_addr: agent_config.rpc.clone(),
                })
            }
        }
    }
    /// Listener loop for push events and handle a recconection attempt on error
    pub async fn run(&mut self) {
        status_ok!("Event Listener Connected", format!("{:?}", self.node_addr));
        loop {
            match self.collect_events().await {
                Ok(..) => continue,
                Err(err) => {
                    trace!("Web socket error: {}", err);
                    // Try to reconnect
                    match event_listener::EventListener::connect(self.node_addr.clone()).await {
                        Ok(event_listener) => {
                            trace!("Reconnected");
                            self.event_listener = event_listener;
                                match self.event_listener.subscribe(event_listener::EventSubscription::TransactionSubscription).await {
                                    Ok(..) => continue,
                                    Err(err) => {
                                        trace!("Error on recreating subscribptions {}", err);
                                        delay_for(Duration::from_millis(500)).await
                                    }
                                };
                            
                        }
                        Err(err) => {
                            trace!("Error on recconnection from{}", err);
                        }
                    }
                }
            }
        }
    }

    /// get and type an event
    pub async fn collect_events(&mut self) -> Result<(), TMError> {
        let raw_event = self.event_listener.get_event().await?;
        let events = IBCEvent::get_all_events(raw_event);
        self.event_out_queue.send(events).await?;
        Ok(())
    }
}
/// The Event Reporter runs concurrently with the Event Listener and takes pushed events sends to the collector
pub struct EventReporter {
    ///Channel endpoint for the report to the collector
    event_rx_queue: Receiver<Vec<IBCEvent>>,

    ///Address for collector
    collector_addr: CollectorAddr,

    ///ID of the node
    node: node::Id,

    ///ID of the chain
    chain: chain::Id,
}

impl EventReporter {
    /// Constructor for the Event Reporter
    pub fn new(
        agent_config: &AgentConfig,
        event_rx_queue: Receiver<Vec<IBCEvent>>,
        node: node::Id,
        chain: chain::Id,
    ) -> Self {
        Self {
            event_rx_queue,
            collector_addr: agent_config.collector.clone(),
            node,
            chain,
        }
    }

    ///Reporter loop that runs concurrently with the listener loop
    pub async fn run(&mut self) {
        loop {
            let events = self.event_rx_queue.recv().await.unwrap();

            for event in events {
                if let Some(env) = Envelope::new(self.chain, self.node, vec![Message::from(event)])
                {
                    match self.report(env).await {
                        Ok(_) => {}
                        Err(err) => {
                            status_err!("error reporting events node: {}", err);
                            delay_for(Duration::from_millis(500)).await
                        }
                    }
                }
            }
        }
    }

    async fn report(&self, msg: Envelope) -> Result<(), Error> {
        let url = match &self.collector_addr {
            CollectorAddr::Http(HttpConfig {
                addr: net::Address::Tcp { host, port, .. },
            }) => format!("http://{}:{}/collector", host, port),
            other => fail!(ErrorKind::ConfigError, "unsupported collector: {:?}", other),
        };

        let client = reqwest::Client::new();
        let res = client
            .post(&url)
            .body(msg.to_json())
            .send()
            .await
            .map_err(|e| format_err!(ErrorKind::ReportError, "{}", e))?;

        res.error_for_status()
            .map_err(|e| format_err!(ErrorKind::ReportError, "{}", e))?;
        Ok(())
    }
}
