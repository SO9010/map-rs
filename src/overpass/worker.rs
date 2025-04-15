use crate::geojson::{MapFeature, get_data_from_string_osm};
use crate::workspace::{RequestType, Selection, SelectionType, WorkspaceData, WorkspaceRequest};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_map_viewer::DistanceType;
use bevy_tasks::futures_lite::future;
use crossbeam_channel::{Receiver, bounded};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::OverpassClientResource;

/// A resource that manages Overpass API requests and tasks.
/// It maintains a queue of pending requests and ensures that only a limited
/// number of tasks are processed concurrently.
#[derive(Resource)]
pub struct OverpassWorker {
    /// A thread-safe queue of pending Overpass requests.
    pending_requests: Arc<Mutex<Vec<OverpassRequest>>>,
    /// The maximum number of concurrent tasks allowed.
    max_concurrent: usize,
    /// A counter to track the number of active tasks.
    active_tasks: Arc<Mutex<usize>>,
}

pub struct OverpassRequest {
    id: String,
    selection: Selection,
    tx: crossbeam_channel::Sender<Vec<MapFeature>>,
}

#[derive(Resource, Deref)]
pub struct OverpassReceiver(pub Receiver<Vec<MapFeature>>);

impl OverpassWorker {
    pub fn new(max_workers: usize) -> Self {
        OverpassWorker {
            pending_requests: Arc::new(Mutex::new(Vec::new())),
            max_concurrent: max_workers,
            active_tasks: Arc::new(Mutex::new(0)),
        }
    }

    /// Change this to take the workspace so then we can handle everthing in the workspace too.
    pub fn queue_request(&self, mut workspace: WorkspaceData) -> Receiver<Vec<MapFeature>> {
        let (tx, rx) = bounded(1);
        let id = Uuid::new_v4().to_string();
        let request = OverpassRequest {
            id: id.clone(),
            selection: workspace.get_selection(),
            tx,
        };
        // Update the workspace with the new request
        workspace.add_request(id);
        let serded = serde_json::to_string(&workspace).unwrap();
        info!("Updated workspace: {}", serded);
        // TODO: Save the workspace to the database

        let mut pending = self.pending_requests.lock().unwrap();
        pending.push(request);

        rx
    }
}

pub fn get_bounds(selection: Selection) -> String {
    if selection.points.is_some() {
        if selection.selection_type == SelectionType::POLYGON {
            let points_string = selection
                .points
                .unwrap()
                .iter()
                .map(|point| format!("{} {}", point.lat, point.long))
                .collect::<Vec<String>>()
                .join(" ");
            format!("poly:\"{}\"", points_string)
        } else {
            String::default()
        }
    } else if selection.start.is_some() && selection.end.is_some() {
        match selection.selection_type {
            SelectionType::RECTANGLE => {
                let start = selection.start.unwrap();
                let end = selection.end.unwrap();
                format!(
                    "poly:\"{} {} {} {} {} {} {} {}\"",
                    start.lat,
                    start.long,
                    start.lat,
                    end.long,
                    end.lat,
                    end.long,
                    end.lat,
                    start.long
                )
            }
            SelectionType::CIRCLE => {
                let start = selection.start.unwrap();
                let end = selection.end.unwrap();
                let (mut dist, dist_type) = start.distance(&end);
                match dist_type {
                    DistanceType::Km => dist *= 1000.0,
                    DistanceType::M => {}
                    DistanceType::CM => dist /= 100.0,
                }
                format!("around:{}, {}, {}", dist, start.lat, start.long)
            }
            _ => {
                return String::new();
            }
        }
    } else {
        return String::new();
    }
}

/// Processes pending Overpass requests by spawning tasks for them.
/// Ensures that the number of active tasks does not exceed the maximum limit.
pub fn process_requests(
    mut commands: Commands,
    worker: Res<OverpassWorker>,
    overpass_requester: Res<OverpassClientResource>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    let pending_requests = worker.pending_requests.clone();
    let active_tasks = worker.active_tasks.clone();

    let can_process = {
        let active = *active_tasks.lock().unwrap();
        active < worker.max_concurrent
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

        if let Some(request) = maybe_request {
            {
                let mut active = active_tasks.lock().unwrap();
                *active += 1;
            }

            let active_tasks_clone = active_tasks.clone();
            let mut client = overpass_requester.client.clone();
            client.bounds = get_bounds(request.selection);

            let task = task_pool.spawn(async move {
                let mut result = Vec::new();
                if let Ok(reslt) = client.send_overpass_query() {
                    if let Ok(data) = get_data_from_string_osm(&reslt) {
                        // TODO: Save the data with the request id so
                        WorkspaceRequest::new(
                            request.id.clone(),
                            2,
                            RequestType::OverpassTurboRequest(
                                client
                                    .build_overpass_query_string()
                                    .unwrap_or_else(|_| String::new()),
                            ),
                            reslt.as_bytes().to_vec(),
                        );
                        // info!("Overpass response: {:?}", data);
                        result.extend(data);
                    } else {
                        info!("Error parsing Overpass response");
                    }
                } else {
                    info!("Error sending Overpass query");
                }

                let _ = request.tx.send(result);

                let mut active = active_tasks_clone.lock().unwrap();
                *active -= 1;
            });

            commands.spawn(TaskComponent(task));
        }
    }
}

#[derive(Component)]
struct TaskComponent(Task<()>);

/// Cleans up completed tasks by despawning their associated entities.
fn cleanup_tasks(mut commands: Commands, mut tasks: Query<(Entity, &mut TaskComponent)>) {
    for (entity, mut task) in tasks.iter_mut() {
        if future::block_on(future::poll_once(&mut task.0)).is_some() {
            commands.entity(entity).despawn();
        }
    }
}

/// A plugin that sets up the Overpass worker system.
/// It initializes the worker resource and adds systems for processing and cleaning up tasks.
pub struct OverpassWorkerPlugin;

impl Plugin for OverpassWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OverpassWorker::new(3))
            .add_systems(Update, process_requests)
            .add_systems(Update, cleanup_tasks);
    }
}
