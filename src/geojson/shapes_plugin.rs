use bevy::prelude::*;

use super::{MapBundle, respawn_shapes};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapBundle::new())
            .add_systems(Update, respawn_shapes);
    }
}
