use bevy::app::*;
use super::{MeasurePlugin, SelectionPlugin};

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((SelectionPlugin, MeasurePlugin));
    }
}