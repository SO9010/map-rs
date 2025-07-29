//! # Overpass API Module
//! 
//! This module provides integration with the OpenStreetMap Overpass API for
//! querying and retrieving geographic data from OpenStreetMap.
//! 
//! ## Purpose
//! - Connect to Overpass API endpoints for OSM data queries
//! - Build and execute complex spatial queries
//! - Parse and process OpenStreetMap data formats
//! - Provide efficient caching and data management for OSM data
//! 
//! ## Sub-modules
//! - `client`: Overpass API client with query building and execution
//! - `overpass_types`: Data structures for OSM features and query responses
//! 
//! ## Key Features
//! - Support for complex Overpass QL (Query Language) queries
//! - Spatial filtering and bounding box queries
//! - Feature type filtering (nodes, ways, relations)
//! - Efficient data parsing and conversion to internal formats
//! - Rate limiting and respectful API usage
//! 
//! ## Query Types
//! - Bounding box queries for specific geographic areas
//! - Feature-based queries (buildings, roads, amenities, etc.)
//! - Attribute-based filtering and selection
//! - Spatial relationship queries

mod client;
mod overpass_types;

pub use client::*;
pub use overpass_types::*;
use ureq::Agent;

#[derive(Clone)]
pub struct OverpassClient {
    url: String,
    pub agent: Agent,
    pub bounds: String,
    pub settings: Settings,
}
