//! # Map-RS Main Application
//! 
//! This is the entry point for the Map-RS interactive map viewer application.
//! 
//! ## Purpose
//! - Initializes the Bevy application with all necessary plugins
//! - Configures window settings and renderer options  
//! - Sets up the main game loop and system scheduling
//! - Manages input absorption for UI interactions
//! 
//! ## Architecture
//! The application uses a modular plugin system where each major feature
//! is implemented as a separate Bevy plugin:
//! - WorkspacePlugin: Core workspace and data management
//! - CameraSystemPlugin: Map navigation and camera controls
//! - InteractionSystemPlugin: User input handling
//! - RenderPlugin: GeoJSON and geographic data rendering
//! - SettingsPlugin: Application configuration
//! - ToolsPlugin: Interactive map tools
//! - DebugPlugin: Development and debugging utilities
//! 
//! ## Key Systems
//! - `absorb_egui_inputs`: Prevents game input when UI is active
//! - Plugin initialization and dependency management
//! - Window and renderer configuration

use bevy::{
    prelude::*,
    winit::{UpdateMode, WinitSettings},
};

use bevy_egui::EguiPlugin;
use bevy_map_viewer::EguiBlockInputState;
use camera::CameraSystemPlugin;
use debug::DebugPlugin;
use egui_extras::install_image_loaders;
use geojson::RenderPlugin;
use interaction::InteractionSystemPlugin;
use settings::SettingsPlugin;
use tools::ToolsPlugin;
use workspace::WorkspacePlugin;

pub mod camera;
pub mod debug;
pub mod geojson;
pub mod interaction;
pub mod llm;
pub mod overpass;
pub mod settings;
pub mod tools;
pub mod workspace;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Map Viewer".to_string(),
                        name: Some("Map Viewer".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .build()
                .disable::<bevy::audio::AudioPlugin>(),
        )
        .add_plugins((WorkspacePlugin, CameraSystemPlugin, InteractionSystemPlugin))
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
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
    install_image_loaders(ctx);
    state.block_input = ctx.wants_pointer_input() || ctx.is_pointer_over_area();
}
