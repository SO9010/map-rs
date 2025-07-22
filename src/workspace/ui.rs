use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, Align2, Checkbox, CornerRadius, RichText},
};
use bevy_map_viewer::{
    Coord, EguiBlockInputState, MapViewerMarker, TileMapResources, ZoomChangedEvent, game_to_coord,
};
use rstar::{AABB, Envelope, RTreeObject};
use uuid::Uuid;

use crate::{
    geojson::MapFeature,
    overpass::{build_overpass_query_string, get_bounds},
    tools::ToolResources,
    workspace::{SelectionType, Workspace, WorkspaceData},
};

use super::{RequestType, WorkspaceRequest};

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

    if let Ok((_, mut camera_transform)) = camera.single_mut() {
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
                                        .unwrap_or_default()
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
                                                if workspace_res
                                                    .workspace
                                                    .clone()
                                                    .unwrap_or_default()
                                                    .get_id()
                                                    != workspace.get_id()
                                                {
                                                    workspace_res.workspace =
                                                        Some(workspace.clone());
                                                }

                                                // Center the camera on the selection
                                                let selection: super::Selection =
                                                    workspace.get_selection();
                                                tile_map_res.location_manager.location =
                                                    selection.start.unwrap_or_default();
                                                camera_transform.translation =
                                                    center(&selection, &tile_map_res).extend(0.0);

                                                // Make request to the overpass server
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

                                                tools.selection_areas.respawn = true;
                                                zoom_event.write(ZoomChangedEvent);
                                            }
                                        }
                                    });
                                    // https://blog.afi.io/blog/how-to-draw-and-view-boundary-data-with-openstreetmap-osm/
                                    /*
                                    ui.horizontal(|ui| {
                                        ui.label("Workspaces");
                                        if ui.button("Add").clicked() {}
                                    });
                                    */
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
}

#[derive(Resource, Default)]
pub struct PersistentInfoWindows {
    pub windows: HashMap<String, serde_json::Value>,
}

pub fn item_info(
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut persistent_info_windows: ResMut<PersistentInfoWindows>,
    res_manager: Res<TileMapResources>,
    state: Res<EguiBlockInputState>,
    tools: Res<ToolResources>,
    mut workspace: ResMut<Workspace>,
    mut zoom_change: EventWriter<ZoomChangedEvent>,
) {
    if mouse_button.just_pressed(MouseButton::Left) && !state.block_input && tools.pointer {
        let (camera, camera_transform) = camera.iter().next().expect("Couldnt get camera");
        let Ok(window) = windows.single() else {
            return;
        };
        if let Some(cursor_pos) = window.cursor_position() {
            let world_position = camera
                .viewport_to_world_2d(camera_transform, cursor_pos)
                .unwrap();
            let position = game_to_coord(
                world_position.x,
                world_position.y,
                res_manager.chunk_manager.refrence_long_lat,
                res_manager.chunk_manager.displacement,
                14,
                res_manager.zoom_manager.tile_quality,
            );
            let envelope: AABB<[f64; 2]> = AABB::from_corners(
                [position.lat as f64, position.long as f64],
                [position.lat as f64, position.long as f64],
            );

            let mut features: Vec<MapFeature> = Vec::new();
            for i in workspace.get_rendered_requests() {
                if i.get_processed_data().size() == 0 {
                    continue;
                }
                features.extend(
                    i.get_processed_data()
                        .locate_in_envelope_intersecting(&envelope)
                        .cloned()
                        .collect::<Vec<_>>(),
                );
            }

            for feature in features {
                let feat = feature.clone();
                if persistent_info_windows
                    .windows
                    .contains_key(&feat.id.to_string())
                {
                    continue;
                }
                persistent_info_windows
                    .windows
                    .insert(feat.id.to_string(), feat.properties);
            }
        }
    }

    let mut windows_to_remove = Vec::new();
    for (id, window_state) in persistent_info_windows.windows.iter() {
        egui::Window::new(id.clone()).show(contexts.ctx_mut(), |ui| {
            egui::Grid::new("grid").show(ui, |ui| {
                if let Some(object) = window_state.as_object() {
                    for (key, value) in object {
                        ui.horizontal(|ui| {
                            ui.label(key);

                            ui.label(format!("{}", value));

                            if let Some(color) = workspace
                                .workspace
                                .as_mut()
                                .unwrap()
                                .properties
                                .get(&(key.clone(), value.clone()))
                            {
                                let mut color = egui::Color32::from_rgb(
                                    (color.red * 255.0) as u8,
                                    (color.green * 255.0) as u8,
                                    (color.blue * 255.0) as u8,
                                );
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    // Update the workspace properties with the new color
                                    workspace.workspace.as_mut().unwrap().properties.insert(
                                        (key.clone(), value.clone()),
                                        Srgba {
                                            red: color.r() as f32 / 255.0,
                                            green: color.g() as f32 / 255.0,
                                            blue: color.b() as f32 / 255.0,
                                            alpha: color.a() as f32 / 255.0,
                                        },
                                    );
                                    zoom_change.write(ZoomChangedEvent);
                                }
                            } else {
                                let mut color = egui::Color32::WHITE; // Default color
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    // Update the workspace properties with the new color
                                    workspace.workspace.as_mut().unwrap().properties.insert(
                                        (key.clone(), value.clone()),
                                        Srgba {
                                            red: color.r() as f32 / 255.0,
                                            green: color.g() as f32 / 255.0,
                                            blue: color.b() as f32 / 255.0,
                                            alpha: color.a() as f32 / 255.0,
                                        },
                                    );
                                    zoom_change.write(ZoomChangedEvent);
                                }
                            }
                        });
                        ui.end_row();
                    }
                }
            });

            if ui.button("Close").clicked() {
                windows_to_remove.push(id.clone());
            }
        });
    }
    for id in windows_to_remove {
        persistent_info_windows.windows.remove(&id);
    }
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

// We should add a right panel for area analysis
/*
Right Panel: Area Analysis
1. Customizable Styles

    Color by Attribute:
        A dropdown for selecting the attribute to color by
        (e.g., "Building type," "Land use," "Population density").
        This dynamically updates the map with chosen color schemes.

    Size by Attribute:
        Allow users to adjust point size based on specific attributes,
        such as the "size of buildings" or "household income."

    Icons for Points:
        Enable users to select different icons for points on the map based on attributes
        (e.g., a house icon for residences, a tree for parks).

2. Thematic Mapping: Choropleths

    Dropdown for Layer Selection:
        A list to choose which layers to apply choropleth styling to.

    Legend for Choropleth:
        Automatically generate a color scale showing the range of values
        (e.g., darker shades of blue for higher population density).

3. Legend Generation

    Auto-generated Legends:
        Once you style a layer or apply thematic mapping,
        the panel will automatically generate a legend explaining the color, size, and icons used on the map.

4. Charts & Statistics

    House Type Breakdown:
        Show a pie chart or bar graph breaking down the area by different house types
        (e.g., "Detached," "Semi-detached," "Apartments").

    Area Size:
        Display the total size (area) of a selected polygon or area (e.g., "Total area: 150,000 m²").

    Population Density:
        Show the population density for the selected area (e.g., people per km²).

    Weather Analysis:
        Include a weather summary for the selected area,
        such as temperature, precipitation, or other relevant weather parameters.

    UV & Sunlight Analysis:
        Provide insights about the UV index or sunlight exposure in the area,
        possibly overlaying heat maps to indicate high/low sunlight regions.

    Attribute Analysis:
        A detailed breakdown of any attributes present in the area
        (e.g., "Number of residential buildings," "Average household income").

5. Layer Pickers

    Add Layers:
        Allow users to add additional data layers, such as road networks, land use, pollution levels, etc.

    Layer Toggle:
        Let users enable/disable specific layers in the analysis.

UI Layout Suggestions:

    Top Section:
        A title or summary of the selected area (e.g., "Cambridge - Central Area").

    Left Section:
        A panel with sliders/dropdowns for custom styles, color by attribute, and thematic mapping options.

    Middle Section:
        Interactive charts (pie charts, bar graphs, heatmaps) showing key statistics,
        such as population density and area size.

    Bottom Section:
        A mini map preview (if space allows), or layer picker dropdown with checkboxes to enable/disable data layers.
*/
// Have a little option to hide to the side. Also allow resizing.
pub fn workspace_analysis_ui(mut contexts: EguiContexts, workspace_res: ResMut<Workspace>) {
    if let Some(workspace) = &workspace_res.workspace {
        let ctx = contexts.ctx_mut();
        let screen_rect = ctx.screen_rect();

        let tilebox_width = 200.0;
        let tilebox_height = screen_rect.height() - 40.0;

        let tilebox_pos = egui::pos2(screen_rect.width() - tilebox_width - 10.0, 30.0);

        // Number of items:
        let mut item_count = 0;
        for lists in workspace_res.get_rendered_requests().iter() {
            item_count += lists.get_processed_data().size();
        }
        let workspace_area = workspace.get_area();
        egui::Area::new("info".into())
            .fixed_pos(tilebox_pos)
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
                        ui.set_width(tilebox_width);
                        ui.set_height(tilebox_height);
                        ui.vertical_centered(|ui| {
                            ui.label("");
                            ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
                            ui.label(RichText::new("Workspace Analysis").strong());
                            ui.separator();
                            ui.label(format!("Number of items: {}", item_count));
                            ui.label(format!(
                                "Workspace area: {} {:#?}",
                                workspace_area.0, workspace_area.1
                            ));
                        });
                    });
            });
    }
}
