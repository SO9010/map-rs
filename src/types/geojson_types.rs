use bevy::prelude::*;
use geo::BoundingRect;
use rstar::{RTree, RTreeObject, AABB};

#[derive(Component, Clone, Debug, PartialEq)]
pub struct MapFeature {
    pub id: String,
    pub properties: serde_json::Value,  // Use serde_json for flexible properties such as buidling type
    // Next make this a spacial hashmap, it becomes slower to check if a point is in a polygon the more there are
    pub geometry: geo::Polygon    // Next make this a spacial hashmap
}
impl MapFeature {
    pub fn get_in_world_space(&self) -> Vec<Vec2> {
        let new_geo = self.geometry.clone();
        let exterior = new_geo.exterior().clone();
        let mut new_points = Vec::new();
        for coord in exterior {
            let point = lat_lon_to_world_mercator(coord.x as f32, coord.y as f32, SCALE, STARTING_LONG_LAT.x, STARTING_LONG_LAT.y);
            new_points.push(point);
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

fn polygon_area(geometry: &Vec<Vec2>) -> f32 {
    let mut area: f32 = 0.0;
    let j = geometry.len() - 1;
    for i in 0..geometry.len() {
        area += (geometry[j].x + geometry[i].x) * (geometry[j].y - geometry[i].y);
    }

    area
}

impl WorldSpaceRect {
    // This will split the current rect into multiple rects, it really struggles with getting anything if it is overflowing to the left.
    pub fn split(&self, rects: Vec<WorldSpaceRect>) -> Option<Vec<WorldSpaceRect>> {
        let mut result = vec![self.clone()];

        for rect in rects {
            let mut new_result = Vec::new();
            for r in result {
                if let Some(mut split_rects) = r.split_single(&rect) {
                    new_result.append(&mut split_rects);
                } else {
                    new_result.push(r);
                }
            }
            result = new_result;
        }

        Some(result)
    }

    pub fn split_single(&self, rect: &WorldSpaceRect) -> Option<Vec<WorldSpaceRect>> {
        let mut result = Vec::new();

        // Add the left region
        if self.left < rect.left {
            result.push(WorldSpaceRect {
                left: self.left,
                right: rect.left,
                bottom: self.bottom,
                top: self.top,
            });
        }

        // Add the right region
        if self.right > rect.right {
            result.push(WorldSpaceRect {
                left: rect.right,
                right: self.right,
                bottom: self.bottom,
                top: self.top,
            });
        }

        // Add the bottom region
        if self.bottom < rect.bottom {
            result.push(WorldSpaceRect {
                left: rect.left,
                right: rect.right,
                bottom: self.bottom,
                top: rect.bottom,
            });
        }

        // Add the top region
        if self.top > rect.top {
            result.push(WorldSpaceRect {
                left: rect.left,
                right: rect.right,
                bottom: rect.top,
                top: self.top,
            });
        }

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}

impl RTreeObject for WorldSpaceRect {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners([self.left, self.bottom], [self.right, self.top])
    }
}

#[derive(Component, Clone, Debug)]
pub struct SpatialIndex {
    rtree: RTree<WorldSpaceRect>,
}

impl SpatialIndex {
    pub fn new() -> Self {
        SpatialIndex {
            rtree: RTree::new(),
        }
    }

    pub fn insert(&mut self, rect: WorldSpaceRect) {
        self.rtree.insert(rect);
    }

    pub fn insert_vec(&mut self, rect: Vec<WorldSpaceRect>) {
        for rect in rect {
            self.rtree.insert(rect);
        }
    }

    pub fn query(&self, rect: &WorldSpaceRect) -> Vec<&WorldSpaceRect> {
        self.rtree.locate_in_envelope_intersecting(&rect.envelope()).collect()
    }

    pub fn split(&self, rect: &WorldSpaceRect) -> Vec<WorldSpaceRect> {
        let r2: Vec<WorldSpaceRect> = self.rtree.locate_in_envelope_intersecting(&rect.envelope()).cloned().collect();
        if !r2.is_empty() {
            rect.split(r2).unwrap()
        } else {
            Vec::new()
        }
    }

    pub fn is_covered(&self, rect: &WorldSpaceRect) -> bool {
     //   let mut result = rect.clone();
     //   self.rtree.locate_in_envelope_intersecting(&result.envelope()).into_iter().all(|r| {
           //  result = result.split(r).unwrap();
    //        true
   //     });

        false   
    }
}

#[derive(Component, Clone, Debug)]
pub struct MapPoints {
    pub spatial_index: SpatialIndex,
    pub refrencee_point: RefrencePoint, // Refrence point of the map, this is used to calculate the scale and offset
    pub scale: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct MapBundle {
    /// A collection of map features, please put this in a spatial hashmap
    pub features: RTree<MapFeature>,
    pub selected_features: Vec<MapFeature>,

    /// Map points of the map, this is used to calculate the scale and offset
    pub map_points: MapPoints,

    pub features_to_respawn: Vec<MapFeature>,

    pub respawn_specific_features: bool,
    pub respawn: bool,
    pub get_more_data: bool,
}


impl MapBundle {
    pub fn new(long: f32, lat: f32, scale: f32) -> Self {
        Self {
            features: RTree::new(),
            selected_features: Vec::new(),
            features_to_respawn: Vec::new(),
            map_points: MapPoints {
                refrencee_point: RefrencePoint::new(long, lat),
                spatial_index: SpatialIndex::new(),
                scale,
            },
            respawn_specific_features: false,
            respawn: false,
            get_more_data: false,
        }
    }

    // Method to apply a Mercator projection to a coordinate, otherwise the coordinates will be too small to be rendered
    pub fn lat_lon_to_mercator(&self, lat: f32, lon: f32) -> Vec2 {
        lat_lon_to_world_mercator(lat, lon, self.map_points.scale, self.map_points.refrencee_point.long, self.map_points.refrencee_point.lat)
    }
}