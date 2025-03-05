use bevy::prelude::*;
use bevy_egui::{egui::{self, Color32, RichText}, EguiContexts, EguiPreUpdateSet};


use super::{Measure, Pins, SelectionAreas, SelectionSettings};

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

// Soon make overpass query. Also to view selected points and waypoints. This will be big when this is done.
// Try and add more apis like weather. We want to be able to add layers.

fn tool_actions_ui(
    mut contexts: EguiContexts,
    mut selections: ResMut<SelectionAreas>
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
    
                        for selection in selections.areas.iter_mut() {
                                ui.horizontal_wrapped(|ui| {
                                    ui.set_max_width(tilebox_width - 10.);
                                    if ui.checkbox(&mut false, RichText::new(&selection.selection_name)).clicked() {
                                    }
                                });
                        }
                    });
                });
            });
        });

}