use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPreUpdateSet,
    egui::{self, Color32, RichText},
};
use bevy_map_viewer::{Coord, MapViewerMarker, TileMapResources, ZoomChangedEvent};
use rstar::{AABB, Envelope};
use uuid::Uuid;

use crate::{
    overpass::{build_overpass_query_string, get_bounds},
    workspace::{RequestType, SelectionType, Workspace, WorkspaceRequest},
};

use super::ToolResources;

/// A plugin that provides UI components for interacting with tools and workspaces.
/// This includes a toolbar for tool selection and a workspace management panel.
pub struct ToolbarUiPlugin;

impl Plugin for ToolbarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tool_ui.after(EguiPreUpdateSet::InitContexts),
                workspace_actions_ui.after(EguiPreUpdateSet::InitContexts),
            ),
        );
    }
}

/// Renders the toolbar UI for selecting tools such as workspace, measure, and pin.
fn tool_ui(mut tools: ResMut<ToolResources>, mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();

    let toolbar_width = 225.0;
    let toolbar_height = 40.0;

    let screen_rect = ctx.screen_rect();
    let toolbar_pos = egui::pos2(
        (screen_rect.width() - toolbar_width) / 2.0,
        screen_rect.height() - toolbar_height - 10.0,
    );

    egui::Area::new("toolbar".into())
        .fixed_pos(toolbar_pos)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 220))
                .corner_radius(10.0)
                .shadow(egui::epaint::Shadow {
                    color: egui::Color32::from_black_alpha(60),
                    offset: [5, 5],
                    blur: 10,
                    spread: 5,
                })
                .show(ui, |ui| {
                    ui.set_width(toolbar_width);
                    ui.set_height(toolbar_height);

                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(8.0, 0.0);
                        // Tool buttons
                        let button_selected = |selected: bool, text: &str| {
                            if selected {
                                egui::Button::new(RichText::new(text).color(Color32::WHITE))
                                    .fill(Color32::from_rgb(70, 130, 180))
                                    .corner_radius(8.0)
                            } else {
                                egui::Button::new(RichText::new(text).color(Color32::WHITE))
                                    .fill(Color32::from_rgb(40, 40, 40))
                                    .corner_radius(8.0)
                            }
                        };

                        ui.add(egui::Label::new(""));

                        if ui
                            .add_sized(
                                [64.0, 30.0],
                                button_selected(tools.selection_settings.enabled, "Workspace"),
                            )
                            .clicked()
                        {
                            if tools.selection_settings.enabled {
                                tools.selection_settings.tool_type.iterate();
                            }
                            tools.select_tool("workspace");
                        }
                        if ui
                            .add_sized(
                                [64.0, 30.0],
                                button_selected(tools.measure.enabled, "Measure"),
                            )
                            .clicked()
                        {
                            tools.select_tool("measure");
                        }
                        if ui
                            .add_sized([64.0, 30.0], button_selected(tools.pins.enabled, "Pin"))
                            .clicked()
                        {
                            tools.select_tool("pins");
                        }
                    });
                });
        });
}

/// Renders the workspace management UI, allowing users to interact with and manage workspaces.
fn workspace_actions_ui(
    mut tile_map_res: ResMut<TileMapResources>,
    mut contexts: EguiContexts,
    mut camera: Query<(&Camera, &mut Transform), With<MapViewerMarker>>,
    mut tools: ResMut<ToolResources>,
    mut zoom_event: EventWriter<ZoomChangedEvent>,
    mut workspace_res: ResMut<Workspace>,
) {
    let ctx = contexts.ctx_mut();

    let tilebox_width = 200.0;
    let tilebox_height = 100.0;

    let screen_rect = ctx.screen_rect();
    let tilebox_pos = egui::pos2(screen_rect.width() - 210., 10.0);

    let mut camera_transform = camera.single_mut().1;

    egui::Area::new("Workspace".into())
        .fixed_pos(tilebox_pos)
        .fade_in(true)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 255))
                .corner_radius(10.0)
                .shadow(egui::epaint::Shadow {
                    color: egui::Color32::from_black_alpha(60),
                    offset: [5, 5],
                    blur: 10,
                    spread: 5,
                })
                .show(ui, |ui| {
                    ui.set_min_width(tilebox_width);
                    ui.set_min_height(tilebox_height);
                    ui.set_max_width(tilebox_width);
                    ui.set_max_height(tilebox_height);

                    ui.vertical_centered(|ui| {
                        ui.add(egui::Label::new(
                            RichText::new("Workspaces").color(Color32::WHITE),
                        ));
                        ui.separator();

                        if let Some(workspace) = &tools.selection_areas.focused_area {
                            if ui.button(RichText::new(workspace.get_name())).clicked() {
                                let selection = workspace.get_selection();

                                match selection.selection_type {
                                    SelectionType::RECTANGLE => {
                                        let starting = selection
                                            .start
                                            .unwrap()
                                            .to_game_coords(tile_map_res.clone());
                                        let ending = selection
                                            .end
                                            .unwrap()
                                            .to_game_coords(tile_map_res.clone());
                                        let movement = Coord::new(
                                            starting.x - ((starting.x - ending.x) / 2.0),
                                            starting.y - ((starting.y - ending.y) / 2.0),
                                        );
                                        camera_transform.translation =
                                            movement.to_vec2().extend(1.0);
                                    }
                                    SelectionType::POLYGON => {
                                        let mut min = [f64::MAX, f64::MAX];
                                        let mut max = [f64::MIN, f64::MIN];
                                        for point in selection.points.as_ref().unwrap() {
                                            if point.long < min[0] as f32 {
                                                min[0] = point.long as f64;
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
                                        let center = AABB::from_corners(min, max).center().to_vec();
                                        let movement =
                                            Coord::new(center[1] as f32, center[0] as f32)
                                                .to_game_coords(tile_map_res.clone());
                                        camera_transform.translation = movement.extend(1.0);
                                    }
                                    SelectionType::CIRCLE => {
                                        let starting = selection
                                            .start
                                            .unwrap()
                                            .to_game_coords(tile_map_res.clone());
                                        camera_transform.translation =
                                            Vec3::new(starting.x, starting.y, 1.0);
                                    }
                                    _ => {}
                                }
                                let q = build_overpass_query_string(
                                    get_bounds(selection.clone()),
                                    workspace_res.overpass_agent.settings.clone(),
                                );
                                if let Ok(query) = q {
                                    let request = WorkspaceRequest::new(
                                        Uuid::new_v4().to_string(),
                                        1,
                                        RequestType::OverpassTurboRequest(query),
                                        Vec::new(),
                                    );
                                    workspace_res.worker.queue_request(request);
                                    tools.selection_areas.respawn = true;
                                }

                                zoom_event.send(ZoomChangedEvent);
                            }
                        }
                        ui.separator();
                    });

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            let available_width = tilebox_width - 10.0; // 10px padding on each side
                            ui.set_max_width(available_width);

                            let selections_clone: Vec<_> =
                                tools.selection_areas.areas.iter().cloned().collect();

                            for workspace in selections_clone {
                                ui.vertical_centered(|ui| {
                                    ui.set_max_width(tilebox_width - 10.);
                                    let mut enabled = false;
                                    if tools.selection_areas.focused_area.is_some() {
                                        enabled = true;
                                    }
                                    if ui
                                        .checkbox(&mut enabled, RichText::new(workspace.get_name()))
                                        .clicked()
                                    {
                                        let selection = workspace.get_selection();
                                        tile_map_res.location_manager.location =
                                            selection.start.unwrap_or_default();
                                        tools.selection_areas.focused_area =
                                            Some(workspace.clone());
                                        workspace_res.workspace = Some(workspace.clone());
                                        match selection.selection_type {
                                            SelectionType::RECTANGLE => {
                                                let starting = selection
                                                    .start
                                                    .unwrap()
                                                    .to_game_coords(tile_map_res.clone());
                                                let ending = selection
                                                    .end
                                                    .unwrap()
                                                    .to_game_coords(tile_map_res.clone());
                                                let movement = Coord::new(
                                                    starting.x - ((starting.x - ending.x) / 2.0),
                                                    starting.y - ((starting.y - ending.y) / 2.0),
                                                );
                                                camera_transform.translation =
                                                    movement.to_vec2().extend(1.0);
                                            }
                                            SelectionType::POLYGON => {
                                                let mut min = [f64::MAX, f64::MAX];
                                                let mut max = [f64::MIN, f64::MIN];
                                                for point in selection.points.as_ref().unwrap() {
                                                    if point.long < min[0] as f32 {
                                                        min[0] = point.long as f64;
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
                                                let center =
                                                    AABB::from_corners(min, max).center().to_vec();
                                                let movement =
                                                    Coord::new(center[1] as f32, center[0] as f32)
                                                        .to_game_coords(tile_map_res.clone());
                                                camera_transform.translation = movement.extend(1.0);
                                            }
                                            SelectionType::CIRCLE => {
                                                let starting = selection
                                                    .start
                                                    .unwrap()
                                                    .to_game_coords(tile_map_res.clone());
                                                camera_transform.translation =
                                                    Vec3::new(starting.x, starting.y, 1.0);
                                            }
                                            _ => {}
                                        }
                                        tools.selection_areas.respawn = true;
                                        zoom_event.send(ZoomChangedEvent);
                                    }
                                });
                            }
                        });
                    });
                });
        });
}

// Soon make overpass query. Also to view selected points and waypoints. This will be big when this is done.
// Try and add more apis like weather. We want to be able to add layers.
