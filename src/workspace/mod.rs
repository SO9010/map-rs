use std::sync::{Arc, Mutex};

use bevy::{
    ecs::system::Resource,
    utils::{HashMap, HashSet},
};
use serde::{Deserialize, Serialize};
use worker::WorkspaceWorker;
pub use workspace_types::*;

use crate::overpass::OverpassClient;

mod renderer;
mod worker;
mod workspace_types;

pub struct WorkspacePlugin;

#[derive(Resource)]
pub struct Workspace {
    pub workspace: Option<WorkspaceData>,
    // (rendered, id)
    pub loaded_requests: Arc<Mutex<HashMap<String, (WorkspaceRequest, bool)>>>,
    pub worker: WorkspaceWorker,

    // Request Clients:
    pub overpass_agent: OverpassClient,
}

impl Default for Workspace {
    fn default() -> Self {
        Workspace {
            workspace: None,
            loaded_requests: Arc::new(Mutex::new(HashMap::new())),
            worker: WorkspaceWorker::new(4),
            overpass_agent: OverpassClient::new("https://overpass-api.de/api/interpreter"),
        }
    }
}
// Need a function which sits and renders the data from the requests. Recognise the data type, then decide what to do with it and how to display it to the user.
// Functions we want:
// WorkspaceWorker,
// WorkspaceRenderer,
// WorkspaceSettings,

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceData {
    id: String,
    name: String,
    selection: Selection,
    creation_date: i64,
    last_modified: i64,
    requests: Option<HashSet<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceRequest {
    id: String,
    layer: u32,
    visible: bool,
    request: RequestType,
    raw_data: Vec<u8>, // Raw data from the request maybe have this as a id list aswell...
    last_query_date: i64, // When the OSM data was fetched
}
