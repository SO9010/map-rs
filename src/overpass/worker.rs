use crate::geojson::get_data_from_string_osm;
use crate::types::{MapFeature, Selection, SelectionType};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_map_viewer::DistanceType;
use bevy_tasks::futures_lite::future;
use crossbeam_channel::{bounded, Receiver};
use std::sync::{Arc, Mutex};

use super::OverpassClientResource;

#[derive(Resource)]
pub struct OverpassWorker {
    pending_requests: Arc<Mutex<Vec<OverpassRequest>>>,
    max_concurrent: usize,
    active_tasks: Arc<Mutex<usize>>,
}

pub struct OverpassRequest {
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

    pub fn queue_request(&self, selection: Selection) -> Receiver<Vec<MapFeature>> {
        let (tx, rx) = bounded(1);
        let request = OverpassRequest { selection, tx };
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.push(request);
        }

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

fn cleanup_tasks(mut commands: Commands, mut tasks: Query<(Entity, &mut TaskComponent)>) {
    for (entity, mut task) in tasks.iter_mut() {
        if future::block_on(future::poll_once(&mut task.0)).is_some() {
            commands.entity(entity).despawn();
        }
    }
}

pub struct OverpassWorkerPlugin;

impl Plugin for OverpassWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OverpassWorker::new(3))
            .add_systems(Update, process_requests)
            .add_systems(Update, cleanup_tasks);
    }
}
