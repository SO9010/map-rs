use bevy::{core_pipeline::bloom::Bloom, prelude::*, window::PrimaryWindow};
use bevy_pancam::{DirectionKeys, PanCam, PanCamPlugin};
use rstar::RTree;

use crate::{debug::DebugPlugin, tiles::{ChunkManager, Location, OfmTiles, TileMapPlugin, ZoomManager}, types::{world_mercator_to_lat_lon, Coord}, STARTING_DISPLACEMENT, STARTING_LONG_LAT, TILE_QUALITY};

pub struct CameraSystemPlugin;

impl Plugin for CameraSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((PanCamPlugin, TileMapPlugin))
            .insert_resource(Location::default())
            .add_plugins(DebugPlugin)
            .insert_resource(OfmTiles {
                tiles: RTree::new(),
                tiles_to_render: Vec::new(),
            })
            // This is being allowed, as it can't get the managers and location
            .add_systems(Startup, setup_camera)
            .add_systems(Update, handle_mouse);
    }
}

pub fn setup_camera(mut commands: Commands) {
    let starting = STARTING_DISPLACEMENT.to_game_coords(STARTING_LONG_LAT, 14, TILE_QUALITY.into());
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true, // HDR is required for the bloom effect
            ..default()
        },
        Transform {
            translation: Vec3::new(starting.x, starting.y, 1.0),
            ..Default::default()
        },
        PanCam {
            grab_buttons: vec![MouseButton::Middle], // which buttons should drag the camera
            move_keys: DirectionKeys {      // the keyboard buttons used to move the camera
                up:    vec![KeyCode::ArrowUp], // initalize the struct like this or use the provided methods for
                down:  vec![KeyCode::ArrowDown], // common key combinations
                left:  vec![KeyCode::ArrowLeft],
                right: vec![KeyCode::ArrowRight],
            },
            speed: 400., // the speed for the keyboard movement
            enabled: true, // when false, controls are disabled. See toggle example.
            zoom_to_cursor: false, // whether to zoom towards the mouse or the center of the screen
            min_scale: 0.25, // prevent the camera from zooming too far in
            max_scale: f32::INFINITY, // prevent the camera from zooming too far out
            min_x: f32::NEG_INFINITY, // minimum x position of the camera window
            max_x: f32::INFINITY, // maximum x position of the camera window
            min_y: f32::NEG_INFINITY, // minimum y position of the camera window
            max_y: f32::INFINITY, // maximum y position of the camera window
        },
        Bloom::NATURAL,
    ));
}

pub fn camera_space_to_lat_long_rect(
    transform: &GlobalTransform,
    window: &Window,
    projection: OrthographicProjection,
    zoom: u32,
    quality: f32,
    reference: Coord,
) -> Option<geo::Rect<f32>> {
    // Get the window size
    let window_width = window.width(); 
    let window_height = window.height();

    // Get the camera's position
    let camera_translation = transform.translation();

    // Compute the world-space rectangle
    // The reason for not dividing by 2 is to make the rectangle larger, as then it will mean that we can load more data
    let left = camera_translation.x ;
    let right = camera_translation.x  + ((window_width * projection.scale) / 2.0);
    let bottom = camera_translation.y + ((window_height * projection.scale) / 2.0);
    let top = camera_translation.y;
    
    Some(geo::Rect::<f32>::new(
        world_mercator_to_lat_lon(left.into(), bottom.into(), reference, zoom, quality).to_tuple(),
        world_mercator_to_lat_lon(right.into(), top.into(), reference, zoom, quality).to_tuple(),
    ))
}

pub fn camera_middle_to_lat_long(
    transform: &GlobalTransform,
    zoom: u32,
    quality: f32,
    reference: Coord,
) -> Coord {
    let camera_translation = transform.translation();
    world_mercator_to_lat_lon(camera_translation.x.into(), camera_translation.y.into(), reference, zoom, quality)
}

pub fn handle_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    zoom_manager: Res<ZoomManager>,
    mut location_manager: ResMut<Location>,
    mut chunk_manager: ResMut<ChunkManager>,
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
            info!("{:?}", world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size));
        }
    }   
    if buttons.pressed(MouseButton::Middle){
        chunk_manager.update = true;
    }
    if buttons.just_released(MouseButton::Middle) {
        let movement = camera_middle_to_lat_long(camera_transform, zoom_manager.zoom_level, zoom_manager.tile_size, chunk_manager.refrence_long_lat);
        if movement != location_manager.location {
            location_manager.location = movement;
            chunk_manager.update = true;
        }
    }
}
