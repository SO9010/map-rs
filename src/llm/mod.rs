//! # LLM Integration Module
//!
//! This module provides Large Language Model (LLM) integration for AI-powered
//! analysis and insights of geographic and map data.
//!
//! ## Purpose
//! - Connect to OpenRouter API for access to various LLM providers
//! - Analyze geographic data and provide natural language insights
//! - Enable conversational interaction with map data
//! - Generate automated reports and summaries of spatial analysis
//!
//! ## Sub-modules
//! - `client`: OpenRouter API client implementation
//! - `openrouter_types`: Request/response data structures for API communication
//!
//! ## Key Features
//! - Support for multiple LLM providers through OpenRouter
//! - Contextual analysis of map features and spatial relationships
//! - Natural language querying of geographic data
//! - Integration with workspace data for comprehensive analysis
//! - Secure API key management and authentication
//!
//! ## Usage
//! The LLM client can analyze map features, answer questions about geographic data,
//! and provide insights based on spatial relationships and attribute data.

mod client;
mod openrouter_types;

use std::fmt::Display;

pub use openrouter_types::*;
use serde::{Deserialize, Serialize};
use ureq::{Agent, unversioned::transport::time::Duration};

use crate::llm::client::LLM_PROMPT;

#[derive(Clone, Serialize)]
pub struct OpenrouterClient {
    pub token: Option<String>,
    #[serde(skip)]
    pub agent: Agent,
    pub url: String,
    pub prompt: String,
}

impl Default for OpenrouterClient {
    fn default() -> Self {
        let config = Agent::config_builder().build();
        let agent: Agent = config.into();
        OpenrouterClient {
            agent,
            url: String::new(),
            token: None,
            prompt: LLM_PROMPT.to_string(),
        }
    }
}

impl OpenrouterClient {
    pub fn new(url: impl Display, token: Option<String>) -> Self {
        let config = Agent::config_builder().build();
        let agent: Agent = config.into();
        OpenrouterClient {
            agent,
            url: url.to_string(),
            token,
            prompt: LLM_PROMPT.to_string(),
        }
    }

    pub fn set_url(&mut self, url: impl Display) {
        self.url = url.to_string();
    }

    pub fn set_token(&mut self, token: impl Display) {
        self.token = Some(token.to_string());
    }
}
