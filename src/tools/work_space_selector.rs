use std::f32::consts::PI;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_prototype_lyon::{draw::Fill, entity::ShapeBundle, path::PathBuilder, prelude::GeometryBuilder};
use rstar::{RTree, RTreeObject, AABB};

use crate::{tiles::TileMapResources, types::{world_mercator_to_lat_lon, Coord}, EguiBlockInputState};
use super::ToolResources;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_selection)
            .add_systems(PostUpdate, (render_selection_box, render_darkening_overlay));
    }
}

pub struct SelectionSettings {
    pub tool_type: SelectionType,
    pub enabled: bool,
} 

impl Default for SelectionSettings {
    fn default() -> Self {
        Self {
            tool_type: SelectionType::CIRCLE,
            enabled: true,
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

impl SelectionType {
    pub fn iterate(&mut self) {
        match self {
            SelectionType::NONE => *self = SelectionType::RECTANGLE,
            SelectionType::RECTANGLE => *self = SelectionType::POLYGON,
            SelectionType::POLYGON => *self = SelectionType::CIRCLE,
            SelectionType::CIRCLE => *self = SelectionType::RECTANGLE,
        }
    }
}

// What we will want to do is have a ui for the selection points. We also want to be able to select it by clicking the edge then we can risize or annotate or change the things.
pub struct SelectionAreas {
    pub areas: RTree<Selection>,
    unfinished_selection: Option<Selection>,
    pub respawn: bool,
}

impl Default for SelectionAreas {
    fn default() -> Self {
        Self::new()
    }
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

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Selection {
    pub selection_name: String,
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
            new_points
        } else {
            let mut new_points = Vec::new();
            if self.start.is_some() {
                new_points.push(self.start.unwrap().to_game_coords(reference, zoom, tile_quality));
            }
            if self.end.is_some() {
                new_points.push(self.end.unwrap().to_game_coords(reference, zoom, tile_quality));
            }
            new_points
        }
    }
}

/// These implementations are for constructors.
impl Selection {
    pub fn new(selection_type: SelectionType, start: Coord, end: Coord) -> Self {
        Self {
            selection_name: format!("{:#?}-{:#?}", selection_type, start),
            selection_type,
            start: Some(start),
            end: Some(end),
            points: None,
        }
    }

    pub fn new_poly(selection_type: SelectionType, start: Coord) -> Self {
        Self {
            selection_name: format!("{:#?}-{:#?}", selection_type, start),
            selection_type,
            start: None,
            end: None,
            points: Some(vec![start]),
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
    mut tools: ResMut<ToolResources>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    res_manager: ResMut<TileMapResources>,
    state: Res<EguiBlockInputState>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (camera, camera_transform) = camera.single();
    if tools.selection_settings.enabled {
        if let Some(position) = q_windows.single().cursor_position() {
            if buttons.just_pressed(MouseButton::Left) && !state.block_input {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), res_manager.chunk_manager.refrence_long_lat, res_manager.zoom_manager.zoom_level, res_manager.zoom_manager.tile_size);

                let point = Coord::new(pos.lat as f32, pos.long as f32);

                if tools.selection_settings.tool_type == SelectionType::POLYGON {
                    if let Some(selection) = tools.selection_areas.unfinished_selection.as_mut() {
                        selection.points.as_mut().unwrap().push(point);
                        tools.selection_areas.respawn = true;
                    } else {
                        tools.selection_areas.unfinished_selection = Some(Selection::new_poly(tools.selection_settings.tool_type.clone(), point));
                        tools.selection_areas.respawn = true;
                    }
                } else {
                    tools.selection_areas.unfinished_selection = Some(Selection::new(tools.selection_settings.tool_type.clone(), point, point));
                    tools.selection_areas.respawn = true;
                }
            }
            if buttons.pressed(MouseButton::Left) {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), res_manager.chunk_manager.refrence_long_lat, res_manager.zoom_manager.zoom_level, res_manager.zoom_manager.tile_size);

                if tools.selection_settings.tool_type != SelectionType::POLYGON {
                    if let Some(selection) = tools.selection_areas.unfinished_selection.as_mut() {
                        if selection.end != Some(Coord::new(pos.lat as f32, pos.long as f32)) {
                            selection.end = Some(Coord::new(pos.lat as f32, pos.long as f32));
                            tools.selection_areas.respawn = true;
                        }
                    }
                }
            }
            if !buttons.pressed(MouseButton::Left) && tools.selection_areas.unfinished_selection.is_some() {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), res_manager.chunk_manager.refrence_long_lat, res_manager.zoom_manager.zoom_level, res_manager.zoom_manager.tile_size);
                let areas_size = tools.selection_areas.areas.size();

                if tools.selection_settings.tool_type != SelectionType::POLYGON {
                    if let Some(selection) = tools.selection_areas.unfinished_selection.as_mut() {
                        if selection.end != selection.start {
                            selection.end = Some(Coord::new(pos.lat as f32, pos.long as f32));
                            selection.selection_name = format!("{:#?}-{}", selection.selection_type, areas_size);
                        } else {
                            tools.selection_areas.unfinished_selection = None;
                            tools.selection_areas.respawn = true;
                            return;
                        }
                    }
                    if let Some(selection) = tools.selection_areas.unfinished_selection.take() {
                        tools.selection_areas.add(selection);
                    }
                    tools.selection_areas.respawn = true;
                }
            }
            if buttons.pressed(MouseButton::Right) {
                tools.selection_areas.unfinished_selection = None;
                tools.selection_areas.respawn = true;
            }
            if keys.just_pressed(KeyCode::Enter) {
                if tools.selection_settings.tool_type == SelectionType::POLYGON {
                    if let Some(selection) = tools.selection_areas.unfinished_selection.take() {
                        tools.selection_areas.add(selection);
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct SelectionMarker;

// TODO: We want to have the currently selected area to be highlighted.
// We want to darken everything else.
// We want to have the selected area to be on the side in a bar.
fn render_selection_box(
    mut gizmos: Gizmos,
    tools: ResMut<ToolResources>,
    res_manager: ResMut<TileMapResources>,
) {
    let mut intersection_candidates = tools.selection_areas.areas.clone().into_iter().collect::<Vec<_>>();
    
    if tools.selection_areas.unfinished_selection.is_some() {
        intersection_candidates.push(tools.selection_areas.unfinished_selection.as_ref().unwrap().clone());
    }

    let stroke_color = Color::srgba(0.5, 0.5, 0.9, 0.9); // Bright green
    let elevation = 1100.0; // Keep this slightly above other elements

    for feature in intersection_candidates {
        let points: Vec<Vec2> = feature.get_in_world_space(
            res_manager.chunk_manager.refrence_long_lat, 
            res_manager.zoom_manager.zoom_level, 
            res_manager.zoom_manager.tile_size.into()
        );

        match feature.selection_type {
            SelectionType::RECTANGLE => {
                // Calculate rectangle corners
                let min_x = points[0].x.min(points[1].x);
                let max_x = points[0].x.max(points[1].x);
                let min_y = points[0].y.min(points[1].y);
                let max_y = points[0].y.max(points[1].y);

                // Create rectangle corners
                let corners = [
                    Vec3::new(min_x, min_y, elevation),
                    Vec3::new(max_x, min_y, elevation),
                    Vec3::new(max_x, max_y, elevation),
                    Vec3::new(min_x, max_y, elevation),
                ];

                // Draw rectangle
                gizmos.line_2d(
                    Vec2::new(corners[0].x, corners[0].y),
                    Vec2::new(corners[1].x, corners[1].y),
                    stroke_color
                );
                gizmos.line_2d(
                    Vec2::new(corners[1].x, corners[1].y),
                    Vec2::new(corners[2].x, corners[2].y),
                    stroke_color
                );
                gizmos.line_2d(
                    Vec2::new(corners[2].x, corners[2].y),
                    Vec2::new(corners[3].x, corners[3].y),
                    stroke_color
                );
                gizmos.line_2d(
                    Vec2::new(corners[3].x, corners[3].y),
                    Vec2::new(corners[0].x, corners[0].y),
                    stroke_color
                );
            },
            SelectionType::POLYGON => {
                // Draw polygon by connecting points
                if points.len() >= 2 {
                    for i in 0..points.len() - 1 {
                        gizmos.line_2d(
                            points[i],
                            points[i + 1],
                            stroke_color
                        );
                    }
                    
                    // Close the polygon
                    if points.len() >= 3 {
                        gizmos.line_2d(
                            points[points.len() - 1],
                            points[0],
                            stroke_color
                        );
                    }
                }
            },
            SelectionType::CIRCLE => {
                // Calculate center and radius
                let center = points[0];
                let radius = points[0].distance(points[1]);
                
                // Draw circle
                gizmos.circle_2d(center, radius, stroke_color);
            },
            _ => {},
        }
    }
}


#[derive(Component)]
pub struct DarkeningOverlay;

#[derive(Component)]
pub struct SelectionCutout;

fn render_darkening_overlay(
    mut commands: Commands,
    mut tools: ResMut<ToolResources>,
    res_manager: ResMut<TileMapResources>,
    camera_query: Query<(&Camera, &GlobalTransform, &OrthographicProjection), With<Camera2d>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    overlay_query: Query<Entity, With<DarkeningOverlay>>,
) {
    if tools.selection_areas.respawn {
        tools.selection_areas.respawn = false;

        for entity in overlay_query.iter() {
            commands.entity(entity).despawn();
        }
    
        let mut intersection_candidates = tools.selection_areas.areas.clone().into_iter().collect::<Vec<_>>();
        
        if tools.selection_areas.unfinished_selection.is_some() {
            intersection_candidates.push(tools.selection_areas.unfinished_selection.as_ref().unwrap().clone());
        }
        
        if intersection_candidates.is_empty() {
            return;
        }
        
        let (camera, camera_transform, _) = camera_query.single();
        let window = match primary_window_query.get_single() {
            Ok(window) => window,
            Err(_) => return, // Exit early if window not available
        };
        
        let top_left = camera.viewport_to_world_2d(camera_transform, Vec2::ZERO).unwrap();
        let bottom_right = camera
            .viewport_to_world_2d(camera_transform, Vec2::new(window.width(), window.height()))
            .unwrap();
    
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(top_left);
        path_builder.line_to(Vec2::new(bottom_right.x, top_left.y));
        path_builder.line_to(bottom_right);
        path_builder.line_to(Vec2::new(top_left.x, bottom_right.y));
        path_builder.close();
    
        for feature in intersection_candidates {
            let points = feature.get_in_world_space(
                res_manager.chunk_manager.refrence_long_lat,
                res_manager.zoom_manager.zoom_level,
                res_manager.zoom_manager.tile_size.into(),
            );
    
            match feature.selection_type {
                SelectionType::RECTANGLE => {
                    if points.len() < 2 {
                        continue;
                    }
                    let min_x = points[0].x.min(points[1].x);
                    let max_x = points[0].x.max(points[1].x);
                    let min_y = points[0].y.min(points[1].y);
                    let max_y = points[0].y.max(points[1].y);
    
                    path_builder.move_to(Vec2::new(min_x, min_y));
                    path_builder.line_to(Vec2::new(max_x, min_y));
                    path_builder.line_to(Vec2::new(max_x, max_y));
                    path_builder.line_to(Vec2::new(min_x, max_y));
                    path_builder.close();
                }
                SelectionType::CIRCLE => {
                    if points.len() < 2 {
                        continue;
                    }
                    let center = points[0];
                    let radius = points[0].distance(points[1]);
    
                    path_builder.move_to(center + Vec2::new(radius, 0.0));
                    path_builder.arc(center, Vec2::splat(radius), PI*2.0, std::f32::consts::TAU);
                    path_builder.close();
                }
                SelectionType::POLYGON => {
                    if points.len() < 3 {
                        continue;
                    }
    
                    path_builder.move_to(points[0]);
                    for &point in points.iter().skip(1) {
                        path_builder.line_to(point);
                    }
                    path_builder.close();
                }
                _ => {}
            }
        }
    
        // Build the final shape
        let path = path_builder.build();
    
        // Spawn the darkening overlay with holes
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&path),
                transform: Transform::from_xyz(0.0, 0.0, 900.0),
                ..default()
            },
            Fill::color(Color::srgba(0.0, 0.0, 0.0, 0.5)), // Dark overlay with holes
            DarkeningOverlay,
        ));
    }
}
