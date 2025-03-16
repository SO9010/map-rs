use std::io::{BufRead, BufReader, Read};
use bevy::prelude::*;

use crate::{geojson::get_data_from_string_osm, tools::{Selection, SelectionType}, types::{DistanceType, MapBundle, MapFeature, SettingsOverlay}};

use super::{OverpassReceiver, OverpassWorkerPlugin};

pub struct OverpassPlugin;
// TODO: Fix requests for routes, relations and points. THESE ARE A MATTER OF PRIORITY!
impl Plugin for OverpassPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapBundle::new())
            .add_plugins(OverpassWorkerPlugin)
            .add_systems(FixedUpdate, read_overpass_receiver);
    }
}

pub fn build_overpass_query_string(bounds: String, overpass_settings: &mut SettingsOverlay) -> String {
    let mut query = String::default();
    let opening = "[out:json];";
    let closing = "\nout body geom;";

    for (category, key) in overpass_settings.get_true_keys_with_category() {
            if key == "n/a" {
                continue;
            } else if key == "*" {
                query.push_str(&format!(r#"
                (
                way["{}"]({bounds}); 
                node["{}"]({bounds}); 
                relation["{}"]({bounds}); 
                );
                "#, category.to_lowercase(), category.to_lowercase(), category.to_lowercase()));
            } else {
                query.push_str(&format!(r#"
                (
                way["{}"="{}"]({bounds}); 
                node["{}"="{}"]({bounds}); 
                relation["{}"="{}"]({bounds}); 
                );
                "#, category.to_lowercase(), key.to_lowercase(), category.to_lowercase(), key.to_lowercase(), category.to_lowercase(), key.to_lowercase()));
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


pub fn get_overpass_query(selection: Selection, overpass_settings: &mut SettingsOverlay) -> String {
    let mut bounds: String = String::default();
    if selection.points.is_some() {
        if selection.selection_type == SelectionType::POLYGON {
            let points_string = selection.points.unwrap().iter().map(|point| {
                format!("{} {}", point.lat, point.long)
            }).collect::<Vec<String>>().join(" ");
            bounds = format!("poly:\"{}\"", points_string);
        }
    } else if selection.start.is_some() && selection.end.is_some() {
        match selection.selection_type {
            SelectionType::RECTANGLE => {
                let start = selection.start.unwrap();
                let end = selection.end.unwrap();
                bounds = format!(
                    "poly:\"{} {} {} {} {} {} {} {}\"", 
                    start.lat, start.long,
                    start.lat, end.long,
                    end.lat, end.long,
                    end.lat, start.long
                );       
            },
            SelectionType::CIRCLE => {
                let start = selection.start.unwrap();
                let end = selection.end.unwrap();
                let (mut dist, dist_type) = start.distance(&end);
                match dist_type {
                    DistanceType::Km => dist *= 1000.0,
                    DistanceType::M => {},
                    DistanceType::CM => dist /= 100.0,
                }
                bounds = format!("around:{}, {}, {}", dist, start.lat, start.long);
            }
            _ => {}
        }
    }

    let query = build_overpass_query_string(bounds, overpass_settings);
    if query != "ERR" {
        return query;
    } 
    "ERR".to_string()
}

pub fn send_overpass_query(query: String) -> Vec<MapFeature> {
    if query.is_empty() {
        return vec![];
    }
    let url = "https://overpass-api.de/api/interpreter";
    // let url = "http://localhost:12345/api/interpreter";
    let mut status = 429;
    while status == 429 {
        if let Ok(response) = ureq::post(url).send_string(&query) {
            if response.status() == 200 {
                let reader: BufReader<Box<dyn Read + Send + Sync>> = BufReader::new(response.into_reader());
            
                let mut response_body = String::default();
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
    
                let features = get_data_from_string_osm(&response_body);
                if let Ok(features) = features {
                    return features;
                } else {
                    info!("Error parsing response: {:?}", features.err());
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

pub fn read_overpass_receiver(
    map_receiver: Option<Res<OverpassReceiver>>,
    mut map_bundle: ResMut<MapBundle>,
) {
    if let Some(map_receiver) = map_receiver {
        if let Ok(v) = map_receiver.0.try_recv() {
            for feature in &v {
                map_bundle.features.insert(feature.clone());
            }
            map_bundle.respawn = true;
        }
    }
}