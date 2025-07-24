//! # Debug System Module
//! 
//! This module provides debugging utilities and performance monitoring for the application.
//! 
//! ## Purpose
//! - Real-time performance metrics display (FPS, frame time)
//! - Debug overlays and visual debugging tools
//! - Development-time diagnostics and profiling
//! - Error reporting and logging utilities
//! 
//! ## Key Components
//! - `DebugPlugin`: Main debug plugin with conditional compilation
//! - `DebugText`: Component for debug information display
//! - Performance monitoring systems
//! - Debug UI rendering
//! 
//! ## Features
//! - FPS counter and frame time diagnostics
//! - Debug text overlays
//! - Conditional debug builds (only active in debug mode)
//! - Performance profiling integration

use bevy::{
    color::palettes::css::GOLD,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.add_plugins(FrameTimeDiagnosticsPlugin {
                max_history_length: 10,
                smoothing_factor: 10.0,
            })
            .add_systems(Startup, (debug_draw_fps, debug_draw_entity_no))
            .add_systems(Update, (text_update_fps, count_entities));
        }
    }
}

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct EntityText;

pub fn debug_draw_fps(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            // Create a Text with multiple child spans.
            Text::new("FPS: "),
            TextFont {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("fonts/BagnardSans.otf"),
                font_size: 21.0,
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(45.0),
                right: Val::Px(5.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            (
                TextFont {
                    font_size: 18.0,
                    font: asset_server.load("fonts/BagnardSans.otf"),
                    ..default()
                },
                TextColor(GOLD.into()),
            ),
            FpsText,
        ));
}

pub fn text_update_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<FpsText>>,
) {
    for mut span in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                **span = format!("{value:.2}");
            }
        }
    }
}

pub fn debug_draw_entity_no(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            // Create a Text with multiple child spans.
            Text::new("Entities: "),
            TextFont {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("fonts/BagnardSans.otf"),
                font_size: 21.0,
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(60.0),
                right: Val::Px(5.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            (
                TextFont {
                    font_size: 18.0,
                    font: asset_server.load("fonts/BagnardSans.otf"),
                    ..default()
                },
                TextColor(GOLD.into()),
            ),
            EntityText,
        ));
}

pub fn count_entities(
    query_entity: Query<Entity>,
    mut query: Query<&mut TextSpan, With<EntityText>>,
) {
    for mut span in &mut query {
        let entity_count = query_entity.iter().count();
        **span = format!("{}", entity_count);
    }
}
