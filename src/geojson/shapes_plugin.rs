use bevy::prelude::*;

use crate::types::MapBundle;

use super::{bbox_system, get_green_belt_data, read_overpass_receiver, respawn_shapes};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapBundle::new())
            .add_systems(Update, respawn_shapes)
            .add_systems(Update, (bbox_system, get_green_belt_data))
            .add_systems(FixedUpdate, read_overpass_receiver);
    }
}

