mod client;
mod overpass_types;

pub use client::*;
pub use overpass_types::*;
use ureq::Agent;

#[derive(Clone)]
pub struct OverpassClient {
    url: String,
    pub agent: Agent,
    pub bounds: String,
    pub settings: Settings,
}
