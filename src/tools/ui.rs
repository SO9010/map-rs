use bevy::{prelude::*, render::camera};
use bevy_egui::{egui::{self, Color32, RichText}, EguiContexts, EguiPreUpdateSet};


use crate::{tiles::{ChunkManager, Location, ZoomManager}, types::Coord};

use super::{Measure, Pins, SelectionAreas, SelectionSettings, SelectionType};

pub struct ToolbarUiPlugin;

impl Plugin for ToolbarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (tool_ui.after(EguiPreUpdateSet::InitContexts), tool_actions_ui.after(EguiPreUpdateSet::InitContexts)));
    }
}


fn tool_ui(
    mut selection_settings: ResMut<SelectionSettings>,
    mut measure: ResMut<Measure>,
    mut pins: ResMut<Pins>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();

    let toolbar_width = 225.0;
    let toolbar_height = 40.0;
    
    let screen_rect = ctx.screen_rect();
    let toolbar_pos = egui::pos2(
        (screen_rect.width() - toolbar_width) / 2.0, 
        screen_rect.height() - toolbar_height - 10.0
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

                        if ui.add_sized(
                            [64.0, 30.0], 
                            button_selected(selection_settings.selection_enabled, "Select")
                        ).clicked() {
                            if selection_settings.selection_enabled {
                                selection_settings.selection_tool_type.iterate();
                            }
                            selection_settings.selection_enabled = true;
                            measure.disable();
                            pins.enabled = false;
                        }
                        if ui.add_sized(
                            [64.0, 30.0], 
                            button_selected(measure.enabled, "Measure")
                        ).clicked() {
                            measure.enabled = true;
                            selection_settings.selection_enabled = false;
                            pins.enabled = false;
                        }
                        if ui.add_sized(
                            [64.0, 30.0], 
                            button_selected(pins.enabled, "Pin")
                        ).clicked() {
                            pins.enabled = true;
                            selection_settings.selection_enabled = false;
                            measure.disable();
                        }
                    });
                });
        });
}

fn tool_actions_ui(
    mut contexts: EguiContexts,
    mut selections: ResMut<SelectionAreas>,
    zoom_manager: Res<ZoomManager>,
    chunk_manager: Res<ChunkManager>,
    mut camera: Query<(&Camera, &mut Transform), With<Camera2d>>,
    mut location_manager: ResMut<Location>,
) {
    let ctx = contexts.ctx_mut();

    let tilebox_width = 200.0;
    let tilebox_height = 100.0;
    
    let screen_rect = ctx.screen_rect();
    let tilebox_pos = egui::pos2(
        screen_rect.width() - 210., 
        10.0
    );
    
    egui::Area::new("tool_action".into())
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
                    egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered( |ui| {
                        let available_width = tilebox_width - 10.0; // 10px padding on each side
                        ui.set_max_width(available_width);
    
                        let selections_clone: Vec<_> = selections.areas.iter().cloned().collect();

                        for selection in selections_clone {
                            ui.vertical_centered(|ui| {
                                ui.set_max_width(tilebox_width - 10.);
                                if ui.checkbox(&mut false, RichText::new(selection.selection_name.clone())).clicked() {
                                    location_manager.location = selection.start.unwrap();
                                    let mut camera_transform = camera.single_mut().1;
                                    match selection.selection_type {
                                        SelectionType::RECTANGLE => {
                                            let starting = selection.start.unwrap().to_game_coords(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into());
                                            let ending = selection.end.unwrap().to_game_coords(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into());
                                            camera_transform.translation = Vec3::new(starting.x - ((starting.x - ending.x) / 2.0), starting.y - ((starting.y - ending.y) / 2.0), 1.0);
                                        }
                                        SelectionType::POLYGON => {
                                            let starting = selection.start.unwrap().to_game_coords(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into());
                                            camera_transform.translation = Vec3::new(starting.x, starting.y, 1.0);  
                                        },
                                        SelectionType::CIRCLE => {
                                            let starting = selection.start.unwrap().to_game_coords(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into());
                                            camera_transform.translation = Vec3::new(starting.x, starting.y, 1.0);                                          
                                        },
                                        _ => {}
                                    }
                              
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