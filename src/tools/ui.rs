use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPreUpdateSet,
    egui::{self, Color32, RichText},
};

use super::ToolResources;

/// A plugin that provides UI components for interacting with tools and workspaces.
/// This includes a toolbar for tool selection and a workspace management panel.
pub struct ToolbarUiPlugin;

impl Plugin for ToolbarUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (tool_ui.after(EguiPreUpdateSet::InitContexts),));
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
