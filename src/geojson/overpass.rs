use std::io::{BufRead, BufReader, Read};

use bevy::prelude::*;
use geojson::{Geometry, Value};

use crate::{geojson::get_data_from_string_osm, types::{MapBundle, MapFeature, SettingsOverlay, WorldSpaceRect}};

fn build_overpass_query(bounds: Vec<WorldSpaceRect>, overpass_settings: &mut SettingsOverlay) -> String {
    let mut query = String::default();
    let opening = "[out:json];(";
    let closing = ");(._;>;);\nout body geom;";

    for bound in bounds {
        for (category, key) in overpass_settings.get_true_keys_with_category() {
            if key == "n/a" {
                continue;
            } else if key == "*" {
                query.push_str(&format!(r#"
                (
                way["{}"]({},{},{},{}); 
                );
                "#, category.to_lowercase(), bound.bottom_right.lat, bound.bottom_right.long, bound.top_left.lat, bound.top_left.long));
            } else {
                query.push_str(&format!(r#"
                (
                way["{}"="{}"]({},{},{},{}); 
                );
                "#, category.to_lowercase(), key.to_lowercase(), bound.bottom_right.lat, bound.bottom_right.long, bound.top_left.lat, bound.top_left.long));
            }
        }
    }

    if !query.is_empty() {
        query.insert_str(0, opening);
        query.push_str(closing);
    } else {
        return "ERR".to_string();
    }
    query
}

pub fn get_overpass_data<'a>(bounds: Vec<WorldSpaceRect>, map_bundle: &mut MapBundle, overpass_settings: &mut SettingsOverlay,
) -> Vec<MapFeature>  {
    if bounds.is_empty() {
        return vec![];
    }
    let query = build_overpass_query(bounds, overpass_settings);
    if query != "ERR" {
        return send_overpass_query(query, map_bundle)
    }
    vec![]
}

fn match_geometry(geom: &Geometry) {
    match geom.value {
        Value::Polygon(_) => println!("Matched a Polygon"),
        Value::MultiPolygon(_) => println!("Matched a MultiPolygon"),
        Value::GeometryCollection(ref gc) => {
            println!("Matched a GeometryCollection");
            // !!! GeometryCollections contain other Geometry types, and can
            // nest — we deal with this by recursively processing each geometry
            for geometry in gc {
                match_geometry(geometry)
            }
        }
        // Point, LineString, and their Multi– counterparts
        _ => println!("Matched some other geometry"),
    }
}

fn send_overpass_query(query: String, map_bundle: &mut MapBundle,
) -> Vec<MapFeature> {
    if query.is_empty() {
        return vec![];
    }
    let url = "https://overpass-api.de/api/interpreter";
    // let url = "http://localhost:12345/api/interpreter";
    info!("Sending query: {}", query);
    let mut status = 429;
    while status == 429 {
        if let Ok(response) = ureq::post(url).send_string(&query) {
            if response.status() == 200 {
                status = 200;
                let reader: BufReader<Box<dyn Read + Send + Sync>> = BufReader::new(response.into_reader());
            
                let mut response_body = String::default();
                info!("Finished query...");
                // Accumulate chunks into a single string
                for line in reader.lines() {
                    match line {
                        Ok(part) => response_body.push_str(part.as_str()),
                        Err(e) => {
                            info!("Error reading response: {}", e);
                            return vec![];
                        }
                    }
                }
    
                let map_features = map_bundle.features.clone();
                let features = get_data_from_string_osm(&response_body);
                info!("Status of rspns: {}", status);
                if features.is_ok() {
                    let new_features: Vec<_> = features.unwrap()
                    .into_iter()
                    .filter(|feature| {
                        !map_features
                            .iter()
                            .any(|existing| existing.id.contains(&feature.id))
                    })
                    .collect();
                    return new_features
                }
                return vec![]
            } else if response.status() == 429 {
                info!("Rate limited, waiting 5 seconds");
                std::thread::sleep(std::time::Duration::from_secs(5));
            } else {
                status = 0;
            }
        }
    }
    vec![]
}

