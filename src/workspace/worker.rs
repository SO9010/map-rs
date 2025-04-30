use crate::workspace::{RequestType, WorkspaceRequest};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_tasks::futures_lite::future;
use std::sync::{Arc, Mutex};

use super::Workspace;

#[derive(Default)]
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

pub fn process_requests(mut commands: Commands, mut workspace: ResMut<Workspace>) {
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
            let client = workspace.overpass_agent.clone();
            // Clone the loaded_requests Arc<Mutex> instead of holding the MutexGuard
            let loaded_requests = workspace.loaded_requests.clone();
            let task = task_pool.spawn(async move {
                let mut result = Vec::new();
                match request.get_request() {
                    RequestType::OverpassTurboRequest(ref query) => {
                        if let Ok(q) = client.send_overpass_query_string(query.clone()) {
                            if !q.is_empty() {
                                result = q.as_bytes().to_vec();
                            }
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
