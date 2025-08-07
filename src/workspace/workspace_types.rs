use std::{
    collections::{HashMap, HashSet},
    fs::read_dir,
};

use bevy::prelude::*;
use bevy_egui::EguiPreUpdateSet;
use bevy_map_viewer::{Coord, TileMapResources};
use rstar::{AABB, RTree, RTreeObject};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use uuid::Uuid;

use crate::{
    geojson::{MapFeature, get_data_from_string_osm},
    llm::Message,
    workspace::{ui::chat_box_ui, worker::load_workspaces},
};

use super::{
    Workspace, WorkspaceData, WorkspacePlugin, WorkspaceRequest,
    renderer::render_workspace_requests,
    ui::{ChatState, PersistentInfoWindows, item_info, workspace_actions_ui},
    worker::{cleanup_tasks, process_requests},
};

impl Plugin for WorkspacePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Workspace::default())
            .insert_resource(ChatState::default())
            .add_systems(FixedUpdate, (process_requests, cleanup_tasks))
            .add_systems(Update, render_workspace_requests)
            .add_systems(Startup, load_workspaces)
            .insert_resource(PersistentInfoWindows::default())
            .add_systems(
                Update,
                ((
                    workspace_actions_ui.after(EguiPreUpdateSet::InitContexts),
                    chat_box_ui.after(EguiPreUpdateSet::InitContexts),
                    item_info.after(EguiPreUpdateSet::InitContexts),
                ),),
            );
    }
}

// This will handle the saving and loading of workspace data.
impl Workspace {
    pub fn save_requests(&self) -> Result<(), std::io::Error> {
        for (_, request) in self.loaded_requests.lock().unwrap().iter() {
            if request.get_processed_data().size() != 0 {
                if !std::path::Path::new(&format!("{}.json", request.get_id())).exists() {
                    info!("Saving request: {}", request.get_id());
                    let mut file = File::create(format!("RQ_{}.json", request.get_id()))?;
                    file.write_all(serde_json::to_string(&request)?.as_bytes())?;
                }
            }
        }
        Ok(())
    }

    pub fn save_workspace(&self) -> Result<(), std::io::Error> {
        let workspace_data = self.workspace.clone().unwrap_or_default();
        let serialized_data = serde_json::to_string(&workspace_data)?;
        let mut file = File::create(format!("WS_{}.json", workspace_data.get_id()))?;
        file.write_all(serialized_data.as_bytes())?;
        Ok(())
    }

    pub fn load_workspace(&mut self) -> Result<(), std::io::Error> {
        let paths = read_dir("./").unwrap();
        for path in paths {
            if let Ok(path) = path {
                if path.path().extension().is_some_and(|ext| ext == "json") {
                    if path
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with("WS_")
                    {
                        info!("Loading workspace: {}", path.path().display());
                        let mut file = File::open(path.path())?;
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)?;
                        let workspace_data: WorkspaceData = serde_json::from_str(&contents)?;
                        self.loaded_workspace
                            .insert(workspace_data.get_id(), workspace_data);
                    }
                    if path
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with("RQ_")
                    {
                        info!("Loading request: {}", path.path().display());
                        let mut file = File::open(path.path())?;
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)?;
                        let request: WorkspaceRequest = serde_json::from_str(&contents)?;
                        self.loaded_requests
                            .lock()
                            .unwrap()
                            .insert(request.get_id(), request);
                    }
                }
            }
        }
        Ok(())
    }
}

impl Workspace {
    /// Processes a `WorkspaceRequest` and updates the workspace with the request data.
    ///
    /// # Arguments
    ///
    /// * `request` - A mutable `WorkspaceRequest` containing the request details.
    ///
    /// # Returns
    ///
    /// * `Ok(WorkspaceRequest)` - If the request is successfully processed and added to the workspace.
    /// * `Err(String)` - If there is an error during processing or if no workspace is found.
    ///
    /// # Behavior
    ///
    /// - Handles different types of requests (`OverpassTurboRequest`, `OpenMeteoRequest`).
    /// - Updates the workspace with the request data and serializes the request for storage.
    /// - Adds the request ID to the workspace's list of requests.
    pub fn process_request(&mut self, request: WorkspaceRequest) {
        info!("Processing request: {:?}", request);
        self.worker.queue_request(request.clone());
    }

    pub fn get_unrendered_requests(&self) -> Vec<WorkspaceRequest> {
        let loaded_requests = self.loaded_requests.lock().unwrap();
        let mut rendered_requests = Vec::new();
        for (_, request) in loaded_requests.iter() {
            if request.get_processed_data().size() == 0 {
                rendered_requests.push(request.clone());
            }
        }
        rendered_requests
    }

    pub fn get_requests(&self) -> Vec<WorkspaceRequest> {
        let loaded_requests = self.loaded_requests.lock().unwrap();
        let mut rendered_requests = Vec::new();
        if let Some(workspace) = &self.workspace {
            for j in workspace.get_requests().iter() {
                if let Some(request) = loaded_requests.get(j) {
                    rendered_requests.push(request.clone());
                }
            }
        }
        rendered_requests
    }
    pub fn get_rendered_requests(&self) -> Vec<WorkspaceRequest> {
        let loaded_requests = self.loaded_requests.lock().unwrap();
        let mut rendered_requests = Vec::new();
        for (_, request) in loaded_requests.iter() {
            if request.get_processed_data().size() != 0 {
                rendered_requests.push(request.clone());
            }
        }
        if let Some(workspace) = &self.workspace {
            for j in workspace.get_requests().iter() {
                if let Some(request) = loaded_requests.get(j) {
                    rendered_requests.push(request.clone());
                }
            }
        }
        rendered_requests
    }
}

impl WorkspaceData {
    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(Message {
            role: role.to_string(),
            content: content.to_string(),
            refusal: serde_json::Value::Null,
            reasoning: serde_json::Value::Null,
        });
    }

    pub fn clear_history(&mut self) {
        self.messages.clear();
    }
}

impl WorkspaceData {
    pub fn get_color_properties(&self) -> HashMap<(String, serde_json::Value), Srgba> {
        self.properties.clone()
    }
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_selection(&self) -> Selection {
        self.selection.clone()
    }

    pub fn get_creation_date(&self) -> i64 {
        self.creation_date
    }

    pub fn get_last_modified(&self) -> i64 {
        self.last_modified
    }

    pub fn get_requests(&self) -> HashSet<String> {
        self.requests.clone()
    }
    pub fn add_request(&mut self, request_id: String) {
        self.requests.insert(request_id);
        self.last_modified = chrono::Utc::now().timestamp();
    }
    pub fn remove_request(&mut self, request_id: String) {
        self.requests.remove(&request_id);
        self.last_modified = chrono::Utc::now().timestamp();
    }
    pub fn clear_requests(&mut self) {
        self.requests = HashSet::new();
        self.last_modified = chrono::Utc::now().timestamp();
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.last_modified = chrono::Utc::now().timestamp();
    }
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = selection;
        self.last_modified = chrono::Utc::now().timestamp();
    }
    pub fn get_area(&self) -> (f32, bevy_map_viewer::DistanceType) {
        match self.selection.selection_type {
            SelectionType::RECTANGLE => {
                if let (Some(start), Some(end)) = (self.selection.start, self.selection.end) {
                    let top =
                        Coord::new(start.lat, start.long).distance(&Coord::new(end.lat, end.long));
                    let left = Coord::new(start.lat, start.long)
                        .distance(&Coord::new(start.lat, end.long));
                    return (top.0 * left.0, left.1);
                }
                (0.0, bevy_map_viewer::DistanceType::Km)
            }
            SelectionType::CIRCLE => {
                if let (Some(center), Some(edge)) = (self.selection.start, self.selection.end) {
                    let radius = center.distance(&edge);
                    return (std::f32::consts::PI * radius.0 * radius.0, radius.1);
                }
                (0.0, bevy_map_viewer::DistanceType::Km)
            }
            SelectionType::POLYGON => {
                if let Some(_points) = &self.selection.points {
                    return (0.0, bevy_map_viewer::DistanceType::Km);
                    // TODO: Implement polygon area calculation
                }
                (0.0, bevy_map_viewer::DistanceType::Km)
            }
            _ => (0.0, bevy_map_viewer::DistanceType::Km),
        }
    }
}

impl WorkspaceRequest {
    pub fn get_visible(&self) -> bool {
        self.visible
    }
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_layer(&self) -> u32 {
        self.layer
    }

    pub fn get_request(&self) -> RequestType {
        self.request.clone()
    }

    pub fn process_request(&mut self) {
        match self.get_request() {
            crate::workspace::RequestType::OverpassTurboRequest(_) => {
                if let Ok(data) =
                    get_data_from_string_osm(&String::from_utf8(self.raw_data.clone()).unwrap())
                {
                    for feature in data {
                        self.processed_data.insert(feature.clone());
                    }
                }
            }
            crate::workspace::RequestType::OpenMeteoRequest(_) => {}
            crate::workspace::RequestType::OpenRouterRequest() => {}
        }
    }

    pub fn get_raw_data(&self) -> Vec<u8> {
        self.raw_data.clone()
    }

    pub fn get_last_query_date(&self) -> i64 {
        self.last_query_date
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OpenMeteoRequest {
    air_quality: openmeteo_rs_ureq::air_quality::AirQualityRequest,
    climate_change: openmeteo_rs_ureq::climate_change::ClimateChangeRequest,
    elevation: openmeteo_rs_ureq::elevation::ElevationRequest,
    ensemble_weather: openmeteo_rs_ureq::ensemble_weather::EnsembleWeatherRequest,
    flood: openmeteo_rs_ureq::flood::FloodRequest,
    geocoding: openmeteo_rs_ureq::geocoding::GeocodingRequest,
    historical_weather: openmeteo_rs_ureq::historical_weather::HistoricalWeatherRequest,
    marine_weather: openmeteo_rs_ureq::marine_weather::MarineWeatherRequest,
    satellite_radiation: openmeteo_rs_ureq::satellite_radiation::SatelliteRadiationRequest,
    weather: openmeteo_rs_ureq::weather::WeatherRequest,
}

#[derive(Clone, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum RequestType {
    // If we want to add more requests we can just add them here.
    OpenMeteoRequest(OpenMeteoRequest),
    OverpassTurboRequest(String),
    OpenRouterRequest(),
}

impl std::fmt::Debug for RequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestType::OpenMeteoRequest(_) => write!(f, "OpenMeteoRequest"),
            RequestType::OverpassTurboRequest(_) => write!(f, "OverpassTurboRequest"),
            RequestType::OpenRouterRequest() => write!(f, "OpenRouterRequest"),
        }
    }
}

impl WorkspaceRequest {
    /// Creates a new `WorkspaceRequest` instance.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the request.
    /// * `layer` - The layer number associated with the request.
    /// * `request` - The type of request being made (e.g., `OpenMeteoRequest`, `OverpassTurboRequest`).
    /// * `raw_data` - The raw data associated with the request, typically as a byte vector.
    ///
    /// # Returns
    ///
    /// A new instance of `WorkspaceRequest` with the provided parameters.
    ///
    /// The `last_query_date` is automatically set to the current UTC timestamp.
    pub fn new(id: String, layer: u32, request: RequestType, raw_data: Vec<u8>) -> Self {
        Self {
            id,
            layer,
            visible: true,
            request,
            raw_data,
            processed_data: RTree::new(),
            last_query_date: chrono::Utc::now().timestamp(),
        }
    }

    pub fn get_processed_data(&self) -> RTree<MapFeature> {
        self.processed_data.clone()
    }
}

impl WorkspaceData {
    pub fn new(name: String, selection: Selection) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            selection,
            creation_date: chrono::Utc::now().timestamp(),
            last_modified: chrono::Utc::now().timestamp(),
            requests: HashSet::new(),
            properties: HashMap::new(),
            messages: Vec::new(),
        }
    }
    pub fn empty() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "Empty".to_string(),
            selection: Selection::empty(),
            creation_date: chrono::Utc::now().timestamp(),
            last_modified: chrono::Utc::now().timestamp(),
            requests: HashSet::new(),
            properties: HashMap::new(),
            messages: Vec::new(),
        }
    }
}

impl RTreeObject for WorkspaceData {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.selection.envelope()
    }
}

/// The goal for this module is to provide a way to select a region of the map, to be able to select featurtes in that region.
/// For example someone should be able to select an eare for turbo overpass data to be downloaded.
/// Or this area can be selected to modify how the map looks in that area.
/// Or this will be used for the user to select their work space area, with in this the data will be permentanly stored and the user can modify it.
/// When someone selects something it would be cool to make it sticky so someone has to pull further than a certian amount to leave the workspace.
/// We could use some movment smoothing.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum SelectionType {
    #[default]
    NONE,
    RECTANGLE,
    POLYGON,
    CIRCLE,
}

impl SelectionType {
    pub fn iterate(&mut self) {
        match self {
            SelectionType::NONE => *self = SelectionType::RECTANGLE,
            SelectionType::RECTANGLE => *self = SelectionType::POLYGON,
            SelectionType::POLYGON => *self = SelectionType::CIRCLE,
            SelectionType::CIRCLE => *self = SelectionType::RECTANGLE,
        }
    }
}

pub struct SelectionAreas {
    pub areas: RTree<WorkspaceData>,
    pub unfinished_selection: Option<Selection>,
    pub respawn: bool,
}

impl Default for SelectionAreas {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectionAreas {
    pub fn new() -> Self {
        Self {
            areas: RTree::new(),
            unfinished_selection: None,
            respawn: false,
        }
    }

    pub fn add(&mut self, selection: WorkspaceData) {
        self.areas.insert(selection);
    }
}

#[derive(Component, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Selection {
    pub selection_type: SelectionType,
    pub start: Option<Coord>,
    pub end: Option<Coord>,
    pub points: Option<Vec<Coord>>,
}

impl Selection {
    pub fn get_in_world_space(&self, tile_map_resources: TileMapResources) -> Vec<Vec2> {
        if self.points.is_some() {
            let mut new_points = Vec::new();
            for point in self.points.as_ref().unwrap() {
                new_points.push(point.to_game_coords(tile_map_resources.clone()));
            }
            new_points
        } else {
            let mut new_points = Vec::new();
            if self.start.is_some() {
                new_points.push(
                    self.start
                        .unwrap()
                        .to_game_coords(tile_map_resources.clone()),
                );
            }
            if self.end.is_some() {
                new_points.push(self.end.unwrap().to_game_coords(tile_map_resources.clone()));
            }
            new_points
        }
    }
}

/// These implementations are for constructors.
impl Selection {
    pub fn new(selection_type: SelectionType, start: Coord, end: Coord) -> Self {
        Self {
            selection_type,
            start: Some(start),
            end: Some(end),
            points: None,
        }
    }

    pub fn new_poly(selection_type: SelectionType, start: Coord) -> Self {
        Self {
            selection_type,
            start: None,
            end: None,
            points: Some(vec![start]),
        }
    }

    pub fn empty() -> Self {
        Self {
            selection_type: SelectionType::NONE,
            start: None,
            end: None,
            points: None,
        }
    }
}

/// These implementations are for the RTreeObject trait.
impl RTreeObject for Selection {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        match self.selection_type {
            SelectionType::RECTANGLE => {
                if let (Some(start), Some(end)) = (self.start, self.end) {
                    return AABB::from_corners(
                        [start.lat as f64, start.long as f64],
                        [end.lat as f64, end.long as f64],
                    );
                }
                AABB::from_corners([0.0, 0.0], [0.0, 0.0])
            }
            SelectionType::CIRCLE => {
                if let (Some(center), Some(edge)) = (self.start, self.end) {
                    let radius = center.to_vec2().distance(edge.to_vec2());

                    return AABB::from_corners(
                        [
                            center.lat as f64 - radius as f64,  // Minimum latitude
                            center.long as f64 - radius as f64, // Minimum longitude
                        ],
                        [
                            center.lat as f64 + radius as f64,  // Maximum latitude
                            center.long as f64 + radius as f64, // Maximum longitude
                        ],
                    );
                }
                AABB::from_corners([0.0, 0.0], [0.0, 0.0])
            }
            SelectionType::POLYGON => {
                if let Some(points) = &self.points {
                    let mut min = [f64::MAX, f64::MAX];
                    let mut max = [f64::MIN, f64::MIN];

                    for point in points {
                        min[0] = min[0].min(point.lat as f64);
                        min[1] = min[1].min(point.long as f64);
                        max[0] = max[0].max(point.lat as f64);
                        max[1] = max[1].max(point.long as f64);
                    }

                    return AABB::from_corners(min, max);
                }
                AABB::from_corners([0.0, 0.0], [0.0, 0.0])
            }
            _ => AABB::from_corners([0.0, 0.0], [0.0, 0.0]),
        }
    }
}
