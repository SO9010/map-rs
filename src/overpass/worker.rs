use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_tasks::futures_lite::future;
use crossbeam_channel::{bounded, Receiver};
use std::sync::{Arc, Mutex};

use crate::tools::Selection;
use crate::types::{MapFeature, SettingsOverlay};

use super::{get_overpass_query, send_overpass_query};

#[derive(Resource)]
pub struct OverpassWorker {
    pending_requests: Arc<Mutex<Vec<OverpassRequest>>>,
    max_concurrent: usize,
    active_tasks: Arc<Mutex<usize>>,
}

pub struct OverpassRequest {
    selection: Selection,
    settings_snapshot: SettingsOverlay,
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
    
    pub fn queue_request(&self, selection: Selection, 
                        settings: SettingsOverlay) -> Receiver<Vec<MapFeature>> {
        let (tx, rx) = bounded(1);
        let request = OverpassRequest {
            selection,
            settings_snapshot: settings,
            tx,
        };
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.push(request);
        }
        
        rx
    }
}

pub fn process_requests(
    mut commands: Commands,
    worker: Res<OverpassWorker>,
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
        
        if let Some(mut request) = maybe_request {
            {
                let mut active = active_tasks.lock().unwrap();
                *active += 1;
            }
            
            let active_tasks_clone = active_tasks.clone();
            let task = task_pool.spawn(async move {

                let mut result = Vec::new();
                
                let query = get_overpass_query(request.selection, &mut request.settings_snapshot);
                info!("Query: {}", query);
                if query != "ERR" {
                    result.extend(send_overpass_query(query));
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

fn cleanup_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut TaskComponent)>,
) {
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