use crate::{EguiBlockInputState, workspace::SelectionAreas};
use bevy::{
    app::*,
    ecs::{
        resource::Resource,
        system::{Res, ResMut},
    },
    input::{ButtonInput, keyboard::KeyCode},
};

use super::{
    Measure, MeasurePlugin, PinPlugin, Pins, SelectionPlugin, SelectionSettings, ToolbarUiPlugin,
};

// Collected res for the tools. When you add a new tool, add it here.
#[derive(Resource)]
pub struct ToolResources {
    pub selection_areas: SelectionAreas,
    pub selection_settings: SelectionSettings,
    pub measure: Measure,
    pub pins: Pins,
    pub pointer: bool,
}

impl Default for ToolResources {
    fn default() -> Self {
        ToolResources {
            selection_areas: SelectionAreas::default(),
            selection_settings: SelectionSettings::default(),
            measure: Measure::default(),
            pins: Pins::default(),
            pointer: true,
        }
    }
}
impl ToolResources {
    pub fn respawn(&mut self) {
        self.selection_areas.respawn = true;
        self.measure.respawn = true;
        self.pins.respawn = true;
    }

    pub fn select_tool(&mut self, tool: &str) {
        match tool {
            "workspace" => {
                self.selection_settings.enabled = true;
                self.measure.disable();
                self.pins.enabled = false;
                self.pointer = false;
            }
            "measure" => {
                self.selection_settings.enabled = false;
                self.measure.enabled = true;
                self.pins.enabled = false;
                self.pointer = false;
            }
            "pins" => {
                self.selection_settings.enabled = false;
                self.measure.disable();
                self.pins.enabled = true;
                self.pointer = false;
            }
            "pointer" => {
                self.selection_settings.enabled = false;
                self.measure.disable();
                self.pins.enabled = false;
                self.pointer = true;
            }
            _ => {
                self.selection_settings.enabled = false;
                self.measure.disable();
                self.pins.enabled = false;
                self.pointer = false;
            }
        }
        self.respawn();
    }
}

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ToolResources::default())
            .add_plugins((SelectionPlugin, MeasurePlugin, PinPlugin, ToolbarUiPlugin))
            .add_systems(Update, handle_tool_keybinds);
    }
}

/// This is incontrol of key binds for the tools.
fn handle_tool_keybinds(
    mut tools: ResMut<ToolResources>,
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<EguiBlockInputState>,
) {
    if !state.block_input {
        if keys.just_pressed(KeyCode::KeyS) {
            if tools.selection_settings.enabled {
                tools.selection_settings.tool_type.iterate();
            }
            tools.select_tool("workspace");
        }
        if keys.just_pressed(KeyCode::KeyM) {
            tools.select_tool("measure");
        }
        if keys.just_pressed(KeyCode::KeyP) {
            tools.select_tool("pins");
        }
        if keys.just_pressed(KeyCode::KeyP) {
            tools.select_tool("pointer");
        }
    }
}
