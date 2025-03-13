use std::f32::consts::PI;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{tiles::TileMapResources, types::{world_mercator_to_lat_lon, Coord}, EguiBlockInputState};

use super::ToolResources;

pub struct MeasurePlugin;

impl Plugin for MeasurePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (render_measure, handle_measure));
    }
}

#[derive(Component, Clone, Default)]
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

pub fn handle_measure(
    mut measure: ResMut<ToolResources>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    tile_map_manager: Res<TileMapResources>,
    state: Res<EguiBlockInputState>,
) {
    let (camera, camera_transform) = camera.single();
    if measure.measure.enabled {
        if let Some(position) = q_windows.single().cursor_position() {
            if buttons.just_pressed(MouseButton::Left) && !state.block_input {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), tile_map_manager.chunk_manager.refrence_long_lat, tile_map_manager.zoom_manager.zoom_level, tile_map_manager.zoom_manager.tile_size);

                let start = Coord::new(pos.lat as f32, pos.long as f32);
                measure.measure.start = Some(start);
                measure.measure.respawn = true;
            }
            if buttons.pressed(MouseButton::Left) {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), tile_map_manager.chunk_manager.refrence_long_lat, tile_map_manager.zoom_manager.zoom_level, tile_map_manager.zoom_manager.tile_size);

                if measure.measure.end != Some(Coord::new(pos.lat as f32, pos.long as f32)) {
                    measure.measure.end = Some(Coord::new(pos.lat as f32, pos.long as f32));
                    measure.measure.respawn = true;
                }
            }
            if buttons.just_released(MouseButton::Left) {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), tile_map_manager.chunk_manager.refrence_long_lat, tile_map_manager.zoom_manager.zoom_level, tile_map_manager.zoom_manager.tile_size);

                measure.measure.end = Some(Coord::new(pos.lat as f32, pos.long as f32));
                measure.measure.respawn = true;
            }
            if buttons.pressed(MouseButton::Right) {
                measure.measure.start = None;
                measure.measure.end = None;
                measure.measure.respawn = true;
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

// Find a way to reduce this by say using a ParamSet
#[allow(clippy::too_many_arguments)]
fn render_measure(
    mut commands: Commands,
    mut measure_query: Query<(Entity, &MeasureMarker)>,
    mut text_trans: Query<&mut Transform, (With<Text2d>, With<MeasureTextTranslation>)>,
    mut measure_length: Query<&mut TextSpan, With<MeasureText>>,
    mut tool_res: ResMut<ToolResources>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    text_query: Query<(Entity, &MeasureTextTranslation)>,
    tile_map_manager: Res<TileMapResources>,
) {
    if tool_res.measure.respawn {
        tool_res.measure.respawn = false;

        let fill_color = Srgba { red: 0.75, green: 0., blue: 0., alpha: 1. };
        let line_width = 2.5;
        let elevation = 1000.0;
        
        if let Ok((entity, _)) = measure_query.get_single_mut(){
                commands.entity(entity).despawn_recursive();
        } 

        if let Ok((entity, _text)) = text_query.get_single() {
            if tool_res.measure.start.is_none() && tool_res.measure.end.is_none() {
                commands.entity(entity).despawn_recursive();
            } else {
                for mut transform in &mut text_trans {
                    let points: Vec<Vec2> = vec![
                        tool_res.measure.start.unwrap().to_game_coords(tile_map_manager.chunk_manager.refrence_long_lat, tile_map_manager.zoom_manager.zoom_level, tile_map_manager.zoom_manager.tile_size.into()),
                        tool_res.measure.end.unwrap().to_game_coords(tile_map_manager.chunk_manager.refrence_long_lat, tile_map_manager.zoom_manager.zoom_level, tile_map_manager.zoom_manager.tile_size.into()),
                    ];
                    let direction = points[1] - points[0];
                    
                    let mut  angle = direction.y.atan2(direction.x);
                    if !(-1.6..=1.5).contains(&angle) {
                        angle -= PI;
                    }

                    let midpoint = Vec3::new(
                        (points[0].x + points[1].x) / 2.0,  // x midpoint
                        (points[0].y + points[1].y) / 2.0,  // y midpoint
                        elevation
                    );

                    let distance = tool_res.measure.start.unwrap().distance(&tool_res.measure.end.unwrap());
                    for mut span in &mut measure_length {
                        **span = format!("{:.3} {:?}", distance.0, distance.1);
                    }

                    transform.translation = midpoint;
                    transform.rotation = Quat::from_rotation_z(angle);
                };
            }
        } else if tool_res.measure.start.is_some() && tool_res.measure.end.is_some() {
            let points: Vec<Vec2> = vec![
                tool_res.measure.start.unwrap().to_game_coords(tile_map_manager.chunk_manager.refrence_long_lat, tile_map_manager.zoom_manager.zoom_level, tile_map_manager.zoom_manager.tile_size.into()),
                tool_res.measure.end.unwrap().to_game_coords(tile_map_manager.chunk_manager.refrence_long_lat, tile_map_manager.zoom_manager.zoom_level, tile_map_manager.zoom_manager.tile_size.into()),
            ];
            let direction = points[1] - points[0];
            
            let mut  angle = direction.y.atan2(direction.x);
            if !(-1.6..=1.5).contains(&angle) {
                angle -= PI;
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

        if tool_res.measure.start.is_some() && tool_res.measure.end.is_some() {
            let points: Vec<Vec2> = vec![
                tool_res.measure.start.unwrap().to_game_coords(tile_map_manager.chunk_manager.refrence_long_lat, tile_map_manager.zoom_manager.zoom_level, tile_map_manager.zoom_manager.tile_size.into()),
                tool_res.measure.end.unwrap().to_game_coords(tile_map_manager.chunk_manager.refrence_long_lat, tile_map_manager.zoom_manager.zoom_level, tile_map_manager.zoom_manager.tile_size.into()),
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
