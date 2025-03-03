use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::{draw::{Fill, Stroke}, entity::ShapeBundle, prelude::GeometryBuilder, shapes};
use rstar::AABB;

use crate::{camera::camera_space_to_lat_long_rect, tiles::{ChunkManager, ZoomManager}, types::MapBundle};

// TODO: we need to make it so it only renders aproximations when we zoom
#[derive(Component)]
pub struct ShapeMarker;

pub fn respawn_shapes(
    mut commands: Commands,
    shapes_query: Query<(Entity, &ShapeMarker)>,
    mut map_bundle: ResMut<MapBundle>,
    zoom_manager: Res<ZoomManager>,
    chunk_manager: Res<ChunkManager>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    query: Query<&mut OrthographicProjection, With<Camera>>,
) {
    if map_bundle.respawn {
        map_bundle.respawn = false;
        for (entity, _) in shapes_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        let mut batch_commands_closed: Vec<(ShapeBundle, Fill, Stroke, ShapeMarker)> = Vec::new();
        let mut batch_commands_open: Vec<(ShapeBundle, Stroke, ShapeMarker)> = Vec::new();
        // Determine the viewport bounds
        let (_, camera_transform) = camera_query.single();
        let viewport: geo::Rect<f32> = camera_space_to_lat_long_rect(camera_transform, primary_window_query.single(), query.single().clone(), zoom_manager.zoom_level, zoom_manager.tile_size, chunk_manager.refrence_long_lat).unwrap();

        let viewport_aabb = AABB::from_corners(
            [viewport.min().x as f64, viewport.min().y as f64],
            [viewport.max().x as f64, viewport.max().y as f64],
        );
        
        let intersection_candidates = map_bundle.features.locate_in_envelope_intersecting(&viewport_aabb).collect::<Vec<_>>();
        
        for feature in intersection_candidates {
            let fill_color = Srgba { red: 0., green: 0.5, blue: 0., alpha: 0.5 };
            let stroke_color = Srgba { red: 0., green: 0.5, blue: 0., alpha: 0.75 };
            let line_width = 1.5;
            let elevation = 10.0;

            let mut points = feature.get_in_world_space(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into());

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