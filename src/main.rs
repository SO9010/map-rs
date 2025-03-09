use bevy::{prelude::*, winit::{UpdateMode, WinitSettings}};

use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use camera::CameraSystemPlugin;
use debug::DebugPlugin;
use geojson::RenderPlugin;
use interaction::InteractionSystemPlugin;
use overpass::OverpassPlugin;
use settings::SettingsPlugin;
use tiles::TileMapPlugin;
use tools::ToolsPlugin;
use types::Coord;

pub mod camera;
pub mod debug;
pub mod geojson;
pub mod overpass;
pub mod tools;
pub mod tiles;
pub mod types;
pub mod settings;
pub mod interaction;

pub const STARTING_LONG_LAT: Coord = Coord::new(0.011, 0.011);
pub const STARTING_DISPLACEMENT: Coord = Coord::new(52.1951, 0.1313);
// This can be changed, it changes the size of each tile too.
pub const TILE_QUALITY: i32 = 256;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Map Viewer".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(DebugPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(TileMapPlugin)
        .insert_resource(EguiBlockInputState::default())
        .add_plugins((CameraSystemPlugin, InteractionSystemPlugin, ShapePlugin))
        .insert_resource(WinitSettings {
            unfocused_mode: UpdateMode::Reactive {
                wait: std::time::Duration::from_secs(1),
                react_to_device_events: true,
                react_to_user_events: true,
                react_to_window_events: true,
            },
            ..Default::default()
        })
        // This should be able to be an option.
        .insert_resource(ClearColor(Color::from(Srgba {
            red: 0.9,
            green: 0.9,
            blue: 0.8,
            alpha: 1.0,
        })))
        .add_plugins(RenderPlugin)
        .add_plugins(OverpassPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(ToolsPlugin)
        .add_systems(Update, absorb_egui_inputs)
        .run();
}

#[derive(Resource, Default)]
pub struct EguiBlockInputState {
    pub block_input: bool,
}
fn absorb_egui_inputs(
    mut contexts: bevy_egui::EguiContexts,
    mut state: ResMut<EguiBlockInputState>,
) {
    let ctx = contexts.ctx_mut();
    state.block_input = ctx.wants_pointer_input() || ctx.is_pointer_over_area();
}