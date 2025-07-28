use crate::{
    geojson::MapFeature,
    workspace::Workspace,
};
use bevy_map_viewer::Coord;
use geo::Centroid;
use rstar::AABB;
use std::fmt::Display;

impl Workspace {
    // i    : General info/stats.
    pub fn get_info(&self) -> String {
        let count = self.count_workspace();
        if let Some(workspace) = &self.workspace {
            let area = workspace.get_area();
            let selection = &workspace.selection;
            return format!(
                "Features: {count}, Area: {area:?}, Selection: {selection:?}"
            );
        }
        format!("Features: {count}")
    }

    // t    : Tags for feature. ex: rq: t 123456
    pub fn get_feature_tags(&self, id: impl Display) -> Option<serde_json::Value> {
        for request in self.get_requests() {
            for feature in request.get_processed_data() {
                if feature.id == id.to_string() {
                    return Some(feature.properties.clone());
                }
            }
        }
        None
    }

    // gt   : Feature details by ID. ex: rq: gt 123456
    pub fn get_feature_by_id(&self, id: impl Display) -> Option<MapFeature> {
        for request in self.get_requests() {
            for feature in request.get_processed_data() {
                if feature.id == id.to_string() {
                    return Some(feature.clone());
                }
            }
        }
        None
    }

    // cnt  : Count features. ex: rq: cnt {51.5,-0.09} r500 or blank for all workspace
    pub fn count_workspace(&self) -> i32 {
        self.get_requests()
            .iter()
            .map(|r| r.processed_data.size() as i32)
            .sum()
    }

    // nb   : Nearby features. ex: rq: nb {51.5,-0.09} r500
    pub fn nearby_point(&self, point: Coord, radius: f64) -> Vec<MapFeature> {
        let mut matches = Vec::new();
        for request in self.get_requests() {
            let mut m: Vec<MapFeature> = request
                .processed_data
                .iter()
                .filter(|feature| {
                    let c = feature.geometry.centroid().unwrap();
                    let coord_c = Coord {
                        lat: c.y() as f32,
                        long: c.x() as f32,
                    };
                    point.distance_haversine(&coord_c) <= radius
                })
                .cloned()
                .collect();
            matches.append(&mut m);
        }
        matches
    }

    // sm   : Summarize features. ex: rq: sm {51.5,-0.09} r500
    pub fn summarize_features(&self, point: Coord, radius: f64) -> String {
        let features = self.nearby_point(point, radius);
        let count = features.len();
        let mut tags_count = std::collections::HashMap::new();

        for f in &features {
            if let Some(props) = f.properties.as_object() {
                for (k, _v) in props {
                    *tags_count.entry(k.clone()).or_insert(0) += 1;
                }
            }
        }

        format!("Count: {count}, Tags Summary: {tags_count:?}")
    }

    // bb   : Features in bbox. ex: rq: bb {51.4,-0.1,51.6,-0.08}
    pub fn features_in_bbox(
        &self,
        min_lat: f64,
        min_lon: f64,
        max_lat: f64,
        max_lon: f64,
    ) -> Vec<MapFeature> {
        let envelope = AABB::from_corners([min_lon, min_lat], [max_lon, max_lat]);
        let mut matches = Vec::new();

        for request in self.get_requests() {
            let mut m: Vec<MapFeature> = request
                .processed_data
                .locate_in_envelope(&envelope)
                .cloned()
                .collect();
            matches.append(&mut m);
        }

        matches
    }

    // d    : Distance between two. ex: rq: d {51.5,-0.09} {51.6,-0.10}
    pub fn distance_between(&self, a: Coord, b: Coord) -> f64 {
        a.distance_haversine(&b)
    }

    // n    : Nearest feature. ex: rq: n {51.5,-0.09}
    pub fn nearest_feature(&self, point: Coord) -> Option<MapFeature> {
        let mut nearest: Option<MapFeature> = None;
        let mut min_dist = f64::MAX;

        for request in self.get_requests() {
            for feature in request.get_processed_data() {
                let c = feature.geometry.centroid().unwrap();
                let coord_c = Coord {
                    lat: c.y() as f32,
                    long: c.x() as f32,
                };
                let dist: f64 = point.distance_haversine(&coord_c);
                if dist < min_dist {
                    min_dist = dist;
                    nearest = Some(feature.clone());
                }
            }
        }

        nearest
    }

    // ply  : Features in polygon. ex: rq: ply {[51.5,-0.1],[51.6,-0.1],[51.6,-0.09]}
    pub fn features_in_polygon(&self, polygon: &[Coord]) -> Vec<MapFeature> {
        fn point_in_polygon(point: &Coord, polygon: &[Coord]) -> bool {
            // Ray casting algorithm
            let mut inside = false;
            let mut j = polygon.len() - 1;
            for i in 0..polygon.len() {
                let xi = polygon[i].long;
                let yi = polygon[i].lat;
                let xj = polygon[j].long;
                let yj = polygon[j].lat;
                let intersect = ((yi > point.lat) != (yj > point.lat))
                    && (point.long < (xj - xi) * (point.lat - yi) / (yj - yi) + xi);
                if intersect {
                    inside = !inside;
                }
                j = i;
            }
            inside
        }

        let mut matches = Vec::new();
        for request in self.get_requests() {
            let mut m: Vec<MapFeature> = request
                .processed_data
                .iter()
                .filter(|feature| {
                    point_in_polygon(
                        &Coord {
                            lat: feature.geometry.centroid().unwrap().y() as f32,
                            long: feature.geometry.centroid().unwrap().x() as f32,
                        },
                        polygon,
                    )
                })
                .cloned()
                .collect();
            matches.append(&mut m);
        }
        matches
    }
}

pub trait HaversineDistance {
    fn distance_haversine(&self, other: &Self) -> f64;
}

impl HaversineDistance for Coord {
    fn distance_haversine(&self, other: &Self) -> f64 {
        let radius_earth = 6_371_000.0_f64; // Earth's radius in meters

        let lat1 = self.lat.to_radians();
        let lon1 = self.long.to_radians();
        let lat2 = other.lat.to_radians();
        let lon2 = other.long.to_radians();

        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;

        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        radius_earth * c as f64
    }
}
