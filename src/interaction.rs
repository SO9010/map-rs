//! # Interaction System Module
//! 
//! This module handles user input and interaction with the map viewer.
//! 
//! ## Purpose
//! - Processes user input events (mouse, keyboard, file drops)
//! - Handles file drag-and-drop functionality for data import
//! - Manages interaction states and user interface events
//! - Coordinates between UI and map interactions
//! 
//! ## Key Components
//! - `InteractionSystemPlugin`: Main plugin for user interactions
//! - File drop handling system
//! - Input event processing
//! - Interaction state management
//! 
//! ## Features
//! - Drag-and-drop file import (GeoJSON, etc.)
//! - Mouse and keyboard input handling
//! - Touch and gesture support preparation
//! - Context-sensitive interaction modes

use bevy::prelude::*;
use bevy_map_viewer::ZoomChangedEvent;

use crate::workspace::Workspace;

pub struct InteractionSystemPlugin;

impl Plugin for InteractionSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, file_drop);
    }
}

fn file_drop(
    mut evr_dnd: EventReader<FileDragAndDrop>,
    workspace_res: ResMut<Workspace>,
    zoom_event: EventWriter<ZoomChangedEvent>,
) {
    for ev in evr_dnd.read() {
        if let FileDragAndDrop::HoveredFile { window, path_buf } = ev {
            if path_buf.extension().unwrap() == "geojson" {
                // Make this so the UI respods to a hover for example we can chanage the ui to have a gray overlay and say "Drop file here" and if it will be accepted
                println!(
                    "Hovered file with path: {:?}, in window id: {:?}",
                    path_buf, window
                );
            }
        }
        if let FileDragAndDrop::DroppedFile { window, path_buf } = ev {
            if path_buf.extension().unwrap() == "geojson" {
                println!(
                    "Dropped file with path: {:?}, in window id: {:?}",
                    path_buf, window
                );
                // TODO ADD ADD WORKSPACE REQUEST
                /*
                get_file_data(&mut map_bundle.features, path_buf.to_str().unwrap());
                zoom_event.write(ZoomChangedEvent);
                */
            }
        }
    }
}
