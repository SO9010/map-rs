use std::f32::consts::PI;

use bevy::{prelude::*, render::view::RenderLayers, window::PrimaryWindow};
use bevy_map_viewer::{Coord, EguiBlockInputState, MapViewerMarker, TileMapResources};
use bevy_prototype_lyon::{
    draw::Fill, entity::ShapeBundle, path::PathBuilder, prelude::GeometryBuilder,
};

use crate::types::{Selection, SelectionType, WorkspaceData};

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

pub fn handle_selection(
    mut tools: ResMut<ToolResources>,
    camera: Query<(&Camera, &GlobalTransform), With<MapViewerMarker>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    res_manager: ResMut<TileMapResources>,
    state: Res<EguiBlockInputState>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (camera, camera_transform) = camera.single();
    if tools.selection_settings.enabled {
        if let Some(position) = q_windows.single().cursor_position() {
            let pos = res_manager.point_to_coord(
                camera
                    .viewport_to_world_2d(camera_transform, position)
                    .unwrap(),
            );

            let mut name = String::new();
            if buttons.just_pressed(MouseButton::Left) && !state.block_input {
                if tools.selection_settings.tool_type == SelectionType::POLYGON {
                    if let Some(selection) = tools.selection_areas.unfinished_selection.as_mut() {
                        selection.points.as_mut().unwrap().push(pos);
                        tools.selection_areas.respawn = true;
                    } else {
                        tools.selection_areas.unfinished_selection = Some(Selection::new_poly(
                            tools.selection_settings.tool_type.clone(),
                            pos,
                        ));
                        tools.selection_areas.respawn = true;
                    }
                } else {
                    tools.selection_areas.unfinished_selection = Some(Selection::new(
                        tools.selection_settings.tool_type.clone(),
                        pos,
                        pos,
                    ));
                    tools.selection_areas.respawn = true;
                }
            }
            if buttons.pressed(MouseButton::Left)
                && tools.selection_settings.tool_type != SelectionType::POLYGON
            {
                if let Some(selection) = tools.selection_areas.unfinished_selection.as_mut() {
                    if selection.end != Some(Coord::new(pos.lat, pos.long)) {
                        selection.end = Some(Coord::new(pos.lat, pos.long));
                        tools.selection_areas.respawn = true;
                    }
                }
            }
            if !buttons.pressed(MouseButton::Left)
                && tools.selection_areas.unfinished_selection.is_some()
            {
                let areas_size = tools.selection_areas.areas.size();
                if tools.selection_settings.tool_type != SelectionType::POLYGON {
                    if let Some(selection) = tools.selection_areas.unfinished_selection.as_mut() {
                        if selection.end != selection.start {
                            selection.end = Some(Coord::new(pos.lat, pos.long));
                            name = format!("{:#?}-{}", selection.selection_type, areas_size);
                        } else {
                            tools.selection_areas.unfinished_selection = None;
                            tools.selection_areas.respawn = true;
                            return;
                        }
                    }
                    if let Some(selection) = tools.selection_areas.unfinished_selection.take() {
                        let workspace = WorkspaceData::new(name.clone(), selection);
                        let serded = serde_json::to_string(&workspace).unwrap();
                        info!("Serialized workspace: {}", serded);
                        tools.selection_areas.add(workspace);
                    }
                    tools.selection_areas.respawn = true;
                }
            }
            if buttons.pressed(MouseButton::Right) {
                tools.selection_areas.unfinished_selection = None;
                tools.selection_areas.respawn = true;
            }
            if keys.just_pressed(KeyCode::Enter)
                && tools.selection_settings.tool_type == SelectionType::POLYGON
            {
                if let Some(selection) = tools.selection_areas.unfinished_selection.take() {
                    tools
                        .selection_areas
                        .add(WorkspaceData::new(name, selection));
                }
            }
        }
    }
}

#[derive(Component)]
pub struct SelectionMarker;

// We want to darken everything else.
// We want to have the selected area to be on the side in a bar.
// TODO: change this to a shader, so that it gradually gets darker too. Also this causes some issues sometimes.
fn render_selection_box(
    mut gizmos: Gizmos,
    tools: ResMut<ToolResources>,
    res_manager: ResMut<TileMapResources>,
) {
    let mut intersection_candidates = tools
        .selection_areas
        .areas
        .clone()
        .into_iter()
        .collect::<Vec<_>>();

    if tools.selection_areas.unfinished_selection.is_some() {
        intersection_candidates.push(WorkspaceData::new(
            "unfinised".to_string(),
            tools
                .selection_areas
                .unfinished_selection
                .as_ref()
                .unwrap()
                .clone(),
        ));
    }

    let stroke_color = Color::srgba(0.5, 0.5, 0.9, 0.9); // Bright green
    let elevation = 1000.0; // Keep this far above other elements

    for feature in intersection_candidates {
        let points: Vec<Vec2> = feature
            .get_selection()
            .get_in_world_space(res_manager.clone());
        match feature.get_selection().selection_type {
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
                    stroke_color,
                );
                gizmos.line_2d(
                    Vec2::new(corners[1].x, corners[1].y),
                    Vec2::new(corners[2].x, corners[2].y),
                    stroke_color,
                );
                gizmos.line_2d(
                    Vec2::new(corners[2].x, corners[2].y),
                    Vec2::new(corners[3].x, corners[3].y),
                    stroke_color,
                );
                gizmos.line_2d(
                    Vec2::new(corners[3].x, corners[3].y),
                    Vec2::new(corners[0].x, corners[0].y),
                    stroke_color,
                );
            }
            SelectionType::POLYGON => {
                // Draw polygon by connecting points
                if points.len() >= 2 {
                    for i in 0..points.len() - 1 {
                        gizmos.line_2d(points[i], points[i + 1], stroke_color);
                    }

                    // Close the polygon
                    if points.len() >= 3 {
                        gizmos.line_2d(points[points.len() - 1], points[0], stroke_color);
                    }
                }
            }
            SelectionType::CIRCLE => {
                // Calculate center and radius
                let center = points[0];
                let radius = points[0].distance(points[1]);

                // Draw circle
                gizmos.circle_2d(center, radius, stroke_color);
            }
            _ => {}
        }
    }
}

#[derive(Component)]
pub struct DarkeningOverlay;

#[derive(Component)]
pub struct SelectionCutout;

fn render_darkening_overlay(
    mut commands: Commands,
    tools: Res<ToolResources>,
    res_manager: ResMut<TileMapResources>,
    camera_query: Query<
        (&Camera, &GlobalTransform, &OrthographicProjection),
        With<MapViewerMarker>,
    >,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    overlay_query: Query<Entity, With<DarkeningOverlay>>,
) {
    for entity in overlay_query.iter() {
        commands.entity(entity).despawn();
    }

    let mut intersection_candidates: Vec<Selection> = Vec::new();

    if let Some(selection_areas) = tools.selection_areas.focused_area.clone() {
        intersection_candidates.push(selection_areas.get_selection());
    }

    if tools.selection_areas.unfinished_selection.is_some() {
        intersection_candidates.push(
            tools
                .selection_areas
                .unfinished_selection
                .as_ref()
                .unwrap()
                .clone(),
        );
    }

    if intersection_candidates.is_empty() {
        return;
    }

    let (camera, camera_transform, _) = camera_query.single();
    let window = match primary_window_query.get_single() {
        Ok(window) => window,
        Err(_) => return, // Exit early if window not available
    };

    let top_left = camera
        .viewport_to_world_2d(camera_transform, Vec2::ZERO)
        .unwrap();
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
        let points = feature.get_in_world_space(res_manager.clone());

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
                path_builder.arc(center, Vec2::splat(radius), PI * 2.0, std::f32::consts::TAU);
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
        RenderLayers::layer(1),
    ));
}
