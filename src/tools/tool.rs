use bevy::{app::*, ecs::system::{Res, ResMut}, input::{keyboard::KeyCode, ButtonInput}};
use super::{Measure, MeasurePlugin, PinPlugin, Pins, SelectionPlugin, SelectionSettings, ToolbarUiPlugin};

// TODO: !IMPORTANT! make it so that the clicks dont go through the ui.
pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SelectionPlugin, MeasurePlugin, PinPlugin, ToolbarUiPlugin))
            .add_systems(Update, handle_tool_keybinds);
    }
}

/// This is incontrol of key binds for the tools.
fn handle_tool_keybinds(
    mut selection_settings: ResMut<SelectionSettings>,
    mut measure: ResMut<Measure>,
    mut pins: ResMut<Pins>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyS) {
        if selection_settings.selection_enabled {
            selection_settings.selection_tool_type.iterate();
        }
        selection_settings.selection_enabled = true;
        measure.disable();
        pins.enabled = false;
    }
    if keys.just_pressed(KeyCode::KeyM) {
        measure.enabled = true;
        selection_settings.selection_enabled = false;
        pins.enabled = false;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        pins.enabled = true;
        selection_settings.selection_enabled = false;
        measure.disable();
    }
}
