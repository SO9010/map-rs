use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, Align2, Checkbox, CornerRadius, RichText},
};
use bevy_map_viewer::{Coord, MapViewerMarker, TileMapResources, ZoomChangedEvent};
use rstar::{AABB, Envelope};

use crate::{
    tools::ToolResources,
    workspace::{SelectionType, Workspace, WorkspaceData},
};

pub fn workspace_actions_ui(
    mut tile_map_res: ResMut<TileMapResources>,
    mut contexts: EguiContexts,
    mut camera: Query<(&Camera, &mut Transform), With<MapViewerMarker>>,
    mut tools: ResMut<ToolResources>,
    mut zoom_event: EventWriter<ZoomChangedEvent>,
    mut workspace_res: ResMut<Workspace>,
) {
    let ctx = contexts.ctx_mut();

    let screen_rect = ctx.screen_rect();

    let tilebox_width: f32 = screen_rect.width();
    let tilebox_height = 25.0;

    let mut camera_transform = camera.single_mut().1;

    egui::Area::new("top_bar".into())
        .anchor(Align2::CENTER_TOP, [0.0, 0.0])
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 255))
                .corner_radius(CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: 10,
                    se: 10,
                })
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
                });
        });
    egui::Area::new("workspace_selector".into())
        .anchor(Align2::CENTER_TOP, [0.0, 2.5])
        .show(ctx, |ui| {
            egui::Frame::new().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.horizontal_centered(|ui| {
                        egui::ComboBox::from_id_salt("workspace_selector_box")
                            .selected_text(RichText::new(
                                workspace_res
                                    .workspace
                                    .clone()
                                    .unwrap_or(WorkspaceData::default())
                                    .get_name(),
                            ))
                            .show_ui(ui, |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    let selections_clone: Vec<_> =
                                        tools.selection_areas.areas.iter().cloned().collect();

                                    for workspace in selections_clone {
                                        let mut enabled = false;
                                        if workspace_res
                                            .workspace
                                            .as_ref()
                                            .unwrap_or(&WorkspaceData::default())
                                            .get_id()
                                            == workspace.get_id()
                                        {
                                            enabled = true;
                                        }
                                        if ui
                                            .checkbox(
                                                &mut enabled,
                                                RichText::new(workspace.get_name()),
                                            )
                                            .clicked()
                                        {
                                            let selection: super::Selection =
                                                workspace.get_selection();
                                            tile_map_res.location_manager.location =
                                                selection.start.unwrap_or_default();
                                            workspace_res.workspace = Some(workspace.clone());

                                            camera_transform.translation = Vec3::from(
                                                center(&selection, &tile_map_res).extend(0.0),
                                            );

                                            tools.selection_areas.respawn = true;
                                            zoom_event.send(ZoomChangedEvent);
                                        }
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Workspaces");
                                    if ui.button("Add").clicked() {}
                                });
                            });
                    });
                });
            });
        });

    egui::Area::new("tilemap_selector".into())
        .anchor(Align2::RIGHT_TOP, [-10.0, 2.5])
        .show(ctx, |ui| {
            egui::Frame::new().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.horizontal_centered(|ui| {
                        egui::ComboBox::from_id_salt("tilemap_selector_box")
                            .selected_text(RichText::new(
                                tile_map_res
                                    .tile_request_client
                                    .tile_web_origin
                                    .clone()
                                    .into_iter()
                                    .find(|item| item.1.0) // Find the item with enabled = true
                                    .map(|item| item.0.clone()) // Use the item's name if found
                                    .unwrap_or_else(|| "".to_string()), // Default text if none are enabled
                            ))
                            .show_ui(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    egui::ScrollArea::vertical().show(ui, |ui| {
                                        ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
                                        ui.set_max_width(tilebox_width);
                                        for (url, (enabled, _)) in &mut tile_map_res
                                            .tile_request_client
                                            .tile_web_origin
                                            .clone()
                                        {
                                            ui.vertical_centered(|ui| {
                                                ui.set_max_width(tilebox_width);
                                                if ui
                                                    .add_sized(
                                                        [tilebox_width - 10., 20.],
                                                        Checkbox::new(
                                                            enabled,
                                                            RichText::new(url.as_str()),
                                                        ),
                                                    )
                                                    .clicked()
                                                {
                                                    tile_map_res
                                                        .tile_request_client
                                                        .enable_only_tile_web_origin(url);
                                                }
                                            });
                                        }
                                    });
                                    ui.separator();
                                    if ui
                                        .button("Add URL -- Dummy")
                                        .on_hover_text("Add xyz map")
                                        .clicked()
                                    {}
                                });
                            });
                    });
                });
            });
        });
}

fn center(selection: &super::Selection, tile_map_res: &TileMapResources) -> Vec2 {
    match selection.selection_type {
        SelectionType::RECTANGLE => {
            let starting = selection
                .start
                .unwrap()
                .to_game_coords(tile_map_res.clone());
            let ending = selection.end.unwrap().to_game_coords(tile_map_res.clone());
            Vec2::new(
                starting.x - ((starting.x - ending.x) / 2.0),
                starting.y - ((starting.y - ending.y) / 2.0),
            )
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
            Coord::new(center[1] as f32, center[0] as f32).to_game_coords(tile_map_res.clone())
        }
        SelectionType::CIRCLE => selection
            .start
            .unwrap()
            .to_game_coords(tile_map_res.clone()),
        _ => Vec2::new(0.0, 0.0),
    }
}
