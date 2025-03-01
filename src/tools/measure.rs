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
}

/// These implementations are for constructors.
impl Measure {
    fn default() -> Self {
        Measure {
            start: None,
            end: None,
            enabled: true,
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
        }
    }
}

#[derive(Component)]
pub struct MeasureText;

fn render_measure(
    mut commands: Commands,
    selections_query: Query<(Entity, &Path, &GlobalTransform, &Measure)>,
    zoom_manager: Res<ZoomManager>,
    chunk_manager: Res<ChunkManager>,
    mut measure: ResMut<Measure>,
    asset_server: Res<AssetServer>,
) {
    if measure.respawn {
        measure.respawn = false;
        for (entity, _, _, _) in selections_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        let fill_color = Srgba { red: 0., green: 0.5, blue: 0., alpha: 0.5 };
        let stroke_color = Srgba { red: 0., green: 0.5, blue: 0., alpha: 0.75 };
        let line_width = 5.;
        let elevation = 10.0;

        if measure.start.is_some() && measure.end.is_some() {
            let points: Vec<Vec2> = vec![
                measure.start.unwrap().to_game_coords(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into()),
                measure.end.unwrap().to_game_coords(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into()),
            ];
    
            let shape = shapes::Polygon {
                points: points.clone(),
                closed: true,
            };
     
            commands.spawn((
                ShapeBundle {
                    path: GeometryBuilder::build_as(&shape),
                    transform: Transform::from_xyz(0.0, 0.0, elevation),
                    ..default()
                },
                Fill::color(fill_color),
                Stroke::new(stroke_color, line_width as f32),
                measure.clone(),
            ));
        }
    }
}
