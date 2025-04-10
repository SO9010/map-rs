mod client;
mod overpass_plugin;
mod overpass_types;
pub(crate) mod worker;

use std::time::Duration;

pub use overpass_plugin::*;
pub use overpass_types::*;
use ureq::Agent;
pub use worker::*;

#[derive(Clone)]
pub struct OverpassClient {
    url: String,
    pub agent: Agent,
    pub bounds: String,
    pub settings: Settings,
}

impl Default for OverpassClient {
    fn default() -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(5)))
            .build();
        let agent: Agent = config.into();
        OverpassClient {
            agent,
            url: "https://overpass-api.de/api/interpreter".to_string(),
            bounds: String::new(),
            settings: Settings::default(),
        }
    }
}

impl OverpassClient {
    pub fn new(url: &str) -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(5)))
            .build();
        let agent: Agent = config.into();
        OverpassClient {
            agent,
            url: url.to_string(),
            bounds: String::new(),
            settings: Settings::default(),
        }
    }

    pub fn set_url(&mut self, url: &str) {
        self.url = url.to_string();
    }
}
