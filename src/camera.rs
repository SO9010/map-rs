use bevy::{core_pipeline::bloom::Bloom, prelude::*};
use bevy_pancam::{DirectionKeys, PanCam, PanCamPlugin};
use rstar::RTree;

use crate::{tiles::{OfmTiles, TileMapResources}, tools::ToolResources, types::{world_mercator_to_lat_lon, Coord, MapBundle}, EguiBlockInputState, STARTING_DISPLACEMENT, STARTING_LONG_LAT, TILE_QUALITY};

pub struct CameraSystemPlugin;

impl Plugin for CameraSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanCamPlugin)
            .insert_resource(OfmTiles {
                tiles: RTree::new(),
                tiles_to_render: Vec::new(),
            })
            .insert_resource(CameraPosition::default())
            // This is being allowed, as it can't get the managers and location
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (handle_pancam, track_camera_position, camera_change));
    }
}

fn setup_camera(
    mut commands: Commands,
) {
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

fn handle_pancam(
    mut query: Query<&mut PanCam>, 
    state: Res<EguiBlockInputState>,
) {
    if state.is_changed() {
        for mut pancam in &mut query {
            pancam.enabled = !state.block_input;
        }
    }
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
    let left = camera_translation.x - ((window_width * projection.scale) / 2.0);
    let right = camera_translation.x  + ((window_width * projection.scale) / 2.0);
    let bottom = camera_translation.y + ((window_height * projection.scale) / 2.0);
    let top = camera_translation.y  - ((window_height * projection.scale) / 2.0);
    
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

#[derive(Resource, Default)]
pub struct CameraPosition {
    pub position: Vec3,
    pub changed: bool,
}

fn track_camera_position(
    camera_query: Query<&GlobalTransform, (With<Camera2d>, Changed<GlobalTransform>)>,
    mut camera_position: ResMut<CameraPosition>,
) {
    camera_position.changed = false;
    
    if let Ok(transform) = camera_query.get_single() {
        let new_position = transform.translation();
        
        // Check if position has changed
        if new_position != camera_position.position {
            camera_position.position = new_position;
            camera_position.changed = true;
        }
    }
}

fn camera_change(
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    camera_position: Res<CameraPosition>,
    mut tile_map_res: ResMut<TileMapResources>,
    mut map_bundle: ResMut<MapBundle>,
    mut tools: ResMut<ToolResources>,
) {
    if camera_position.is_changed() {
        let (_, camera_transform) = camera.single();
        let movement = camera_middle_to_lat_long(camera_transform, tile_map_res.zoom_manager.zoom_level, tile_map_res.zoom_manager.tile_size, tile_map_res.chunk_manager.refrence_long_lat);
        if movement != tile_map_res.location_manager.location {
            tile_map_res.location_manager.location = movement;

            if tile_map_res.zoom_manager.zoom_level > 16 {
                map_bundle.get_more_data = true;
            }
   
            tile_map_res.chunk_manager.update = true;
        }
        
        if tile_map_res.zoom_manager.zoom_level > 16 {
            map_bundle.get_more_data = true;
        }
        map_bundle.respawn = true;
        tools.respawn();
    }
}