//! # GeoJSON Processing Module
//! 
//! This module handles all GeoJSON-related functionality including loading, parsing,
//! rendering, and manipulation of geographic data.
//! 
//! ## Purpose
//! - Load and parse GeoJSON files and data streams
//! - Render geographic features as 2D shapes on the map
//! - Provide tools for creating and editing geographic shapes
//! - Manage spatial indexing and querying of geographic data
//! 
//! ## Sub-modules
//! - `loader`: GeoJSON file loading and parsing utilities
//! - `renderer`: 2D rendering of geographic features using tessellation
//! - `shapes_plugin`: Interactive shape creation and editing tools
//! - `types`: Data structures and types for geographic features
//! 
//! ## Key Features
//! - Support for all GeoJSON geometry types (Point, LineString, Polygon, etc.)
//! - Efficient spatial indexing using R-trees
//! - Real-time tessellation for complex polygons
//! - Style-based rendering with customizable colors and stroke properties
//! - Integration with OpenStreetMap data via Overpass API

mod loader;
mod renderer;
mod shapes_plugin;
mod types;

pub use loader::*;
pub use renderer::*;
pub use shapes_plugin::*;
pub use types::*;
