use bevy::prelude::*;

use super::respawn_shapes;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, respawn_shapes);
    }
}
