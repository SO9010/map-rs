mod client;
mod openrouter_types;

use std::fmt::Display;

pub use openrouter_types::*;
use ureq::{Agent, unversioned::transport::time::Duration};

#[derive(Clone)]
pub struct OpenrouterClient {
    pub token: Option<String>,
    pub agent: Agent,
    pub url: String,
}

impl OpenrouterClient {
    pub fn new(url: impl Display, token: Option<String>) -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(*Duration::from_secs(5)))
            .build();
        let agent: Agent = config.into();
        OpenrouterClient {
            agent,
            url: url.to_string(),
            token,
        }
    }

    pub fn set_url(&mut self, url: impl Display) {
        self.url = url.to_string();
    }

    pub fn set_token(&mut self, token: impl Display) {
        self.token = Some(token.to_string());
    }
}
