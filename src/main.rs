use bevy::{
    prelude::*,
    winit::{UpdateMode, WinitSettings},
};
use bevy_prototype_lyon::plugin::ShapePlugin;
use camera::CameraSystemPlugin;
use geojson::MapPlugin;
use interaction::InteractionSystemPlugin;
use settings::SettingsPlugin;
use types::Coord;

pub mod camera;
pub mod debug;
pub mod geojson;
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
        .insert_resource(ClearColor(Color::from(Srgba {
            red: 0.1,
            green: 0.1,
            blue: 0.1,
            alpha: 1.0,
        })))
        .add_plugins(MapPlugin)
        .add_plugins(SettingsPlugin)
        .run();
}
