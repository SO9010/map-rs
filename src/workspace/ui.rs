use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, Align2, Checkbox, CornerRadius, RichText},
};
use bevy_map_viewer::{
    Coord, EguiBlockInputState, MapViewerMarker, TileMapResources, ZoomChangedEvent, game_to_coord,
};
use rstar::{AABB, Envelope};
use uuid::Uuid;

use crate::{
    geojson::MapFeature,
    overpass::{build_overpass_query_string, get_bounds},
    tools::ToolResources,
    workspace::{SelectionType, Workspace, WorkspaceData},
};

use super::{RequestType, WorkspaceRequest};

// This should go into workspace so it can be saved.
#[derive(Resource)]
pub struct ChatState {
    pub inner: Arc<Mutex<ChatStateInner>>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ChatStateInner::default())),
        }
    }
}

impl Clone for ChatState {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[derive(Default, Clone)]
pub struct ChatStateInner {
    pub input_text: String,
    pub chat_history: Vec<ChatMessage>,
    pub is_processing: bool,
}

#[derive(Clone)]
pub struct ChatMessage {
    pub content: String,
    pub is_user: bool,
}

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

                            ui.label(format!("{value}"));

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

pub fn chat_box_ui(
    mut contexts: EguiContexts,
    chat_state: Res<ChatState>,
    mut workspace: ResMut<Workspace>,
) {
    if workspace.workspace.is_none() {
        return;
    }
    let ctx = contexts.ctx_mut();
    let screen_rect = ctx.screen_rect();

    let chat_width = if screen_rect.width() / 4_f32 > 200.0 {
        screen_rect.width() / 4_f32
    } else {
        200.0
    }; // Same width as workspace_analysis_ui
    let chat_height = screen_rect.height() - 50.0;
    let chat_pos = egui::pos2(
        screen_rect.width() - chat_width - 10.0,
        40.0, // Position under item_info with some spacing
    );

    egui::Area::new("chat_box".into())
        .fixed_pos(chat_pos)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(25, 25, 25, 255))
                .corner_radius(10.0)
                .shadow(egui::epaint::Shadow {
                    color: egui::Color32::from_black_alpha(60),
                    offset: [5, 5],
                    blur: 10,
                    spread: 5,
                })
                .show(ui, |ui| {
                    ui.set_width(chat_width);
                    ui.set_height(chat_height);

                    ui.vertical(|ui| {
                        // Header
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("AI Chat Assistant")
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("🗑").on_hover_text("Clear chat").clicked()
                                    {
                                        if let Ok(mut inner) = chat_state.inner.lock() {
                                            inner.chat_history.clear();
                                        }
                                    }
                                },
                            );
                        });

                        ui.separator();

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .stick_to_bottom(true)
                            .max_height(chat_height - 80.0) // Leave space for input
                            .show(ui, |ui| {
                                ui.set_width(chat_width - 20.0);

                                if let Ok(inner) = chat_state.inner.lock() {
                                    if inner.chat_history.is_empty() {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(20.0);
                                            ui.label(
                                                RichText::new("Ask me about the map data!")
                                                    .color(egui::Color32::GRAY)
                                                    .italics(),
                                            );
                                        });
                                    } else {
                                        for message in &inner.chat_history {
                                            render_chat_message(ui, message, chat_width - 30.0);
                                            ui.add_space(8.0);
                                        }
                                    } // Show loading indicator if processing
                                    if inner.is_processing {
                                        ui.horizontal(|ui| {
                                            ui.add_space(10.0);
                                            ui.spinner();
                                            ui.label(
                                                RichText::new("AI is thinking...")
                                                    .color(egui::Color32::GRAY),
                                            );
                                        });
                                    }
                                }
                            });

                        ui.add_space(5.0);
                        ui.separator();

                        // Input area
                        ui.horizontal(|ui| {
                            if let Ok(mut inner) = chat_state.inner.lock() {
                                let text_edit = egui::TextEdit::singleline(&mut inner.input_text)
                                    .desired_width(chat_width - 60.0)
                                    .hint_text("Ask about the map data...");

                                let response = ui.add(text_edit);

                                let send_clicked =
                                    ui.button("📤").on_hover_text("Send message").clicked();

                                let enter_pressed = response.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter));

                                if (send_clicked || enter_pressed)
                                    && !inner.input_text.trim().is_empty()
                                    && !inner.is_processing
                                {
                                    let user_message = inner.input_text.trim().to_string();
                                    inner.input_text.clear();
                                    inner.is_processing = true;

                                    // Add user message to history
                                    inner.chat_history.push(ChatMessage {
                                        content: user_message.clone(),
                                        is_user: true,
                                    });

                                    // Drop the lock before calling send_chat_message_background
                                    drop(inner);
                                    send_chat_message_background(
                                        &chat_state,
                                        &mut workspace,
                                        user_message,
                                    );
                                }
                            }
                        });
                    });
                });
        });
}

fn render_chat_message(ui: &mut egui::Ui, message: &ChatMessage, max_width: f32) {
    let bg_color = if message.is_user {
        egui::Color32::from_rgba_premultiplied(70, 130, 180, 200) // Steel blue for user
    } else {
        egui::Color32::from_rgba_premultiplied(60, 60, 60, 200) // Dark gray for AI
    };

    let text_color = egui::Color32::WHITE;

    ui.horizontal(|ui| {
        ui.add_space(10.0);
        if !message.is_user {
            ui.add_space(20.0);
        }

        egui::Frame::new()
            .fill(bg_color)
            .corner_radius(8.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_max_width(max_width * 0.8);

                // Message header
                ui.horizontal(|ui| {
                    let icon = if message.is_user { "👤" } else { "🤖" };
                    let sender = if message.is_user { "You" } else { "AI" };
                    ui.label(
                        RichText::new(format!("{icon} {sender}"))
                            .small()
                            .color(egui::Color32::LIGHT_GRAY),
                    );
                });

                ui.add(egui::Label::new(RichText::new(&message.content).color(text_color)).wrap());
            });

        if message.is_user {
            ui.add_space(10.0);
        }
    });
}

fn send_chat_message_background(
    chat_state: &ChatState,
    workspace: &mut Workspace,
    user_message: String,
) {
    workspace.llm_agent.set_token("!!! YOUR TOKEN HERE !!!");

    // Build context information about the current workspace and selection
    let mut context_info = String::new();

    if let Some(workspace_data) = &workspace.workspace {
        let area = workspace_data.get_area();
        let selection = workspace_data.get_selection();

        context_info.push_str(&format!(
            "Current workspace: {}\n",
            workspace_data.get_name()
        ));
        context_info.push_str(&format!("Selection area: {:.2} {:#?}\n", area.0, area.1));
        context_info.push_str(&format!(
            "Selection type: {:#?}\n",
            selection.selection_type
        ));

        // Add coordinate information based on selection type
        match selection.selection_type {
            crate::workspace::SelectionType::RECTANGLE => {
                if let (Some(start), Some(end)) = (selection.start, selection.end) {
                    context_info.push_str(&format!(
                        "Bounding box: SW({:.6}, {:.6}) to NE({:.6}, {:.6})\n",
                        start.lat, start.long, end.lat, end.long
                    ));
                }
            }
            crate::workspace::SelectionType::CIRCLE => {
                if let (Some(center), Some(edge)) = (selection.start, selection.end) {
                    let radius = center.distance(&edge);
                    context_info.push_str(&format!(
                        "Center: ({:.6}, {:.6}), Radius: {:.2} {:#?}\n",
                        center.lat, center.long, radius.0, radius.1
                    ));
                }
            }
            crate::workspace::SelectionType::POLYGON => {
                if let Some(points) = &selection.points {
                    context_info.push_str(&format!("Polygon with {} vertices\n", points.len()));
                    for (i, point) in points.iter().take(5).enumerate() {
                        context_info.push_str(&format!(
                            "  Point {}: ({:.6}, {:.6})\n",
                            i + 1,
                            point.lat,
                            point.long
                        ));
                    }
                    if points.len() > 5 {
                        context_info.push_str("  ...\n");
                    }
                }
            }
            _ => {}
        }

        // Add information about loaded data
        let requests = workspace.get_rendered_requests();
        let total_features: usize = requests.iter().map(|r| r.get_processed_data().size()).sum();
        context_info.push_str(&format!("Total features loaded: {total_features}\n"));
        context_info.push_str(&format!("Data layers: {}\n", requests.len()));
    } else {
        context_info.push_str("No workspace selected\n");
    }

    // Create the enhanced user message with context
    let enhanced_message = if context_info.trim().is_empty() {
        user_message.clone()
    } else {
        format!("CONTEXT:\n{context_info}\nUSER QUERY: {user_message}")
    };
    if let Some(workspace) = workspace.workspace.as_mut() {
        workspace.add_message("user", &enhanced_message);
    }

    // Attempt to get response from LLM

    let request = WorkspaceRequest::new(
        Uuid::new_v4().to_string(),
        1,
        RequestType::OpenRouterRequest(),
        Vec::new(),
    );
    workspace.worker.queue_request(request);

    // Clean up old messages
    if let Ok(mut inner) = chat_state.inner.lock() {
        while inner.chat_history.len() > 50 {
            inner.chat_history.remove(0);
        }
    }
}
