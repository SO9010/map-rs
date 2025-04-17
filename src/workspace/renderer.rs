use bevy::ecs::system::{Res, ResMut};

use crate::geojson::{MapBundle, get_data_from_string_osm};

use super::Workspace;

pub fn render_workspace_requests(workspace: Res<Workspace>, mut map_bundle: ResMut<MapBundle>) {
    for request in workspace.get_unrendered_requests() {
        match request.get_request() {
            crate::workspace::RequestType::OverpassTurboRequest(_) => {
                if let Ok(data) =
                    get_data_from_string_osm(&String::from_utf8(request.raw_data.clone()).unwrap())
                {
                    for feature in data {
                        map_bundle.features.insert(feature.clone());
                    }
                }
            }
            crate::workspace::RequestType::OpenMeteoRequest(_) => {}
        }
        // Mark the request as rendered
        workspace.mark_as_rendered(&request.id);
    }
}
