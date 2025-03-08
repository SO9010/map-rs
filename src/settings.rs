use bevy::prelude::*;
#[allow(unused_imports)]
use bevy_egui::{EguiContexts, EguiPlugin, EguiPreUpdateSet, egui::{self, color_picker::color_edit_button_srgba, Color32, RichText}};
use bevy_prototype_lyon::entity::Path;

use crate::types::{MapBundle, MapFeature, SettingsOverlay};
use crate::settings::egui::color_picker::Alpha::Opaque;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Update, ui_example_system.after(EguiPreUpdateSet::InitContexts)) // Ensure the system runs after EguiPreUpdateSet::InitContexts
            .init_resource::<OccupiedScreenSpace>()
            .insert_resource(SettingsOverlay::new());
    }
}

#[derive(Default, Resource)]
pub struct OccupiedScreenSpace {
    pub left: f32,
}

#[allow(dead_code)]
fn ui_example_system(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut overpass_settings: ResMut<SettingsOverlay>,
    shapes_query: Query<(Entity, &Path, &GlobalTransform, &MapFeature)>,
    mut map_bundle: ResMut<MapBundle>,
    mut commands: Commands,
) {
    let ctx = contexts.ctx_mut();

    occupied_screen_space.left = egui::SidePanel::left("Layers")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Layers");

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut color;
                
                for (category_name, category) in &mut overpass_settings.categories {
                    if category.disabled {
                        color = Color32::from_rgb(135, 135, 135);
                    } else {
                        color = Color32::from_rgb(221, 221, 221);
                    }
                    ui.collapsing(RichText::new(category_name).color(color), |ui| {
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut category.all.clone(), RichText::new("All").color(color)).clicked() {
                                if category.all {
                                    category.all = false;
                                } else {
                                    category.all = true;
                                    category.set_children(true);
                                    map_bundle.respawn = true;
                                    map_bundle.get_more_data = true;
                                }
                                if category.none {
                                    category.none = false;
                                }
                            }
                            if ui.checkbox(&mut category.none.clone(), RichText::new("None").color(color)).clicked() {
                                if category.none {
                                    category.none = false;
                                } else {
                                    category.none = true;
                                    category.set_children(false);
                                    map_bundle.respawn = true;
                                    map_bundle.get_more_data = true;
                                }
                                if category.all {
                                    category.all = false;
                                }
                            }
                        });
        
                        // Individual toggles
                        for (item_name, (state, clr)) in &mut category.items {
                            ui.horizontal(|ui| {
                                if ui.checkbox(state , RichText::new(item_name).color(color)).clicked() {
                                    category.all = false;
                                    category.none = false;
                                    map_bundle.respawn = true;
                                    map_bundle.get_more_data = true;
                                }
                                if color_edit_button_srgba(ui, clr, Opaque).changed() {
                                    // TODO: Find a way to not update as soon as it changes but only when the user is done.
                                    map_bundle.respawn = true;
                                }
                            });
                        }
                    });
                }
                if ui.button("Clear Map").on_hover_text("Despawns the data which makes up this map").clicked() {
                    for (entity, _, _, _) in shapes_query.iter() {
                        commands.entity(entity).despawn_recursive();
                    } 
                }
            });
            
    
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
}