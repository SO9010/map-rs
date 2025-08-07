use bevy::{prelude::*, window::PrimaryWindow};
use bevy_map_viewer::{Coord, EguiBlockInputState, MapViewerMarker, TileMapResources};

use crate::{
    camera::PostProcessSettings,
    workspace::{Selection, SelectionType, Workspace, WorkspaceData},
};

use super::ToolResources;

/// A plugin that provides functionality for selecting areas on the map.
/// This includes handling user input for creating selections and rendering them.
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
            enabled: false,
        }
    }
}

/// Handles user input for creating and modifying selections on the map.
/// Supports rectangle, polygon, and circle selection types.
pub fn handle_selection(
    mut tools: ResMut<ToolResources>,
    camera: Query<(&Camera, &GlobalTransform), With<MapViewerMarker>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    res_manager: ResMut<TileMapResources>,
    state: Res<EguiBlockInputState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut workspace: ResMut<Workspace>,
) {
    let (camera, camera_transform) = match camera.single() {
        Ok(result) => result,
        Err(_) => return,
    };
    if tools.selection_settings.enabled {
        if let Some(position) = q_windows
            .single()
            .expect("Couldn't get cursor position")
            .cursor_position()
        {
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
                        let _ = workspace.save_workspace();
                    } else {
                        tools.selection_areas.unfinished_selection = Some(Selection::new_poly(
                            tools.selection_settings.tool_type.clone(),
                            pos,
                        ));
                        tools.selection_areas.respawn = true;
                        let _ = workspace.save_workspace();
                    }
                } else {
                    tools.selection_areas.unfinished_selection = Some(Selection::new(
                        tools.selection_settings.tool_type.clone(),
                        pos,
                        pos,
                    ));
                    tools.selection_areas.respawn = true;
                    let _ = workspace.save_workspace();
                }
            }
            if buttons.pressed(MouseButton::Left)
                && tools.selection_settings.tool_type != SelectionType::POLYGON
            {
                if let Some(selection) = tools.selection_areas.unfinished_selection.as_mut() {
                    if selection.end != Some(Coord::new(pos.lat, pos.long)) {
                        selection.end = Some(Coord::new(pos.lat, pos.long));
                        tools.selection_areas.respawn = true;
                        let _ = workspace.load_workspace();
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
                        let workspace_data = WorkspaceData::new(name.clone(), selection);
                        let serded = serde_json::to_string(&workspace_data).unwrap();
                        info!("Serialized workspace: {}", serded);
                        tools.selection_areas.add(workspace_data);
                        let _ = workspace.save_workspace();
                    }
                    tools.selection_areas.respawn = true;
                }
            }
            if buttons.pressed(MouseButton::Right) {
                tools.selection_areas.unfinished_selection = None;
                tools.selection_areas.respawn = true;
                let _ = workspace.save_workspace();
            }
            if keys.just_pressed(KeyCode::Enter)
                && tools.selection_settings.tool_type == SelectionType::POLYGON
            {
                if let Some(selection) = tools.selection_areas.unfinished_selection.take() {
                    tools
                        .selection_areas
                        .add(WorkspaceData::new(name, selection));
                    let _ = workspace.save_workspace();
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

/// Renders the selection box or shape on the map.
/// This includes rectangles, polygons, and circles.
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

/// TODO: Renders a darkening overlay on the map, excluding the selected areas.
/// This helps visually highlight the selected regions.
/// Hmm change to something else, lyon isnt working anymore do try and use a custom shader
/// TODO: use a shader for this, so that it can be animated
#[allow(unused_variables)]
#[allow(unused_mut)]
fn render_darkening_overlay(
    tools: Res<ToolResources>,
    res_manager: ResMut<TileMapResources>,
    camera_query: Query<(&Camera, &GlobalTransform, &Projection), With<MapViewerMarker>>,
    primary_window_query: Query<&Window, With<PrimaryWindow>>,
    overlay_query: Query<Entity, With<DarkeningOverlay>>,
    workspace_res: Res<Workspace>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
    mut settings: Query<&mut PostProcessSettings>,
) {
    // Now spawn "clear" shapes over selections
    let mut intersection_candidates: Vec<Selection> = Vec::new();
    if let Some(selection_areas) = workspace_res.workspace.clone() {
        intersection_candidates.push(selection_areas.get_selection());
    }
    if let Some(unfinished) = tools.selection_areas.unfinished_selection.as_ref() {
        intersection_candidates.push(unfinished.clone());
    }

    if intersection_candidates.is_empty() {
        for mut setting in &mut settings {
            setting.on = 0;
        }
        return;
    }
    for mut setting in &mut settings {
        setting.on = 1;
    }
    /*
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

                    let vertices = vec![
                        [min_x, min_y, 401.0],
                        [max_x, min_y, 401.0],
                        [max_x, max_y, 401.0],
                        [min_x, max_y, 401.0],
                    ];
                    let indices = vec![0, 1, 2, 0, 2, 3];

                    let clear_mesh = Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::default(),
                    )
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone())
                    .with_inserted_indices(Indices::U32(indices.clone()));

                    commands.spawn((
                        Mesh2d(meshes.add(clear_mesh)),
                        MeshMaterial2d(
                            materials.add(ColorMaterial::from_color(Srgba::new(0., 0., 0., 0.0))),
                        ), // fully transparent
                        Transform::from_translation(Vec3::new(0.0, 0.0, 1.1)), // slightly in front
                        DarkeningOverlay,
                        RenderLayers::layer(1),
                    ));
                }
                SelectionType::CIRCLE => {
                    if points.len() < 2 {
                        continue;
                    }
                    let center = points[0];
                    let radius = points[0].distance(points[1]);

                    let segments = 32;
                    let mut vertices = Vec::new();
                    let mut indices = Vec::new();
                    for i in 0..segments {
                        let angle = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                        vertices.push([
                            center.x + radius * angle.cos(),
                            center.y + radius * angle.sin(),
                            401.0,
                        ]);
                    }
                    for i in 1..(segments - 1) {
                        indices.extend(vec![0, i as u32, (i + 1) as u32]);
                    }

                    let clear_mesh = Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::default(),
                    )
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone())
                    .with_inserted_indices(Indices::U32(indices.clone()));

                    commands.spawn((
                        Mesh2d(meshes.add(clear_mesh)),
                        MeshMaterial2d(
                            materials.add(ColorMaterial::from_color(Srgba::new(0., 0., 0., 0.0))),
                        ), // fully transparent
                        Transform::from_translation(Vec3::new(0.0, 0.0, 1.1)),
                        DarkeningOverlay,
                        RenderLayers::layer(1),
                    ));
                }
                SelectionType::POLYGON => {
                    if points.len() < 3 {
                        continue;
                    }
                    let mut vertices = Vec::new();
                    let mut indices = Vec::new();
                    for point in &points {
                        vertices.push([point.x, point.y, 401.0]);
                    }
                    for i in 1..(points.len() - 1) {
                        indices.extend(vec![0, i as u32, (i + 1) as u32]);
                    }

                    let clear_mesh = Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::default(),
                    )
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone())
                    .with_inserted_indices(Indices::U32(indices.clone()));

                    commands.spawn((
                        Mesh2d(meshes.add(clear_mesh)),
                        MeshMaterial2d(
                            materials.add(ColorMaterial::from_color(Srgba::new(0., 0., 0., 0.0))),
                        ), // fully transparent
                        Transform::from_translation(Vec3::new(0.0, 0.0, 1.1)),
                        DarkeningOverlay,
                        RenderLayers::layer(1),
                    ));
                }
                _ => {}
            }
        //}
        */
}
