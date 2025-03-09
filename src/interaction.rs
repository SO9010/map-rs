use bevy::prelude::*;

use crate::{camera::camera_middle_to_lat_long, geojson::get_file_data, tiles::{ChunkManager, Location, ZoomManager}, tools::{Measure, Pins, SelectionAreas}, types::MapBundle};

pub struct InteractionSystemPlugin;

impl Plugin for InteractionSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_mouse)
            .add_systems(Update, camera_change)
            .add_systems(Update, file_drop);
    }
}

// TODO: Change this all to a on detect camera change. 
fn handle_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    zoom_manager: Res<ZoomManager>,
    mut location_manager: ResMut<Location>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut map_bundle: ResMut<MapBundle>,
) {
    let (_, camera_transform) = camera.single();
   
    if buttons.pressed(MouseButton::Middle){
        chunk_manager.update = true;
    }
    if buttons.just_released(MouseButton::Middle) {
        let movement = camera_middle_to_lat_long(camera_transform, zoom_manager.zoom_level, zoom_manager.tile_size, chunk_manager.refrence_long_lat);
        if movement != location_manager.location {
            location_manager.location = movement;

            if zoom_manager.zoom_level > 16 {
                map_bundle.get_more_data = true;
            }
   
            chunk_manager.update = true;
        }
    }
}

fn camera_change(
    zoom_manager: Res<ZoomManager>,
    mut map_bundle: ResMut<MapBundle>,
    mut selections: ResMut<SelectionAreas>,
    mut measure: ResMut<Measure>,
    mut pins: ResMut<Pins>,
) {
    if zoom_manager.is_changed() {
        if zoom_manager.zoom_level > 16 {
            map_bundle.get_more_data = true;
        }
        map_bundle.respawn = true;
        selections.respawn = true;
        measure.respawn = true;        
        pins.respawn = true;
    }
}

fn file_drop(
    mut evr_dnd: EventReader<FileDragAndDrop>,
    mut map_bundle: ResMut<MapBundle>,
) {
    for ev in evr_dnd.read() {
        if let FileDragAndDrop::HoveredFile { window, path_buf } = ev {
            if path_buf.extension().unwrap() == "geojson" {
                // Make this so the UI respods to a hover for example we can chanage the ui to have a gray overlay and say "Drop file here" and if it will be accepted
                println!("Hovered file with path: {:?}, in window id: {:?}", path_buf, window);
            }
        }
        if let FileDragAndDrop::DroppedFile { window, path_buf } = ev {
            if path_buf.extension().unwrap() == "geojson" {
                println!("Dropped file with path: {:?}, in window id: {:?}", path_buf, window);
                get_file_data(&mut map_bundle.features, path_buf.to_str().unwrap());
                map_bundle.respawn = true;
            }
        }
    }
}