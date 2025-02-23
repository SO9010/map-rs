use std::io::{BufRead, BufReader, Read};

use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use bevy_prototype_lyon::{draw::{Fill, Stroke}, entity::{Path, ShapeBundle}, prelude::GeometryBuilder, shapes};
use crossbeam_channel::{bounded, Receiver};
use geojson::{Geometry, Value};
use rstar::AABB;

use crate::{camera::camera_space_to_lat_long_rect, geojson::get_data_from_string_osm, tiles::{ChunkManager, ZoomManager}, types::{Coord, MapBundle, MapFeature, SettingsOverlay, WorldSpaceRect}};

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
        info!("{}", query);
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
    } else {
        info!("Error building query");
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
    let mut status = 429;
    while status == 429 {
        if let Ok(response) = ureq::post(url).send_string(&query) {
            if response.status() == 200 {
                status = 200;
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

pub fn respawn_overpass_map(
    mut commands: Commands,
    shapes_query: Query<(Entity, &Path, &GlobalTransform, &MapFeature)>,
    overpass_settings: Res<SettingsOverlay>,
    mut map_bundle: ResMut<MapBundle>,
    zoom_manager: Res<ZoomManager>,
    chunk_manager: Res<ChunkManager>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    query: Query<&mut OrthographicProjection, With<Camera>>,
) {
    if map_bundle.respawn {
        map_bundle.respawn = false;

        for (entity, _, _, _) in shapes_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        let mut batch_commands_closed: Vec<(ShapeBundle, Fill, Stroke, MapFeature)> = Vec::new();
        let mut batch_commands_open: Vec<(ShapeBundle, Stroke, MapFeature)> = Vec::new();

        // Determine the viewport bounds
        let (_, camera_transform) = camera_query.single();
        let viewport: geo::Rect<f32> = camera_space_to_lat_long_rect(camera_transform, primary_window_query.single(), query.single().clone(), zoom_manager.zoom_level, zoom_manager.tile_size, chunk_manager.refrence_long_lat).unwrap();

        let viewport_aabb = AABB::from_corners(
            [viewport.min().x as f64, viewport.min().y as f64],
            [viewport.max().x as f64, viewport.max().y as f64],
        );
        let intersection_candidates = map_bundle.features.locate_in_envelope_intersecting(&viewport_aabb).collect::<Vec<_>>();


        let disabled_setting = overpass_settings.get_disabled_categories();
        let enabled_setting = overpass_settings.get_true_keys_with_category_with_individual();

        // Group features by category and key, the string is thing to look for
        // (cat, key)
        let mut feature_groups: HashMap<(String, String), Vec<&MapFeature>> = HashMap::new();
        
        for feature in &map_bundle.features {
            for (cat, key) in &enabled_setting {
                if !disabled_setting.contains(cat) {
                    feature_groups.entry((cat.to_string(), key.to_string())).or_default().push(feature);
                }
            }
        }
        
        for feature in intersection_candidates {
            let mut fill_color= Some(Srgba { red: 0.4, green: 0.400, blue: 0.400, alpha: 1.0 });
            let mut stroke_color = Srgba { red: 0.50, green: 0.500, blue: 0.500, alpha: 1.0 };
            let mut line_width = 1.0;
            let mut elevation = 10.0;
            let closed = true;
            for ((cat, key), _) in &feature_groups {
                if key != "*" && feature.properties.get(cat.to_lowercase()).map_or(false, |v| *v == *key.to_lowercase()) {
                    let color = overpass_settings.categories.get(cat).unwrap().items.get(key).unwrap().1;
                    fill_color = Some(Srgba { red: (color.r() as f32) / 255., green: (color.g() as f32) / 255., blue: (color.b() as f32) / 255., alpha: 1.0 });
                    stroke_color = Srgba { red: (color.r() as f32) / 210., green: (color.g() as f32) / 210., blue: (color.b() as f32) / 210., alpha: 1.0 };
                    if cat == "Highway" || cat == "Railway" {
                        fill_color = None;
                        line_width = 2.5;
                        elevation = 0.;

                        // When zoomed out we should make the primary roads bigger, and the motorways even bigger.
                        if feature.properties.get("highway").map_or(false, |v| v == "residential" || v == "primary" || v == "secondary" || v == "tertiary") {
                            line_width = 5.5;
                        }
                        
                        feature.properties.get("est_width").map_or((), |v| {
                            // line_width = v.as_str().unwrap().replace("\"", "").parse::<f64>().unwrap() as f64;
                        });
                    }

                    if map_bundle.selected_features.contains(feature) {
                        line_width *= 1.25;
                        stroke_color = Srgba { red: 2.5, green: 2.5, blue: 2.5, alpha: 1.0 };
                    } 

                    let mut points = feature.get_in_world_space(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into());

                    points.pop();                            
        
                    if let Some(fill) = fill_color {
                        let shape = shapes::Polygon {
                            points: points.clone(),
                            closed: true,
                        };
                        batch_commands_closed.push((
                            ShapeBundle {
                                path: GeometryBuilder::build_as(&shape),
                                transform: Transform::from_xyz(0.0, 0.0, elevation),
                                ..default()
                            },
                            Fill::color(fill),
                            Stroke::new(stroke_color, line_width as f32),
                            feature.clone(),
                        ));
                    } else {
                        let shape = shapes::Polygon {
                            points: points.clone(),
                            closed: false,
                        };
                        batch_commands_open.push((
                            ShapeBundle {
                                path: GeometryBuilder::build_as(&shape),
                                transform: Transform::from_xyz(0.0, 0.0, elevation),
                                ..default()
                            },
                            Stroke::new(stroke_color, line_width as f32),
                            feature.clone(),
                        ));
                    }
                }
            }
        }

        commands.spawn_batch(batch_commands_closed);
        commands.spawn_batch(batch_commands_open);
    }
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
        let (camera, camera_transform) = query.single();
        let window = primary_window_query.single();

        if let Some(viewport) = camera_space_to_lat_long_rect(camera_transform, window, ortho_projection_query.single().clone(), zoom_manager.zoom_level, zoom_manager.tile_size, chunk_manager.refrence_long_lat) {
            // Here we need to go through the bounding boxes and check if we have already gotten this bounding box 
            let (tx, rx) = bounded::<Vec<MapFeature>>(10);
            let tx_clone = tx.clone();
            let mut map_bundle_clone = map_bundle.clone();
            let mut overpass_settings_clone = overpass_settings.clone();
            let converted_rect = WorldSpaceRect {
                top_left: Coord::new(viewport.max().x, viewport.max().y),
                bottom_right: Coord::new(viewport.min().x, viewport.min().y),
            };
            info!("{:?}", converted_rect);
            std::thread::spawn(move || {
                //tx.send(get_map_data("green-belt.geojson").unwrap());

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

pub fn read_overpass_receiver(
    map_receiver: Option<Res<OverpassReceiver>>, // Use Option<Res<OverpassReceiver>> to handle missing resource
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