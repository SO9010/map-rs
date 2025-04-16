mod client;
mod overpass_plugin;
mod overpass_types;
pub(crate) mod worker;

use std::{io::Error, time::Duration};

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

pub fn build_overpass_query_string(bounds: String, settings: Settings) -> Result<String, Error> {
    let mut query = String::default();
    let opening = "[out:json];";
    let closing = "\nout body geom;";

    for (category, key) in settings.get_true_keys_with_category() {
        if key == "n/a" {
            continue;
        } else if key == "*" {
            query.push_str(&format!(
                r#"
                    (
                    way["{}"]({});
                    node["{}"]({});
                    relation["{}"]({});
                    );
                    "#,
                category.to_lowercase(),
                bounds,
                category.to_lowercase(),
                bounds,
                category.to_lowercase(),
                bounds,
            ));
        } else {
            query.push_str(&format!(
                r#"
                    (
                    way["{}"="{}"]({});
                    node["{}"="{}"]({});
                    relation["{}"="{}"]({});
                    );
                    "#,
                category.to_lowercase(),
                key.to_lowercase(),
                bounds,
                category.to_lowercase(),
                key.to_lowercase(),
                bounds,
                category.to_lowercase(),
                key.to_lowercase(),
                bounds,
            ));
        }
    }

    if !query.is_empty() {
        query.insert_str(0, opening);
        query.push_str(closing);
        Ok(query)
    } else {
        Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "No valid settings provided",
        ))
    }
}
