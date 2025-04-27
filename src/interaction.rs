use bevy::prelude::*;
use bevy_map_viewer::ZoomChangedEvent;

use crate::geojson::{MapBundle, get_file_data};

pub struct InteractionSystemPlugin;

impl Plugin for InteractionSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, file_drop);
    }
}

fn file_drop(
    mut evr_dnd: EventReader<FileDragAndDrop>,
    mut map_bundle: ResMut<MapBundle>,
    mut zoom_event: EventWriter<ZoomChangedEvent>,
) {
    for ev in evr_dnd.read() {
        if let FileDragAndDrop::HoveredFile { window, path_buf } = ev {
            if path_buf.extension().unwrap() == "geojson" {
                // Make this so the UI respods to a hover for example we can chanage the ui to have a gray overlay and say "Drop file here" and if it will be accepted
                println!(
                    "Hovered file with path: {:?}, in window id: {:?}",
                    path_buf, window
                );
            }
        }
        if let FileDragAndDrop::DroppedFile { window, path_buf } = ev {
            if path_buf.extension().unwrap() == "geojson" {
                println!(
                    "Dropped file with path: {:?}, in window id: {:?}",
                    path_buf, window
                );
                get_file_data(&mut map_bundle.features, path_buf.to_str().unwrap());
                zoom_event.write(ZoomChangedEvent);
            }
        }
    }
}
