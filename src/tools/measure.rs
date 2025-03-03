use bevy::{prelude::*, window::PrimaryWindow};

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
        new_points
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

#[derive(Component)]
pub struct MeasureTextMarker;

#[derive(Component)]
struct MeasureTextTranslation;

#[derive(Component)]
struct MeasureText;


fn render_measure(
    mut commands: Commands,
    mut measure_query: Query<(Entity, &MeasureMarker)>,
    text_query: Query<(Entity, &MeasureTextTranslation)>,
    mut text_trans: Query<&mut Transform, (With<Text2d>, With<MeasureTextTranslation>)>,
    mut measure_length: Query<&mut TextSpan, With<MeasureText>>,
    zoom_manager: Res<ZoomManager>,
    chunk_manager: Res<ChunkManager>,
    mut measure: ResMut<Measure>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>
) {
    if measure.respawn {
        measure.respawn = false;

        let fill_color = Srgba { red: 0.5, green: 0.5, blue: 0.5, alpha: 1. };
        let line_width = 2.5;
        let elevation = 1000.0;
        
        if let Ok((entity, _)) = measure_query.get_single_mut(){
                commands.entity(entity).despawn_recursive();
        } 

        if let Ok((entity, _text)) = text_query.get_single() {
            if measure.start.is_none() && measure.end.is_none() {
                commands.entity(entity).despawn_recursive();
            } else {
                for mut transform in &mut text_trans {
                    let points: Vec<Vec2> = vec![
                        measure.start.unwrap().to_game_coords(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into()),
                        measure.end.unwrap().to_game_coords(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into()),
                    ];
                    let direction = points[1] - points[0];
                    
                    let mut  angle = direction.y.atan2(direction.x);
                    if angle > 1.5 || angle < -1.6 {
                        angle -= 3.14;
                    }

                    let midpoint = Vec3::new(
                        (points[0].x + points[1].x) / 2.0,  // x midpoint
                        (points[0].y + points[1].y) / 2.0,  // y midpoint
                        elevation
                    );

                    let distance = measure.start.unwrap().distance(&measure.end.unwrap());
                    for mut span in &mut measure_length {
                        **span = format!("{:.5} {:?}", distance.0, distance.1);
                    }

                    transform.translation = midpoint;
                    transform.rotation = Quat::from_rotation_z(angle);
                };
            }
        } else if measure.start.is_some() && measure.end.is_some() {
            let points: Vec<Vec2> = vec![
                measure.start.unwrap().to_game_coords(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into()),
                measure.end.unwrap().to_game_coords(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into()),
            ];
            let direction = points[1] - points[0];
            
            let mut  angle = direction.y.atan2(direction.x);
            if angle > 1.5 || angle < -1.6 {
                angle -= 3.14;
            }

            let midpoint = Vec3::new(
                (points[0].x + points[1].x) / 2.0,  // x midpoint
                (points[0].y + points[1].y) / 2.0,  // y midpoint
                elevation
            );
            
            let font = asset_server.load("fonts/BagnardSans.otf");
            let text_font = TextFont {
                font: font.clone(),
                font_size: 15.0,
                ..default()
            };

            for mut span in &mut measure_length {
                **span = format!("{:.2} km", points[0].distance(points[1]) / 1000.0);
            }
            
            commands.spawn((
                Text2d::new(""),
                text_font,
                Transform::from_translation(midpoint)
                        .with_rotation(Quat::from_rotation_z(angle)),
                MeasureTextTranslation
            )).with_child((TextSpan::default(),
                TextColor(Color::BLACK),
                MeasureText,));
        }

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
