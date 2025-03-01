use bevy::{prelude::*, window::PrimaryWindow};

use crate::{camera::camera_middle_to_lat_long, geojson::get_file_data, tiles::{ChunkManager, Location, ZoomManager}, types::{world_mercator_to_lat_lon, MapBundle}};

pub struct InteractionSystemPlugin;

impl Plugin for InteractionSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_mouse)
            .add_systems(Update, camera_change)
            .add_systems(Update, file_drop);
    }
}

fn handle_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    zoom_manager: Res<ZoomManager>,
    mut location_manager: ResMut<Location>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut map_bundle: ResMut<MapBundle>,
) {
    let (camera, camera_transform) = camera.single();
    if buttons.pressed(MouseButton::Left) {
        if let Some(position) = q_windows.single().cursor_position() {
            /*
            let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
            let long_lat = world_mercator_to_lat_lon(world_pos.x as f64, world_pos.y as f64, chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size);
            let closest_tile = long_lat.to_tile_coords(zoom_manager.zoom_level).to_lat_long();
            info!("{:?}", closest_tile);
            */

            let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
            info!("{:?}", (world_pos, world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size)));
        }
    }   
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
   
            map_bundle.respawn = true;
            chunk_manager.update = true;
        }
    }

    if buttons.just_released(MouseButton::Right) {
        map_bundle.respawn = true;
        map_bundle.get_green_data = true;
    }
}

fn camera_change(
    zoom_manager: Res<ZoomManager>,
    mut map_bundle: ResMut<MapBundle>,
) {
    if zoom_manager.is_changed() {
        if zoom_manager.zoom_level > 16 {
            map_bundle.get_more_data = true;
        }
        map_bundle.respawn = true;
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