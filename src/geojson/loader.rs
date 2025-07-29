use std::{fs::File, io::BufReader};

use bevy::log::info;
use bevy_map_viewer::Coord;
use geojson::GeoJson;
use rstar::RTree;
use serde::{Deserialize, Serialize};

use super::MapFeature;

/// Parses OSM data from a string and returns a vector of map features.
pub fn get_data_from_string_osm(data: &str) -> Result<Vec<MapFeature>, Box<dyn std::error::Error>> {
    let response: OverpassResponse = serde_json::from_str(data)?;

    let mut features = Vec::new();

    for way in response.elements {
        /*
        if let Some(members) = way.members {
            for member in members {
                let tags = way.tags.clone().unwrap_or_default();
                if let Some(geo) = member.geometry {
                    info!("{:?}", tags);
                    features.push(MapFeature {
                        id: way.id.to_string(),
                        properties: tags.clone(),
                        closed: !geo.is_empty() && geo.first() == geo.last(),
                        geometry: geo::Polygon::new(geo::LineString(geo.into_iter().map(|p| geo::Coord { x: p.lat as f64, y: p.long as f64 }).collect()), vec![]),
                    });
                }
            }
            continue;
        }

        */
        if let Some(geo) = way.geometry {
            let tags = way.tags.unwrap_or_default();
            features.push(MapFeature {
                id: way.id.to_string(),
                properties: tags.clone(),
                closed: !geo.is_empty() && geo.first() == geo.last(),
                geometry: geo::Polygon::new(
                    geo::LineString(
                        geo.into_iter()
                            .map(|p| geo::Coord {
                                x: p.lat as f64,
                                y: p.long as f64,
                            })
                            .collect(),
                    ),
                    vec![],
                ),
            });
        }
    }

    Ok(features)
}

/// Parses OSM data from a string and returns a vector of map features. This takes in geojson data.
pub fn get_map_data(file_path: &str) -> Result<Vec<MapFeature>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let geojson = GeoJson::from_reader(reader)?;

    let mut features: Vec<MapFeature> = Vec::new();
    let mut geo = geo::Polygon::new(geo::LineString(vec![]), vec![]);
    if let GeoJson::FeatureCollection(collection) = geojson {
        for feature in collection.features {
            if let Some(geometry) = feature.geometry {
                let mut closed = true;
                match geometry.value {
                    geojson::Value::Polygon(poly) => {
                        if !poly.is_empty() {
                            let exterior = geo::LineString(
                                poly[0]
                                    .iter()
                                    .map(|p| geo::Coord { x: p[1], y: p[0] })
                                    .collect(),
                            );

                            let interiors: Vec<geo::LineString> = poly
                                .iter()
                                .skip(1) // Skip the exterior ring
                                .map(|ring| {
                                    geo::LineString(
                                        ring.iter()
                                            .map(|p| geo::Coord { x: p[1], y: p[0] })
                                            .collect(),
                                    )
                                })
                                .collect();
                            geo = geo::Polygon::new(exterior, interiors);
                        }
                    }
                    geojson::Value::MultiPolygon(multi_poly) => {
                        if !multi_poly.is_empty() && !multi_poly[0].is_empty() {
                            let exterior = geo::LineString(
                                multi_poly[0][0]
                                    .iter()
                                    .map(|p| geo::Coord { x: p[1], y: p[0] })
                                    .collect(),
                            );

                            let interiors: Vec<geo::LineString> = multi_poly[0]
                                .iter()
                                .skip(1) // Skip the exterior ring
                                .map(|ring| {
                                    geo::LineString(
                                        ring.iter()
                                            .map(|p| geo::Coord { x: p[1], y: p[0] })
                                            .collect(),
                                    )
                                })
                                .collect();
                            geo = geo::Polygon::new(exterior, interiors);
                        }
                    }
                    geojson::Value::LineString(line) => {
                        closed = false;
                        geo = geo::Polygon::new(
                            geo::LineString(
                                line.iter()
                                    .map(|p| geo::Coord { x: p[1], y: p[0] })
                                    .collect(),
                            ),
                            vec![],
                        );
                    }
                    _ => continue,
                }

                features.push(MapFeature {
                    id: feature.id.map_or_else(
                        || {
                            format!(
                                "{}_{:?}",
                                file_path,
                                feature.properties.clone().unwrap().get_key_value("entity")
                            )
                        },
                        |id| format!("{id:?}"),
                    ),
                    properties: serde_json::Value::Object(feature.properties.unwrap_or_default()),
                    closed,
                    geometry: geo.clone(),
                });
            }
        }
    }
    info!("Loaded {} features from {}", features.len(), file_path);
    Ok(features)
}

pub fn get_file_data(features: &mut RTree<MapFeature>, file_path: &str) {
    for feature in get_map_data(file_path).unwrap() {
        features.insert(feature);
    }
}

// Overpass API, thanks to: https://transform.tools/json-to-rust-serde
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OverpassResponse {
    pub version: Option<f64>,
    pub generator: Option<String>,
    pub osm3s: Option<Osm3s>,
    pub elements: Vec<Section>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Osm3s {
    #[serde(rename = "timestamp_osm_base")]
    pub timestamp_osm_base: Option<String>,
    #[serde(rename = "timestamp_areas_base")]
    pub timestamp_areas_base: Option<String>,
    pub copyright: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Section {
    #[serde(rename = "type")]
    pub type_field: String,
    pub id: i64,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub tags: Option<serde_json::Value>,
    pub bounds: Option<Bounds>,
    #[serde(default)]
    pub members: Option<Vec<Member>>,
    #[serde(default)]
    pub nodes: Option<Vec<i64>>,
    #[serde(default)]
    pub geometry: Option<Vec<Coord>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Bounds {
    pub minlat: f64,
    pub minlon: f64,
    pub maxlat: f64,
    pub maxlon: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "ref")]
    pub ref_field: i64,
    pub role: String,
    pub geometry: Option<Vec<Coord>>,
}
