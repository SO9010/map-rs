/// The purpose of this file will be to create a ui for customising the vector tiles and choosing which tiles to display!
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Checkbox, RichText},
    EguiContexts, EguiPreUpdateSet,
};
use bevy_map_viewer::TileMapResources;

pub struct TilesUiPlugin;

impl Plugin for TilesUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tile_ui.after(EguiPreUpdateSet::InitContexts));
    }
}

fn tile_ui(mut res_manager: ResMut<TileMapResources>, mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();

    let tilebox_width = 250.0;
    let tilebox_height = 75.0;

    let screen_rect = ctx.screen_rect();
    let tilebox_pos = egui::pos2(
        (screen_rect.width() - tilebox_width) - 10.0,
        screen_rect.height() - tilebox_height - 53.0,
    );

    egui::Area::new("tilebox".into())
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

                    ui.vertical_centered(|ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.spacing_mut().item_spacing = egui::vec2(8.0, 10.0);
                            ui.set_max_width(tilebox_width);
                            for (url, (enabled, _)) in
                                &mut res_manager.tile_request_client.tile_web_origin.clone()
                            {
                                ui.vertical_centered(|ui| {
                                    ui.set_max_width(tilebox_width);
                                    if ui
                                        .add_sized(
                                            [tilebox_width - 10., 20.],
                                            Checkbox::new(enabled, RichText::new(url.as_str())),
                                        )
                                        .clicked()
                                    {
                                        res_manager
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
}

// Soon make overpass query. Also to view selected points and waypoints. This will be big when this is done.
// Try and add more apis like weather. We want to be able to add layers.
