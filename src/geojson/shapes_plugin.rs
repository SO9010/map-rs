use bevy::prelude::*;

use crate::types::MapBundle;

use super::{bbox_system, read_overpass_receiver, respawn_overpass_map};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapBundle::new())
            .add_systems(Update, respawn_overpass_map)
            .add_systems(Update, bbox_system)
            .add_systems(FixedUpdate, read_overpass_receiver);
    }
}

