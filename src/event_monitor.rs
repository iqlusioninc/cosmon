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

/// Connect to a tendermint node and recieve push events over a websocket and filter them for the collector.
pub struct EventMonitor {
    /// Websocket to collect events from
    event_listener: event_listener::EventListener,
    /// Channel endpoint for the agent to push events to
    event_out_queue: Sender<Vec<IBCEvent>>,

    // Node Address for reconnection
    node_addr: net::Address,
    // Queries for recconnection
    event_queries: Vec<String>,
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
                for query in &agent_config.event_queries {
                    event_listener.subscribe(query).await?;
                }
                Ok(EventMonitor {
                    event_listener,
                    event_out_queue,
                    node_addr: node_config.rpc.laddr.clone(),
                    event_queries: agent_config.event_queries.clone(),
                })
            }
            None => {
                let mut event_listener =
                    event_listener::EventListener::connect(agent_config.rpc.clone()).await?;
                for query in &agent_config.event_queries {
                    event_listener.subscribe(query).await?;
                }
                Ok(EventMonitor {
                    event_listener,
                    event_out_queue,
                    node_addr: agent_config.rpc.clone(),
                    event_queries: agent_config.event_queries.clone(),
                })
            }
        }
    }
    /// Listener loop for push events and handle a recconection attempt on error
    pub async fn run(&mut self) {
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
                            for query in &self.event_queries {
                                match self.event_listener.subscribe(query).await {
                                    Ok(..) => continue,
                                    Err(err) => {
                                        trace!("Error on recreating subscribptions {}", err);
                                        panic!("Abort during recconnection");
                                    }
                                };
                            }
                        }
                        Err(err) => {
                            trace!("Error on recconnection from{}", err);
                            panic!("Abort on failed reconnection")
                        }
                    }
                }
            }
        }
    }

    /// get and type an event
    pub async fn collect_events(&mut self) -> Result<(), TMError> {
        let raw_event =self.event_listener.get_event().await?;
        match raw_event.clone() {
            event_listener::Event::JsonRPCTransctionResult{data}=>{dbg!(data);},
            _=>{},
        };
        let events = IBCEvent::get_all_events(raw_event);
        dbg!(&events);
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
                    self.report(env).await.unwrap();
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
