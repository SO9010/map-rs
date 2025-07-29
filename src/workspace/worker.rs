use crate::workspace::ui::{ChatMessage, ChatState};
use crate::workspace::{RequestType, WorkspaceRequest};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_map_viewer::Coord;
use bevy_tasks::futures_lite::future;
use std::sync::{Arc, Mutex};

use super::Workspace;
#[derive(Default, Clone)]
pub struct WorkspaceWorker {
    /// A thread-safe queue of pending Overpass requests.
    pending_requests: Arc<Mutex<Vec<WorkspaceRequest>>>,
    /// The maximum number of concurrent tasks allowed.
    max_concurrent: usize,
    /// A counter to track the number of active tasks.
    active_tasks: Arc<Mutex<usize>>,
}

impl WorkspaceWorker {
    pub fn new(max_workers: usize) -> Self {
        WorkspaceWorker {
            pending_requests: Arc::new(Mutex::new(Vec::new())),
            max_concurrent: max_workers,
            active_tasks: Arc::new(Mutex::new(0)),
        }
    }

    /// Change this to take the workspace so then we can handle everthing in the workspace too.
    pub fn queue_request(&self, request: WorkspaceRequest) {
        let mut pending = self.pending_requests.lock().unwrap();
        pending.push(request);
    }
}

pub fn process_requests(
    mut commands: Commands,
    mut workspace: ResMut<Workspace>,
    chat_state: ResMut<ChatState>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    let pending_requests = workspace.worker.pending_requests.clone();
    let active_tasks = workspace.worker.active_tasks.clone();

    let can_process = {
        let active = *active_tasks.lock().unwrap();
        active < workspace.worker.max_concurrent
    };

    if can_process {
        let maybe_request = {
            let mut requests = pending_requests.lock().unwrap();
            if !requests.is_empty() {
                Some(requests.remove(0))
            } else {
                None
            }
        };

        if let Some(mut request) = maybe_request {
            {
                let mut active = active_tasks.lock().unwrap();
                *active += 1;
            }

            let active_tasks_clone = active_tasks.clone();
            if let Some(workspace) = workspace.workspace.as_mut() {
                info!("Request id: {}", request.id);
                workspace.add_request(request.id.clone());
            } else {
                info!("No workspace found");
                return;
            }
            let loaded_requests = workspace.loaded_requests.clone();
            let workspace_clone = workspace.clone();
            let cs = chat_state.clone();
            let task = task_pool.spawn(async move {
                let mut result = Vec::new();
                match request.get_request() {
                    RequestType::OverpassTurboRequest(ref query) => {
                        if let Ok(q) = workspace_clone
                            .overpass_agent
                            .send_overpass_query_string(query.clone())
                        {
                            if !q.is_empty() {
                                result = q.as_bytes().to_vec();
                            }
                        }
                    }
                    RequestType::OpenRouterRequest() => {
                        if let Some(workspace_data) = &workspace_clone.workspace {
                            // Process the LLM request with automatic follow-up capability
                            process_llm_request(&workspace_clone, workspace_data, &cs, 0);
                        }
                    }
                    RequestType::OpenMeteoRequest(_open_meteo_request) => {}
                }

                request.raw_data = result.clone();

                // Acquire the lock only when needed within the async block
                let mut loaded_requests_guard = loaded_requests.lock().unwrap();
                loaded_requests_guard.insert(request.get_id(), request.clone());
                drop(loaded_requests_guard); // Explicitly drop the guard

                let _serded = serde_json::to_string(&request).unwrap();
                // Save to folder.
                let mut active = active_tasks_clone.lock().unwrap();
                *active -= 1;
            });
            commands.spawn(TaskComponent(task));
        }
    }
}

#[derive(Component)]
pub struct TaskComponent(Task<()>);

pub fn cleanup_tasks(mut commands: Commands, mut tasks: Query<(Entity, &mut TaskComponent)>) {
    for (entity, mut task) in tasks.iter_mut() {
        if future::block_on(future::poll_once(&mut task.0)).is_some() {
            commands.entity(entity).despawn();
        }
    }
}

// Recursive function to handle LLM requests with automatic follow-up
fn process_llm_request(
    workspace_clone: &Workspace,
    workspace_data: &super::WorkspaceData,
    cs: &ChatState,
    recursion_depth: usize,
) {
    const MAX_RECURSION_DEPTH: usize = 5; // Prevent infinite loops

    if recursion_depth >= MAX_RECURSION_DEPTH {
        bevy::log::warn!("Maximum recursion depth reached for LLM requests");
        if let Ok(mut inner) = cs.inner.lock() {
            inner.is_processing = false;
        }
        return;
    }

    if let Ok(q) = workspace_clone
        .llm_agent
        .send_openrouter_chat(&workspace_data.messages)
    {
        if let Some(choice) = q.choices.first() {
            let ai_message = choice.message.content.clone();
            let mut workspace_data_mut = workspace_data.clone();
            workspace_data_mut.add_message("assistant", &ai_message);

            // Update chat state with thread-safe access
            if let Ok(mut inner) = cs.inner.lock() {
                inner.chat_history.push(ChatMessage {
                    content: ai_message.clone(),
                    is_user: false,
                });
            }

            if ai_message.len() > 2 && &ai_message[0..3] == "rq:" {
                let command_part = &ai_message[3..].trim();
                let mut parts = command_part.split_whitespace();

                if let Some(cmd) = parts.next() {
                    let response = match cmd {
                        "i" => {
                            // General info/stats
                            workspace_clone.get_info()
                        }
                        "cnt" => {
                            // Count features - can be for whole workspace or specific area
                            format!("Feature count: {}", workspace_clone.count_workspace())
                        }
                        "nb" => {
                            // Nearby features: rq: nb {51.5,-0.09} r500
                            if let (Some(coord_str), Some(radius_str)) =
                                (parts.next(), parts.next())
                            {
                                if let (Ok(coord), Ok(radius)) =
                                    (parse_coord(coord_str), parse_radius(radius_str))
                                {
                                    let features = workspace_clone.nearby_point(coord, radius);
                                    format!(
                                        "Found {} nearby features: {:?}",
                                        features.len(),
                                        features.iter().map(|f| &f.id).collect::<Vec<_>>()
                                    )
                                } else {
                                    "Invalid coordinate or radius format. Use: rq: nb {lat,lon} r<meters>".to_string()
                                }
                            } else {
                                "Missing parameters. Use: rq: nb {lat,lon} r<meters>".to_string()
                            }
                        }
                        "sm" => {
                            // Summarize features: rq: sm
                            workspace_clone.summarize_features()
                        }
                        "gt" => {
                            // Feature details by ID: rq: gt 123456
                            if let Some(id) = parts.next() {
                                if let Some(feature) = workspace_clone.get_feature_by_id(id) {
                                    format!("Feature {}: {:?}", id, feature)
                                } else {
                                    format!("Feature {} not found", id)
                                }
                            } else {
                                "Missing feature ID. Use: rq: gt <feature_id>".to_string()
                            }
                        }
                        "t" => {
                            // Feature tags: rq: t 123456
                            if let Some(id) = parts.next() {
                                if let Some(tags) = workspace_clone.get_feature_tags(id) {
                                    format!("Tags for feature {}: {}", id, tags)
                                } else {
                                    format!("Feature {} not found or has no tags", id)
                                }
                            } else {
                                "Missing feature ID. Use: rq: t <feature_id>".to_string()
                            }
                        }
                        "bb" => {
                            // Features in bbox: rq: bb {51.4,-0.1,51.6,-0.08}
                            if let Some(bbox_str) = parts.next() {
                                if let Ok((min_lat, min_lon, max_lat, max_lon)) =
                                    parse_bbox(bbox_str)
                                {
                                    let features = workspace_clone
                                        .features_in_bbox(min_lat, min_lon, max_lat, max_lon);
                                    format!(
                                        "Found {} features in bbox: {:?}",
                                        features.len(),
                                        features.iter().map(|f| &f.id).collect::<Vec<_>>()
                                    )
                                } else {
                                    "Invalid bbox format. Use: rq: bb {min_lat,min_lon,max_lat,max_lon}".to_string()
                                }
                            } else {
                                "Missing bbox. Use: rq: bb {min_lat,min_lon,max_lat,max_lon}"
                                    .to_string()
                            }
                        }
                        "d" => {
                            // Distance between points: rq: d {51.5,-0.09} {51.6,-0.10}
                            if let (Some(coord1_str), Some(coord2_str)) =
                                (parts.next(), parts.next())
                            {
                                if let (Ok(coord1), Ok(coord2)) =
                                    (parse_coord(coord1_str), parse_coord(coord2_str))
                                {
                                    let distance = workspace_clone.distance_between(coord1, coord2);
                                    format!("Distance: {:.2} meters", distance)
                                } else {
                                    "Invalid coordinate format. Use: rq: d {lat1,lon1} {lat2,lon2}"
                                        .to_string()
                                }
                            } else {
                                "Missing coordinates. Use: rq: d {lat1,lon1} {lat2,lon2}"
                                    .to_string()
                            }
                        }
                        "n" => {
                            // Nearest feature: rq: n {51.5,-0.09}
                            if let Some(coord_str) = parts.next() {
                                if let Ok(coord) = parse_coord(coord_str) {
                                    if let Some(feature) = workspace_clone.nearest_feature(coord) {
                                        format!("Nearest feature: {} at {:?}", feature.id, feature)
                                    } else {
                                        "No features found".to_string()
                                    }
                                } else {
                                    "Invalid coordinate format. Use: rq: n {lat,lon}".to_string()
                                }
                            } else {
                                "Missing coordinate. Use: rq: n {lat,lon}".to_string()
                            }
                        }
                        _ => {
                            format!(
                                "Unknown command: {}. Available commands: i, cnt, nb, sm, gt, t, bb, d, n",
                                cmd
                            )
                        }
                    };

                    bevy::log::info!("Command executed: {} -> {}", command_part, response);
                    workspace_data_mut.add_message("user", &response);
                    bevy::log::info!("Command executed: {} -> {}", command_part, response);

                    bevy::log::info!("Automatically following up with LLM after providing data...");
                    process_llm_request(
                        workspace_clone,
                        &workspace_data_mut,
                        cs,
                        recursion_depth + 1,
                    );
                }
            } else {
                // This is a final answer, not a data request - stop processing
                bevy::log::info!(
                    "LLM provided final answer (no command detected): '{}'",
                    ai_message
                );
                bevy::log::info!("Setting is_processing = false");
                if let Ok(mut inner) = cs.inner.lock() {
                    inner.is_processing = false;
                    bevy::log::info!("Successfully set is_processing = false");
                } else {
                    bevy::log::error!("Failed to lock chat state to set is_processing = false");
                }
            }
        } else {
            bevy::log::error!("No choices returned from LLM");
            let mut workspace_data_mut = workspace_data.clone();
            workspace_data_mut.add_message(
                "assistant",
                "Sorry, I didn't receive a proper response from the AI. Please try again.",
            );

            if let Ok(mut inner) = cs.inner.lock() {
                inner.chat_history.push(ChatMessage {
                    content:
                        "Sorry, I didn't receive a proper response from the AI. Please try again."
                            .to_string(),
                    is_user: false,
                });
                inner.is_processing = false;
            }
        }
    } else {
        bevy::log::error!("Failed to send chat request to LLM");
        if let Ok(mut inner) = cs.inner.lock() {
            inner.is_processing = false;
        }
    }
}

// Helper functions for parsing LLM command parameters
fn parse_coord(coord_str: &str) -> Result<Coord, String> {
    let coord_str = coord_str.trim_start_matches('{').trim_end_matches('}');
    let parts: Vec<&str> = coord_str.split(',').collect();

    if parts.len() != 2 {
        return Err("Coordinate must have exactly 2 parts".to_string());
    }

    let lat = parts[0]
        .trim()
        .parse::<f32>()
        .map_err(|_| "Invalid latitude")?;
    let long = parts[1]
        .trim()
        .parse::<f32>()
        .map_err(|_| "Invalid longitude")?;

    Ok(Coord { lat, long })
}

fn parse_radius(radius_str: &str) -> Result<f64, String> {
    if !radius_str.starts_with('r') {
        return Err("Radius must start with 'r'".to_string());
    }

    radius_str[1..]
        .parse::<f64>()
        .map_err(|_| "Invalid radius value".to_string())
}

fn parse_bbox(bbox_str: &str) -> Result<(f64, f64, f64, f64), String> {
    let bbox_str = bbox_str.trim_start_matches('{').trim_end_matches('}');
    let parts: Vec<&str> = bbox_str.split(',').collect();

    if parts.len() != 4 {
        return Err("Bbox must have exactly 4 parts".to_string());
    }

    let min_lat = parts[0]
        .trim()
        .parse::<f64>()
        .map_err(|_| "Invalid min_lat")?;
    let min_lon = parts[1]
        .trim()
        .parse::<f64>()
        .map_err(|_| "Invalid min_lon")?;
    let max_lat = parts[2]
        .trim()
        .parse::<f64>()
        .map_err(|_| "Invalid max_lat")?;
    let max_lon = parts[3]
        .trim()
        .parse::<f64>()
        .map_err(|_| "Invalid max_lon")?;

    Ok((min_lat, min_lon, max_lat, max_lon))
}
