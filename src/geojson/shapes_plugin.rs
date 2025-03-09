use bevy::prelude::*;

use crate::types::MapBundle;

use super::respawn_shapes;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapBundle::new())
            .add_systems(Update, respawn_shapes);
    }
}

