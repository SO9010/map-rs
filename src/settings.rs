use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_egui::{
    EguiContexts, EguiPlugin, EguiPreUpdateSet,
    egui::{self, Color32, RichText, color_picker::color_edit_button_srgba},
};
use bevy_map_viewer::ZoomChangedEvent;

use crate::{settings::egui::color_picker::Alpha::Opaque, workspace::Workspace};

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ui_example_system.after(EguiPreUpdateSet::InitContexts),
        ); // Ensure the system runs after EguiPreUpdateSet::InitContexts
    }
}

#[allow(dead_code)]
fn ui_example_system(
    mut contexts: EguiContexts,
    mut overpass_settings: ResMut<Workspace>,
    mut zoom_event: EventWriter<ZoomChangedEvent>,
) {
    let ctx = contexts.ctx_mut();
    let screen_rect = ctx.screen_rect();

    let tilebox_width = 200.0;
    let tilebox_height = screen_rect.height() - 40.0;

    let tilebox_pos = egui::pos2(10.0, 30.0);

    egui::Area::new("layers".into())
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
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut color;

                        for (category_name, category) in
                            &mut overpass_settings.overpass_agent.settings.categories
                        {
                            if category.disabled {
                                color = Color32::from_rgb(135, 135, 135);
                            } else {
                                color = Color32::from_rgb(221, 221, 221);
                            }
                            ui.collapsing(RichText::new(category_name).color(color), |ui| {
                                ui.horizontal(|ui| {
                                    if ui
                                        .checkbox(
                                            &mut category.all.clone(),
                                            RichText::new("All").color(color),
                                        )
                                        .clicked()
                                    {
                                        if category.all {
                                            category.all = false;
                                        } else {
                                            category.all = true;
                                            category.set_children(true);
                                            zoom_event.write(ZoomChangedEvent);
                                        }
                                        if category.none {
                                            category.none = false;
                                        }
                                    }
                                    if ui
                                        .checkbox(
                                            &mut category.none.clone(),
                                            RichText::new("None").color(color),
                                        )
                                        .clicked()
                                    {
                                        if category.none {
                                            category.none = false;
                                        } else {
                                            category.none = true;
                                            category.set_children(false);
                                            zoom_event.write(ZoomChangedEvent);
                                        }
                                        if category.all {
                                            category.all = false;
                                        }
                                    }
                                });

                                // Individual toggles
                                for (item_name, (state, clr)) in &mut category.items {
                                    ui.horizontal(|ui| {
                                        if ui
                                            .checkbox(state, RichText::new(item_name).color(color))
                                            .clicked()
                                        {
                                            category.all = false;
                                            category.none = false;
                                            zoom_event.write(ZoomChangedEvent);
                                        }
                                        let clrc = &mut bevy_egui::egui::Color32::from_rgb(
                                            clr.0[0], clr.0[1], clr.0[2],
                                        );
                                        if color_edit_button_srgba(ui, clrc, Opaque).changed() {
                                            zoom_event.write(ZoomChangedEvent);
                                        }
                                    });
                                }
                            });
                        }
                    });
                });
        });
}
