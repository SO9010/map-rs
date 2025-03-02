use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::{draw::{Fill, Stroke}, entity::{Path, ShapeBundle}, prelude::GeometryBuilder, shapes};

use crate::{tiles::{ChunkManager, ZoomManager}, types::{world_mercator_to_lat_lon, Coord}};

pub struct MeasurePlugin;

impl Plugin for MeasurePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Measure::default())
            .add_systems(Update, (render_measure, handle_measure));
    }
}

#[derive(Resource, Component, Clone)]
pub struct Measure {
    start: Option<Coord>,
    end: Option<Coord>,
    pub enabled: bool,
    pub respawn: bool,
}

impl Measure {
    pub fn get_in_world_space(&self, reference: Coord, zoom: u32, tile_quality: f64) -> Vec<Vec2> {
        let mut new_points = Vec::new();
        if self.start.is_some() {
            new_points.push(self.start.unwrap().to_game_coords(reference, zoom, tile_quality));
        }
        if self.end.is_some() {
            new_points.push(self.end.unwrap().to_game_coords(reference, zoom, tile_quality));
        }
        return new_points;
    }
        
    pub fn disable(&mut self) {
        *self = Measure {
            start: None,
            end: None,
            enabled: false,
            respawn: true,
        }
    }
}

/// These implementations are for constructors.
impl Measure {
    fn default() -> Self {
        Measure {
            start: None,
            end: None,
            enabled: false,
            respawn: false,
        }
    }
}

pub fn handle_measure(
    mut measure: ResMut<Measure>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    zoom_manager: Res<ZoomManager>,
    chunk_manager: Res<ChunkManager>,
) {
    let (camera, camera_transform) = camera.single();
    if measure.enabled {
        if let Some(position) = q_windows.single().cursor_position() {
            if buttons.just_pressed(MouseButton::Left) {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size);

                let start = Coord::new(pos.lat as f32, pos.long as f32);
                measure.start = Some(start);
                measure.respawn = true;
            }
            if buttons.pressed(MouseButton::Left) {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size);

                if measure.end != Some(Coord::new(pos.lat as f32, pos.long as f32)) {
                    measure.end = Some(Coord::new(pos.lat as f32, pos.long as f32));
                    measure.respawn = true;
                }
            }
            if buttons.just_released(MouseButton::Left) {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size);

                measure.end = Some(Coord::new(pos.lat as f32, pos.long as f32));
                measure.respawn = true;
            }
            if buttons.pressed(MouseButton::Right) {
                measure.start = None;
                measure.end = None;
                measure.respawn = true;
            }
        }
    }
}

#[derive(Component)]
pub struct MeasureMarker;

fn render_measure(
    mut commands: Commands,
    mut measure_query: Query<(Entity, &MeasureMarker)>,
    zoom_manager: Res<ZoomManager>,
    chunk_manager: Res<ChunkManager>,
    mut measure: ResMut<Measure>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if measure.respawn {
        measure.respawn = false;
        if let Ok((entity, _)) = measure_query.get_single_mut(){
            commands.entity(entity).despawn_recursive();
        }

        let fill_color = Srgba { red: 0.5, green: 0.5, blue: 0.5, alpha: 1. };
        let line_width = 2.5;
        let elevation = 1000.0;

        if measure.start.is_some() && measure.end.is_some() {
            let points: Vec<Vec2> = vec![
                measure.start.unwrap().to_game_coords(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into()),
                measure.end.unwrap().to_game_coords(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into()),
            ];
            let direction = points[1] - points[0];
            
            let angle = direction.y.atan2(direction.x);
            
            let midpoint = Vec3::new(
                (points[0].x + points[1].x) / 2.0,  // x midpoint
                (points[0].y + points[1].y) / 2.0,  // y midpoint
                elevation
            );
            
            let length = points[0].distance(points[1]);
            
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(length, line_width))),
                Transform::from_translation(midpoint)
                    .with_rotation(Quat::from_rotation_z(angle)),
                MeshMaterial2d(materials.add(Color::from(fill_color))),
                MeasureMarker,
            ));
        }
    }
}
