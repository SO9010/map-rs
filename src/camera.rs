use bevy::{prelude::*, render::view::RenderLayers};
use bevy_map_viewer::{Coord, MapViewerPlugin, TileMapResources};
use bevy_pancam::{DirectionKeys, PanCam, PanCamPlugin};

use bevy_map_viewer::EguiBlockInputState;

pub struct CameraSystemPlugin;

impl Plugin for CameraSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanCamPlugin)
            .add_plugins(MapViewerPlugin {
                starting_location: Coord::new(52.1951, 0.1313),
                starting_zoom: 14,
                tile_quality: 256.0,
                cache_dir: "cache".to_string(),
            })
            .add_systems(Startup, setup_camera)
            .add_systems(Update, handle_pancam);
    }
}

fn setup_camera(mut commands: Commands, res_manager: Option<Res<TileMapResources>>) {
    if let Some(res_manager) = res_manager {
        let starting = res_manager
            .location_manager
            .location
            .to_game_coords(res_manager.clone());

        commands.spawn((
            Camera2d,
            RenderLayers::from_layers(&[0, 1]),
            Camera { ..default() },
            Transform {
                translation: Vec3::new(starting.x, starting.y, 1.0),
                ..Default::default()
            },
            PanCam {
                grab_buttons: vec![MouseButton::Middle],
                move_keys: DirectionKeys {
                    up: vec![KeyCode::ArrowUp],
                    down: vec![KeyCode::ArrowDown],
                    left: vec![KeyCode::ArrowLeft],
                    right: vec![KeyCode::ArrowRight],
                },
                speed: 400.,
                enabled: true,
                zoom_to_cursor: true,
                min_scale: 0.01,
                max_scale: f32::INFINITY,
                min_x: f32::NEG_INFINITY,
                max_x: f32::INFINITY,
                min_y: f32::NEG_INFINITY,
                max_y: f32::INFINITY,
            },
        ));
    } else {
        error!("TileMapResources not found. Please add the tilemap addon first.");
    }
}

fn handle_pancam(mut query: Query<&mut PanCam>, state: Res<EguiBlockInputState>) {
    if state.is_changed() {
        for mut pancam in &mut query {
            pancam.enabled = !state.block_input;
        }
    }
}
