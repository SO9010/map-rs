use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::{draw::{Fill, Stroke}, entity::ShapeBundle, prelude::GeometryBuilder, shapes};
use rstar::AABB;

use crate::{camera::camera_space_to_lat_long_rect, tiles::TileMapResources, types::{MapBundle, SettingsOverlay}};

#[derive(Component)]
pub struct ShapeMarker;
// TODO: Change to a mesh 2d
pub fn respawn_shapes(
    mut commands: Commands,
    shapes_query: Query<(Entity, &ShapeMarker)>,
    mut map_bundle: ResMut<MapBundle>,
    tile_map_manager: Res<TileMapResources>,
    overpass_settings: Res<SettingsOverlay>,
    camera_query: Query<(&Camera, &GlobalTransform, &OrthographicProjection), With<Camera2d>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if map_bundle.respawn {
        map_bundle.respawn = false;
        for (entity, _) in shapes_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        let mut batch_commands_closed: Vec<(ShapeBundle, Fill, Stroke, ShapeMarker)> = Vec::new();
        let mut batch_commands_open: Vec<(ShapeBundle, Stroke, ShapeMarker)> = Vec::new();
        // Determine the viewport bounds
        let (_, camera_transform, zoom) = camera_query.single();
        let viewport: geo::Rect<f32> = camera_space_to_lat_long_rect(camera_transform, primary_window_query.single(), zoom.clone(), tile_map_manager.zoom_manager.zoom_level, tile_map_manager.zoom_manager.tile_size, tile_map_manager.chunk_manager.refrence_long_lat).unwrap();

        let viewport_aabb = AABB::from_corners(
            [viewport.min().x as f64, viewport.min().y as f64],
            [viewport.max().x as f64, viewport.max().y as f64],
        );
        
        let intersection_candidates = map_bundle.features.locate_in_envelope_intersecting(&viewport_aabb).collect::<Vec<_>>();
        
        for feature in intersection_candidates {
            let mut fill_color = Srgba { red: 0., green: 0.5, blue: 0., alpha: 0.5 };
            let mut stroke_color = Srgba { red: 0., green: 0.5, blue: 0., alpha: 0.75 };
            let line_width = 1.5;
            let elevation = 10.0;

            for (cat, key) in overpass_settings.get_true_keys_with_category_with_individual().iter() {
                if let Some(cate) = feature.properties.get(cat.to_lowercase()) {
                    if cate.as_str().unwrap() != key {
                        continue;
                    }
                    let color = overpass_settings.categories.get(cat).unwrap().items.get(key).unwrap().1;
                    stroke_color = Srgba { red: (color.r() as f32) / 210., green: (color.g() as f32) / 210., blue: (color.b() as f32) / 210., alpha: 0.50 };
                    fill_color = Srgba { red: (color.r() as f32) / 210., green: (color.g() as f32) / 210., blue: (color.b() as f32) / 210., alpha: 0.50 };
                }
            }

            let mut points = feature.get_in_world_space(tile_map_manager.chunk_manager.refrence_long_lat, tile_map_manager.zoom_manager.zoom_level, tile_map_manager.zoom_manager.tile_size.into());

            points.pop();

            if feature.closed {
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
                    Fill::color(fill_color),
                    Stroke::new(stroke_color, line_width as f32),
                    ShapeMarker,
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
                    ShapeMarker,
                ));
            }
        }

        commands.spawn_batch(batch_commands_closed);
        commands.spawn_batch(batch_commands_open);
    }
}