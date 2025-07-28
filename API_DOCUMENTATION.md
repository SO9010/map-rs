# Map-RS API Documentation

## Project Overview

Map-RS is a Rust-based interactive map viewer built with the Bevy game engine. It provides advanced geospatial data visualization, real-time data analysis, and AI-powered insights for geographic information systems (GIS) applications.

## Key Features

- **Interactive Map Visualization**: Real-time map rendering with multiple tile providers
- **Geospatial Data Processing**: Support for GeoJSON, Overpass API integration
- **AI-Powered Analysis**: LLM integration for map data insights
- **Advanced Tools**: Measurement, shape drawing, area selection
- **Workspace Management**: Save and load different map configurations
- **Multi-API Support**: Weather data, environmental data, and spatial queries

## Architecture

The application follows a modular plugin architecture using Bevy's ECS (Entity Component System):

```
map-rs/
├── src/
│   ├── main.rs                 # Application entry point and plugin registration
│   ├── camera.rs              # Camera controls and movement systems
│   ├── debug.rs               # Debug utilities and performance monitoring
│   ├── interaction.rs         # User input handling and map interactions
│   ├── settings.rs            # Application configuration and preferences
│   ├── apis/                  # External API integrations
│   │   ├── weather/           # Weather data providers (OpenMeteo, NWS, EarthData)
│   │   └── environmental/     # Environmental data (Europa, LandCover)
│   ├── geojson/               # GeoJSON processing and rendering
│   │   ├── mod.rs            # Module exports and main functionality
│   │   ├── loader.rs         # GeoJSON file loading and parsing
│   │   ├── renderer.rs       # 2D rendering of geographic features
│   │   ├── shapes_plugin.rs  # Shape creation and manipulation
│   │   └── types.rs          # Data structures for geographic features
│   ├── llm/                   # AI/LLM integration
│   │   ├── mod.rs            # Module exports
│   │   ├── client.rs         # OpenRouter API client
│   │   └── openrouter_types.rs # API request/response types
│   ├── overpass/              # OpenStreetMap Overpass API
│   │   ├── mod.rs            # Module exports
│   │   ├── client.rs         # Overpass query client
│   │   └── overpass_types.rs # OSM data structures
│   ├── storage/               # Data persistence and caching
│   ├── tools/                 # Interactive map tools
│   │   ├── mod.rs            # Tool management system
│   │   ├── tool.rs           # Base tool trait and implementations
│   │   ├── ui.rs             # Tool UI components
│   │   ├── measure.rs        # Distance and area measurement
│   │   ├── pin.rs            # Map marker placement
│   │   ├── feature_picker.rs # Feature selection and info display
│   │   └── work_space_selector.rs # Workspace area selection
│   └── workspace/             # Workspace and data management
│       ├── mod.rs            # Core workspace functionality
│       ├── ui.rs             # Workspace UI components
│       ├── renderer.rs       # Workspace-specific rendering
│       ├── worker.rs         # Background task processing
│       ├── commands.rs       # Workspace operations
│       └── workspace_types.rs # Data structures and plugin setup
├── assets/                    # Static assets
│   ├── buttons/              # UI button icons
│   ├── fonts/                # Typography assets
│   ├── icon/                 # Application icons
│   ├── shaders/              # Custom WGSL shaders
│   └── test/                 # Test data files
└── target/                   # Compiled artifacts
```

## Core Systems

### 1. Main Application (`src/main.rs`)
The entry point that initializes all plugins and systems.

### 2. Camera System (`src/camera.rs`)
Handles map navigation, zoom controls, and viewport management.

### 3. Workspace System (`src/workspace/`)
Manages different map configurations, data layers, and user sessions.

### 4. GeoJSON Processing (`src/geojson/`)
Handles loading, parsing, and rendering of geographic data.

### 5. Tools System (`src/tools/`)
Interactive tools for map manipulation and measurement.

### 6. API Integrations (`src/apis/`)
External data providers for weather, environmental, and geographic data.

## Plugin Architecture

Each major system is implemented as a Bevy plugin:

- `WorkspacePlugin`: Core workspace management
- `CameraSystemPlugin`: Camera controls
- `InteractionSystemPlugin`: User input handling
- `RenderPlugin`: GeoJSON rendering
- `SettingsPlugin`: Configuration management
- `ToolsPlugin`: Interactive tools
- `DebugPlugin`: Development utilities

## Data Flow

1. **Input**: User interactions, API responses, file imports
2. **Processing**: Data parsing, spatial calculations, filtering
3. **Storage**: Workspace persistence, caching, session management
4. **Rendering**: 2D map rendering, UI updates, visual effects
5. **Output**: Interactive map display, analysis results, exports

## External Dependencies

- **Bevy**: Game engine and ECS framework
- **egui**: Immediate mode GUI
- **rstar**: Spatial indexing (R-tree)
- **lyon**: 2D path tessellation
- **ureq**: HTTP client for API requests
- **serde**: Serialization framework
- **geojson**: GeoJSON parsing
- **chrono**: Date and time handling

## Development Setup

1. Install Rust toolchain
2. Clone repository
3. Run `cargo build` to compile
4. Run `cargo run` to start application
5. **For AI features**: Edit `src/workspace/ui.rs` and replace `"!!! YOUR TOKEN HERE !!!"` with your OpenRouter API key from [openrouter.ai](https://openrouter.ai/)

The application will function without an API key, but AI-powered chat and analysis features will be unavailable.

This documentation provides a high-level overview of the system architecture. Detailed API documentation for each module follows in the subsequent sections.
