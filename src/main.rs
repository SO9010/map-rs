use bevy::{
    prelude::*,
    winit::{UpdateMode, WinitSettings},
};

use bevy_egui::EguiPlugin;
use bevy_map_viewer::EguiBlockInputState;
use bevy_prototype_lyon::plugin::ShapePlugin;
use camera::CameraSystemPlugin;
use debug::DebugPlugin;
use geojson::RenderPlugin;
use interaction::InteractionSystemPlugin;
use overpass::OverpassPlugin;
use settings::SettingsPlugin;
use tiles::TilesUiPlugin;
use tools::ToolsPlugin;

pub mod camera;
pub mod debug;
pub mod geojson;
pub mod interaction;
pub mod overpass;
pub mod settings;
pub mod tiles;
pub mod tools;
pub mod types;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Map Viewer".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins((
            CameraSystemPlugin,
            InteractionSystemPlugin,
            ShapePlugin,
            TilesUiPlugin,
        ))
        .add_plugins(EguiPlugin)
        .add_plugins(DebugPlugin)
        .insert_resource(WinitSettings {
            unfocused_mode: UpdateMode::Reactive {
                wait: std::time::Duration::from_secs(1),
                react_to_device_events: true,
                react_to_user_events: true,
                react_to_window_events: true,
            },
            ..Default::default()
        })
        .add_plugins(RenderPlugin)
        .add_plugins(OverpassPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(ToolsPlugin)
        .add_systems(Update, absorb_egui_inputs)
        .run();
}

fn absorb_egui_inputs(
    mut contexts: bevy_egui::EguiContexts,
    mut state: ResMut<EguiBlockInputState>,
) {
    let ctx = contexts.ctx_mut();
    state.block_input = ctx.wants_pointer_input() || ctx.is_pointer_over_area();
}
