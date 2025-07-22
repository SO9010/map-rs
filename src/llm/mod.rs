mod client;
mod openrouter_types;

pub use client::*;
pub use openrouter_types::*;
use ureq::Agent;

#[derive(Clone)]
pub struct OpenrouterClient {
    pub token: Option<String>,
    pub agent: Agent,
}
