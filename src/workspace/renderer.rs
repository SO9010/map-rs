use bevy::ecs::{event::EventWriter, system::Res};
use bevy_map_viewer::ZoomChangedEvent;

use super::Workspace;

pub fn render_workspace_requests(
    workspace: Res<Workspace>,
    mut zoom_event: EventWriter<ZoomChangedEvent>,
) {
    let mut loaded_requests = workspace.loaded_requests.lock().unwrap();

    for (_, request) in loaded_requests.iter_mut() {
        if request.get_processed_data().size() != 0 {
            continue;
        }
        request.process_request();
        zoom_event.write(ZoomChangedEvent);
    }
}
