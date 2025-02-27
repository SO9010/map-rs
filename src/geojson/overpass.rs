use std::io::{BufRead, BufReader, Read};

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::{draw::Fill, entity::ShapeBundle, prelude::GeometryBuilder, shapes};
use crossbeam_channel::{bounded, Receiver};

use crate::{camera::camera_space_to_lat_long_rect, geojson::get_data_from_string_osm, tiles::{ChunkManager, ZoomManager}, types::{Coord, MapBundle, MapFeature, SettingsOverlay, WorldSpaceRect}};

use super::get_map_data;

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
        info!("Sending query: {}", query);
        let result = send_overpass_query(query, map_bundle);
        info!("Got {} features", result.len());
        return result;
    } else {
    }
    vec![]
}

fn send_overpass_query(query: String, map_bundle: &mut MapBundle,
) -> Vec<MapFeature> {
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
    
                let map_features = map_bundle.features.clone();
                let features = get_data_from_string_osm(&response_body);
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

#[derive(Resource, Deref)]
pub struct OverpassReceiver(Receiver<Vec<MapFeature>>);

pub fn bbox_system(
    mut commands: Commands,
    query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    ortho_projection_query: Query<&mut OrthographicProjection, With<Camera>>,
    mut map_bundle: ResMut<MapBundle>,
    overpass_settings: ResMut<SettingsOverlay>,
    zoom_manager: Res<ZoomManager>,
    chunk_manager: Res<ChunkManager>,
) {
    if map_bundle.get_more_data {
        map_bundle.get_more_data = false;
        let (_camera, camera_transform) = query.single();
        let window = primary_window_query.single();

        if let Some(viewport) = camera_space_to_lat_long_rect(camera_transform, window, ortho_projection_query.single().clone(), zoom_manager.zoom_level, zoom_manager.tile_size, chunk_manager.refrence_long_lat) {
            // Here we need to go through the bounding boxes and check if we have already gotten this bounding box 
            let (tx, rx) = bounded::<Vec<MapFeature>>(10);
            let _tx_clone = tx.clone();
            let mut map_bundle_clone = map_bundle.clone();
            let mut overpass_settings_clone = overpass_settings.clone();
            let converted_rect = WorldSpaceRect {
                top_left: Coord::new(viewport.max().x, viewport.max().y),
                bottom_right: Coord::new(viewport.min().x, viewport.min().y),
            };
            std::thread::spawn(move || {
                let _ = tx.send(get_overpass_data(vec![converted_rect], &mut map_bundle_clone, &mut overpass_settings_clone));
            });

            let shape = shapes::RoundedPolygon {
                points: vec![
                    Vec2::new(viewport.min().x, viewport.max().y),
                    Vec2::new(viewport.max().x, viewport.max().y),
                    Vec2::new(viewport.max().x, viewport.min().y),
                    Vec2::new(viewport.min().x, viewport.min().y),
                ],
                radius: 25.0,
                closed: true,
            };
            commands.spawn((ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                transform: Transform::from_xyz(0.0, 0.0, -0.1),
                ..default()
            },
                Fill::color(Srgba {red: 0.071, green: 0.071, blue: 0.071, alpha: 1.0 })
            ));
            commands.insert_resource(OverpassReceiver(rx));

        }
    }
}

pub fn get_green_belt_data(
    mut commands: Commands,
    mut map_bundle: ResMut<MapBundle>,
) {
    if map_bundle.get_green_data {
        map_bundle.get_green_data = false;

        let (tx, rx) = bounded::<Vec<MapFeature>>(10);
        let _tx_clone = tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(get_map_data("assets/test/green-belt.geojson").unwrap());
        });

        commands.insert_resource(OverpassReceiver(rx));

    }
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