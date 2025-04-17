use bevy::{prelude::*, utils::HashSet};
use bevy_map_viewer::{Coord, TileMapResources};
use rstar::{AABB, RTree, RTreeObject};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    Workspace, WorkspaceData, WorkspacePlugin, WorkspaceRequest,
    renderer::render_workspace_requests,
    worker::{cleanup_tasks, process_requests},
};

impl Plugin for WorkspacePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Workspace::default())
            .add_systems(FixedUpdate, (process_requests, cleanup_tasks))
            .add_systems(Update, render_workspace_requests);
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
        self.worker.queue_request(request.clone());
    }

    pub fn get_unrendered_requests(&self) -> Vec<WorkspaceRequest> {
        let loaded_requests = self.loaded_requests.lock().unwrap();
        loaded_requests
            .values()
            .filter_map(|(request, rendered)| {
                if !rendered {
                    Some(request.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn mark_as_rendered(&self, request_id: &str) {
        let mut loaded_requests = self.loaded_requests.lock().unwrap();
        if let Some((_request, rendered)) = loaded_requests.get_mut(request_id) {
            *rendered = true; // Mark as rendered
        }
    }
}

impl WorkspaceData {
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

    pub fn get_requests(&self) -> Option<HashSet<String>> {
        self.requests.clone()
    }
    pub fn add_request(&mut self, request_id: String) {
        self.requests
            .get_or_insert_with(HashSet::new)
            .insert(request_id);
        self.last_modified = chrono::Utc::now().timestamp();
    }
    pub fn remove_request(&mut self, request_id: String) {
        if let Some(requests) = &mut self.requests {
            requests.remove(&request_id);
        }
        self.last_modified = chrono::Utc::now().timestamp();
    }
    pub fn clear_requests(&mut self) {
        self.requests = Some(HashSet::new());
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
}

impl WorkspaceRequest {
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_layer(&self) -> u32 {
        self.layer
    }

    pub fn get_request(&self) -> RequestType {
        self.request.clone()
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
}

impl std::fmt::Debug for RequestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestType::OpenMeteoRequest(_) => write!(f, "OpenMeteoRequest"),
            RequestType::OverpassTurboRequest(_) => write!(f, "OverpassTurboRequest"),
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
            last_query_date: chrono::Utc::now().timestamp(),
        }
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
            requests: None,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectionType {
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
    pub focused_area: Option<WorkspaceData>,
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
            focused_area: None,
            areas: RTree::new(),
            unfinished_selection: None,
            respawn: false,
        }
    }

    pub fn add(&mut self, selection: WorkspaceData) {
        self.areas.insert(selection);
    }
}

#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
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
}

/// These implementations are for the RTreeObject trait.
impl RTreeObject for Selection {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        match self.selection_type {
            SelectionType::RECTANGLE => {
                return AABB::from_corners(
                    [
                        self.start.unwrap().lat.into(),
                        self.start.unwrap().long as f64,
                    ],
                    [self.end.unwrap().lat as f64, self.end.unwrap().long as f64],
                );
            }
            SelectionType::CIRCLE => {
                if let (Some(center), Some(edge)) = (self.start, self.end) {
                    let radius = center.to_vec2().distance(edge.to_vec2());

                    let lat_radius = radius as f64; // Approximation - adjust if needed
                    let long_radius = radius as f64;

                    return AABB::from_corners(
                        [
                            center.lat as f64 - lat_radius,
                            center.long as f64 - long_radius,
                        ],
                        [
                            center.lat as f64 + lat_radius,
                            center.long as f64 + long_radius,
                        ],
                    );
                }
                return AABB::from_corners([0.0, 0.0], [0.0, 0.0]);
            }
            SelectionType::POLYGON => {
                let mut min = [f64::MAX, f64::MAX];
                let mut max = [f64::MIN, f64::MIN];
                for point in self.points.as_ref().unwrap() {
                    if point.long < min[0] as f32 {
                        min[0] = point.long as f64;
                    }
                    if point.lat < min[1] as f32 {
                        min[1] = point.lat as f64;
                    }
                    if point.long > max[0] as f32 {
                        max[0] = point.long as f64;
                    }
                    if point.lat > max[1] as f32 {
                        max[1] = point.lat as f64;
                    }
                }
                return AABB::from_corners(min, max);
            }
            _ => AABB::from_corners([0.0, 0.0], [0.0, 0.0]),
        };
        AABB::from_corners([0.0, 0.0], [0.0, 0.0])
    }
}
