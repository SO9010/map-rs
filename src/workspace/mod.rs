//! # Workspace Management Module
//!
//! This module provides comprehensive workspace management functionality for
//! organizing, persisting, and analyzing geographic data and map configurations.
//!
//! ## Purpose
//! - Manage different map workspace configurations and data layers
//! - Provide data persistence and session management
//! - Handle background processing of geographic data requests
//! - Coordinate between different data sources and analysis tools
//! - Enable collaborative workspace sharing and management
//!
//! ## Sub-modules
//! - `commands`: Workspace operation commands and state management
//! - `renderer`: Workspace-specific rendering and visualization
//! - `ui`: User interface components for workspace interaction
//! - `worker`: Background task processing and data pipeline management
//! - `workspace_types`: Core data structures and plugin implementation
//!
//! ## Key Features
//! - Multi-layered data organization and management
//! - Persistent workspace storage and loading
//! - Background processing of API requests and data analysis
//! - Integration with multiple data sources (OSM, weather, environmental)
//! - Real-time data updates and synchronization
//! - Collaborative workspace features preparation
//!
//! ## Workspace Components
//! - Data layers with independent styling and visibility
//! - Analysis results and cached computations
//! - User annotations and custom features
//! - API integration settings and credentials

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use bevy::{color::Srgba, ecs::resource::Resource};
use rstar::RTree;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worker::WorkspaceWorker;
pub use workspace_types::*;

use crate::{
    geojson::MapFeature,
    llm::{Message, OpenrouterClient},
    overpass::OverpassClient,
};

mod commands;
mod renderer;
mod ui;
mod worker;
mod workspace_types;

pub struct WorkspacePlugin;

#[derive(Resource)]
pub struct Workspace {
    pub workspace: Option<WorkspaceData>,
    // (id, request)
    pub loaded_requests: Arc<Mutex<HashMap<String, WorkspaceRequest>>>,
    pub worker: WorkspaceWorker,

    // Request Clients:
    pub overpass_agent: OverpassClient,
    pub llm_agent: OpenrouterClient,
    // Add llm requests agent!
}

impl Default for Workspace {
    fn default() -> Self {
        Workspace {
            workspace: None,
            loaded_requests: Arc::new(Mutex::new(HashMap::new())),
            worker: WorkspaceWorker::new(4),
            overpass_agent: OverpassClient::new("https://overpass-api.de/api/interpreter"),
            llm_agent: OpenrouterClient::new("https://openrouter.ai/api/v1/chat/completions", None),
        }
    }
}
// Need a function which sits and renders the data from the requests. Recognise the data type, then decide what to do with it and how to display it to the user.
// Functions we want:
// WorkspaceWorker,
// WorkspaceRenderer,
// WorkspaceSettings,

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceData {
    id: String,
    name: String,
    selection: Selection,
    creation_date: i64,
    last_modified: i64,
    requests: HashSet<String>,
    properties: HashMap<(String, Value), Srgba>,
    messages: Vec<Message>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceRequest {
    id: String,
    layer: u32,
    visible: bool,
    request: RequestType,
    raw_data: Vec<u8>, // Raw data from the request maybe have this as a id list aswell...
    #[serde(skip)]
    processed_data: RTree<MapFeature>,
    last_query_date: i64, // When the OSM data was fetched
}
