use bevy::{prelude::*, window::PrimaryWindow};
use rstar::{RTree, RTreeObject, AABB};
use bevy_prototype_lyon::{draw::{Fill, Stroke}, entity::{Path, ShapeBundle}, prelude::GeometryBuilder, shapes};

use crate::{tiles::{ChunkManager, ZoomManager}, types::{world_mercator_to_lat_lon, Coord}};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectionSettings::default())
            .insert_resource(SelectionAreas::new())
            .add_systems(Update, (render_selection_box, handle_selection));
    }
}

#[derive(Resource)]
pub struct SelectionSettings {
    pub selection_tool_type: SelectionType,
    pub selection_enabled: bool,
} 

impl SelectionSettings {
    pub fn default() -> Self {
        Self {
            selection_tool_type: SelectionType::CIRCLE,
            selection_enabled: false,
        }
    }
}

/// The goal for this module is to provide a way to select a region of the map, to be able to select featurtes in that region.
/// For example someone should be able to select an eare for turbo overpass data to be downloaded. 
/// Or this area can be selected to modify how the map looks in that area.
/// Or this will be used for the user to select their work space area, with in this the data will be permentanly stored and the user can modify it.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionType {
    NONE,
    RECTANGLE,
    POLYGON,
    CIRCLE,
}

// What we will want to do is have a ui for the selection points. We also want to be able to select it by clicking the edge then we can risize or annotate or change the things.
#[derive(Resource)]
pub struct SelectionAreas {
    areas: RTree<Selection>,
    unfinished_selection: Option<Selection>,
    pub respawn: bool,
}

impl SelectionAreas {
    pub fn new() -> Self {
        Self { 
            areas: RTree::new(),
            unfinished_selection: None,
            respawn: false
        }
    }
    
    fn add(&mut self, selection: Selection) {
        self.areas.insert(selection);
    }
}

#[derive(Component, Clone)]
struct Selection {
    pub selection_type: SelectionType,
    pub start: Option<Coord>,
    pub end: Option<Coord>,
    pub points: Option<Vec<Coord>>,
}

impl Selection {
    pub fn get_in_world_space(&self, reference: Coord, zoom: u32, tile_quality: f64) -> Vec<Vec2> {
        if self.points.is_some() {
            let mut new_points = Vec::new();
            for point in self.points.as_ref().unwrap() {
                new_points.push(point.to_game_coords(reference, zoom, tile_quality));
            }
            return new_points;
        } else {
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
}

/// These implementations are for constructors.
impl Selection {
    pub fn new(selection_type: SelectionType, start: Coord, end: Coord) -> Self {
        Self {
            selection_type,
            start: Some(start),
            end: Some(end),
            points: None,
        }
    }
}

/// These implementations are for the RTreeObject trait.
impl RTreeObject for Selection {
    type Envelope = AABB<[f64; 2]>;
    
    fn envelope(&self) -> Self::Envelope {
        match self.selection_type {
            SelectionType::RECTANGLE => AABB::from_corners([self.start.unwrap().long, self.start.unwrap().lat], [self.end.unwrap().long, self.end.unwrap().lat]),
            SelectionType::CIRCLE => AABB::from_corners([self.start.unwrap().long, self.start.unwrap().lat], [self.end.unwrap().long, self.end.unwrap().lat]),
            SelectionType::POLYGON => {
                let mut min = [f64::MAX, f64::MAX];
                let mut max = [f64::MIN, f64::MIN];
                for point in self.points.as_ref().unwrap() {
                    if point.long < min[0] as f32 {
                        min[0] = point.long as f64 ;
                    }
                    if point.lat < min[1] as f32 {
                        min[1] = point.lat as f64;
                    }
                    if point.long > max[0] as f32 {
                        max[0] = point.long as f64;
                    }
                    if point.lat > max[1] as f32 {
                        max[1] = point.lat as f64;
                    }
                }
                return AABB::from_corners(min, max);
            },
            _ => AABB::from_corners([0.0, 0.0], [0.0, 0.0]),
        };
        AABB::from_corners([0.0, 0.0], [0.0, 0.0])
    }
}

pub fn handle_selection(
    mut selections: ResMut<SelectionAreas>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    zoom_manager: Res<ZoomManager>,
    chunk_manager: Res<ChunkManager>,
    selection_settings: Res<SelectionSettings>,
) {
    let (camera, camera_transform) = camera.single();
    if selection_settings.selection_enabled {
        if let Some(position) = q_windows.single().cursor_position() {
            // TODO ADD POLYGON SELECTION
            if buttons.just_pressed(MouseButton::Left) {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size);

                let start = Coord::new(pos.lat as f32, pos.long as f32);
                selections.unfinished_selection = Some(Selection::new(selection_settings.selection_tool_type.clone(), start, start));
                selections.respawn = true;
            }
            if buttons.pressed(MouseButton::Left) {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size);

                if let Some(selection) = selections.unfinished_selection.as_mut() {
                    if selection.end != Some(Coord::new(pos.lat as f32, pos.long as f32)) {
                        selection.end = Some(Coord::new(pos.lat as f32, pos.long as f32));
                        selections.respawn = true;
                    }
                }
            }
            if buttons.just_released(MouseButton::Left) {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size);

                if let Some(selection) = selections.unfinished_selection.as_mut() {
                    selection.end = Some(Coord::new(pos.lat as f32, pos.long as f32));
                }
                if let Some(selection) = selections.unfinished_selection.take() {
                    selections.add(selection);
                }
                selections.respawn = true;
            }
            if buttons.pressed(MouseButton::Right) {
                selections.unfinished_selection = None;
                selections.respawn = true;
            }
        }
    }
}

fn render_selection_box(
    mut commands: Commands,
    mut selections: ResMut<SelectionAreas>,
    selections_query: Query<(Entity, &Path, &GlobalTransform, &Selection)>,
    zoom_manager: Res<ZoomManager>,
    chunk_manager: Res<ChunkManager>,
) {
    if selections.respawn {
        selections.respawn = false;
        for (entity, _, _, _) in selections_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        let mut batch_commands_closed: Vec<(ShapeBundle, Fill, Stroke, Selection)> = Vec::new();

        let mut intersection_candidates = selections.areas.clone().into_iter().collect::<Vec<_>>();
        
        if selections.unfinished_selection.is_some() {
            intersection_candidates.push(selections.unfinished_selection.as_ref().unwrap().clone());
        }

        for feature in intersection_candidates {
            let fill_color = Srgba { red: 0., green: 0.5, blue: 0., alpha: 0.5 };
            let stroke_color = Srgba { red: 0., green: 0.5, blue: 0., alpha: 0.75 };
            let line_width = 5.;
            let elevation = 10.0;

            let points: Vec<Vec2> = feature.get_in_world_space(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into());
            match feature.selection_type {
                SelectionType::RECTANGLE => {
                    let shape: Vec<Vec2> = vec![
                        Vec2::new(points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min), points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min)),
                        Vec2::new(points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max), points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max)),
                    ];
                    let shape: Vec<Vec2> = vec![
                        Vec2::new(shape[0].x, shape[0].y),
                        Vec2::new(shape[1].x, shape[0].y),
                        Vec2::new(shape[1].x, shape[1].y),
                        Vec2::new(shape[0].x, shape[1].y),
                    ];

                    let shape = shapes::Polygon {
                        points: shape.clone(),
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
                        feature.clone(),
                    ));
                },
                SelectionType::POLYGON => {
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
                        feature.clone(),
                    ));
                },
                SelectionType::CIRCLE => {
                    let shape = shapes::Circle {
                        radius: (points[0] - points[1]).length(),
                        center: points[0],
                    };
                    batch_commands_closed.push((
                        ShapeBundle {
                            path: GeometryBuilder::build_as(&shape),
                            transform: Transform::from_xyz(0.0, 0.0, elevation),
                            ..default()
                        },
                        Fill::color(fill_color),
                        Stroke::new(stroke_color, line_width as f32),
                        feature.clone(),
                    ));
                },
                _ => {},
            }
        }

        commands.spawn_batch(batch_commands_closed);
    }
}
