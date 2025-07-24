//! # Interactive Tools Module
//! 
//! This module provides interactive tools for map manipulation, measurement,
//! and data analysis within the map viewer interface.
//! 
//! ## Purpose
//! - Provide interactive tools for map interaction and analysis
//! - Enable measurement of distances, areas, and spatial relationships
//! - Support creation and placement of map markers and annotations
//! - Facilitate workspace area selection and boundary definition
//! 
//! ## Sub-modules
//! - `measure`: Distance and area measurement tools
//! - `pin`: Map marker and annotation placement tools
//! - `tool`: Base tool trait and common tool functionality
//! - `ui`: User interface components for tool interaction
//! - `work_space_selector`: Area selection tools for workspace definition
//! 
//! ## Key Features
//! - Distance measurement between points
//! - Area calculation for polygonal regions
//! - Interactive marker placement and editing
//! - Workspace boundary selection and management
//! - Tool state management and mode switching
//! 
//! ## Tool Types
//! - Measurement tools (distance, area, perimeter)
//! - Annotation tools (pins, labels, notes)
//! - Selection tools (rectangle, polygon, circle)
//! - Drawing tools (freehand, shapes)

mod measure;
mod pin;
mod tool;
mod ui;
mod work_space_selector;

pub use measure::*;
pub use pin::*;
pub use tool::*;
pub use ui::*;
pub use work_space_selector::*;
