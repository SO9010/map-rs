use bevy::prelude::*;
use bevy_map_viewer::{Coord, TileMapResources};
use geo::BoundingRect;
use rstar::{RTree, RTreeObject, AABB};
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapFeature {
    pub id: String,
    pub properties: serde_json::Value,
    pub closed: bool,
    pub geometry: geo::Polygon,
}
impl MapFeature {
    pub fn get_in_world_space(&self, tile_map_resources: TileMapResources) -> Vec<Vec2> {
        let new_geo = self.geometry.clone();
        let exterior = new_geo.exterior().clone();
        let mut new_points = Vec::new();
        for coord in exterior {
            let point = Coord::new(coord.x as f32, coord.y as f32)
                .to_game_coords(tile_map_resources.clone());
            new_points.push(Vec2::new(point.x, point.y));
        }
        new_points
    }
}
impl RTreeObject for MapFeature {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let bbox = self.geometry.bounding_rect().unwrap();
        AABB::from_corners([bbox.min().x, bbox.min().y], [bbox.max().x, bbox.max().y])
    }
}

#[allow(dead_code)]
fn polygon_area(geometry: &[Vec2]) -> f32 {
    let mut area: f32 = 0.0;
    let j = geometry.len() - 1;
    for i in 0..geometry.len() {
        area += (geometry[j].x + geometry[i].x) * (geometry[j].y - geometry[i].y);
    }

    area
}

#[derive(Resource, Clone, Debug)]
pub struct MapBundle {
    /// A collection of map features, please put this in a spatial hashmap
    pub features: RTree<MapFeature>,
}

impl Default for MapBundle {
    fn default() -> Self {
        Self::new()
    }
}

impl MapBundle {
    pub fn new() -> Self {
        Self {
            features: RTree::new(),
        }
    }
}
