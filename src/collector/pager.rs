//! Collector pager

use crate::{collector, config, prelude::*};
use datadog::{send_stream_event, StreamEvent};
use std::collections::BTreeMap;
use std::env;
use std::time::Duration;
use std::time::SystemTime;
use tokio::time;
use tower::{Service, ServiceExt};

/// The collector's [`Pager`] collects pageable events
pub struct Pager {
    /// Interval at which to poll
    poll_interval: Duration,
}

impl Pager {
    /// Initialize the pager from the config
    /// todo(shella): add pager config
    pub fn new(_config: &config::collector::Config) -> Result<Self, Error> {
        let _now = SystemTime::now();

        Ok(Self {
            poll_interval: Duration::from_secs(1),
        })
    }

    /// Route incoming requests.
    pub async fn run<S>(self, collector: &S)
    where
        S: Service<collector::Request, Response = collector::Response, Error = BoxError>
            + Send
            + Clone
            + 'static,
    {
        let mut interval = time::interval(self.poll_interval);

        loop {
            interval.tick().await;
            self.poll(collector.clone()).await;
            info!("waiting for {:?}", self.poll_interval);
        }
    }

    /// Poll sources. If a pageable event occurs, send event to Datadog then to Pagerduty.
    #[cfg_attr(not(feature = "mintscan"), allow(unused_variables))]
    async fn poll<S>(&self, mut collector: S)
    where
        S: Service<collector::Request, Response = collector::Response, Error = BoxError>
            + Send
            + Clone
            + 'static,
    {
        let mut interval = time::interval(self.poll_interval);
        loop {
            interval.tick().await;

            let response = collector
                .ready()
                .await
                .expect("collector not ready")
                .call(collector::Request::PagerEvents {})
                .await
                .expect("error sending poller info");

            let events = match response {
                collector::Response::PagerEvents(ev) => ev,
                _ => unreachable!("unexpected response: {:?}", response),
            };

            for event in events {
                dbg!(&event);
                let dd_api_key = env::var("DD_API_KEY").unwrap();
                let hostname = hostname::get().unwrap();
                let mut ddtags = BTreeMap::new();
                ddtags.insert("env".to_owned(), "staging".to_owned());
                let stream_event = StreamEvent {
                    aggregation_key: None,
                    alert_type: Some(datadog::AlertType::Error),
                    date_happened: Some(SystemTime::now()),
                    device_name: None,
                    hostname: Some(hostname.to_string_lossy().to_string()),
                    priority: Some(datadog::Priority::Normal),
                    related_event_id: None,
                    tags: Some(ddtags),
                    // Text field must contain @pagerduty to trigger alert
                    text: format!("@pagerduty cosmon event: {:?}", &event),
                    title: event,
                };

                // send stream event to datadog which forwards to pagerduty
                let stream_event = send_stream_event(&stream_event, dd_api_key).await;
                match stream_event {
                    Ok(()) => {
                        dbg!("event sent to datadog");
                    }
                    Err(_err) => {
                        warn!("unable to sent event to datadog");
                    }
                }
            }
        }
    }
}
