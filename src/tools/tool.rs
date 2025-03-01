use bevy::app::*;
use super::SelectionPlugin;

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SelectionPlugin);
    }
}