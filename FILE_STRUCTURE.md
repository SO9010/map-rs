# Map-RS File Structure Documentation

## Project Root Structure

```
map-rs/
├── Cargo.toml                 # Project configuration and dependencies
├── Cargo.lock                 # Dependency lock file
├── LICENSE                    # Project license (MIT/Apache)
├── README.md                  # Project overview and setup instructions
├── API_DOCUMENTATION.md       # High-level API documentation
├── DETAILED_API_DOCS.md       # Comprehensive API reference
├── FILE_STRUCTURE.md          # This documentation file
├── assets/                    # Static assets and resources
├── src/                       # Source code directory
└── target/                    # Compiled output (build artifacts)
```

## Assets Directory (`assets/`)

Static resources used by the application at runtime.

```
assets/
├── buttons/                   # UI button icons and graphics
│   ├── arrow.svg             # Navigation arrow icon
│   ├── circle-o.svg          # Circle selection tool icon
│   ├── measure.svg           # Measurement tool icon  
│   ├── north-arrow-n.svg     # North direction indicator
│   ├── pin.svg               # Map pin/marker icon
│   ├── polygon-pt.svg        # Polygon tool icon
│   └── rectangle-pt.svg      # Rectangle selection tool icon
├── fonts/                     # Typography assets
│   └── BagnardSans.otf       # Custom font for UI elements
├── icon/                      # Application icons
│   └── icon1080.png          # High-resolution app icon
├── shaders/                   # Custom WGSL shaders
│   ├── full_screen_pass.wgsl # Post-processing shader
│   └── old_monitor.wgsl      # Retro display effect shader
└── test/                      # Test data files
    └── green-belt.geojson    # Sample GeoJSON for testing
```

## Source Code Directory (`src/`)

Main application source code organized by functionality.

```
src/
├── main.rs                    # Application entry point
├── camera.rs                  # Camera system and controls
├── debug.rs                   # Debug utilities and diagnostics
├── interaction.rs             # User input and file handling
├── settings.rs                # Application configuration
├── apis/                      # External API integrations
├── geojson/                   # Geographic data processing
├── llm/                       # AI/LLM integration
├── overpass/                  # OpenStreetMap API client
├── storage/                   # Data persistence (future)
├── tools/                     # Interactive map tools
└── workspace/                 # Workspace management
```

### APIs Directory (`src/apis/`)

Integration modules for external data services.

```
apis/
├── NOTE                       # Development notes and API keys
├── environmental/             # Environmental data providers
│   ├── europa.rs             # European environmental data API
│   └── landcover.rs          # Land cover classification API
└── weather/                   # Weather data providers
    ├── earthdata.rs          # NASA Earth data API
    ├── nws.rs                # National Weather Service API
    └── openmeteo.rs          # Open-Meteo weather API
```

### GeoJSON Module (`src/geojson/`)

Geographic data processing and rendering components.

```
geojson/
├── mod.rs                     # Module exports and main functionality
├── loader.rs                  # GeoJSON file loading and parsing
├── renderer.rs                # 2D rendering using tessellation
├── shapes_plugin.rs           # Interactive shape creation tools
└── types.rs                   # Geographic feature data structures
```

**Key Components:**
- **loader.rs**: Handles GeoJSON file import, validation, and parsing
- **renderer.rs**: Converts geographic features to renderable meshes using Lyon tessellation
- **shapes_plugin.rs**: Provides tools for creating and editing geometric shapes
- **types.rs**: Defines `MapFeature`, `GeometryType`, and related data structures

### LLM Module (`src/llm/`)

AI and Large Language Model integration for data analysis.

```
llm/
├── mod.rs                     # Module exports and client setup
├── client.rs                  # OpenRouter API client implementation
└── openrouter_types.rs        # API request/response data structures
```

**Functionality:**
- Chat-based interaction with geographic data
- Automated analysis and insight generation
- Natural language querying of spatial information
- Integration with multiple LLM providers through OpenRouter

### Overpass Module (`src/overpass/`)

OpenStreetMap data integration via Overpass API.

```
overpass/
├── mod.rs                     # Module exports and client setup
├── client.rs                  # Overpass API client and query builder
└── overpass_types.rs          # OSM data structures and query types
```

**Features:**
- Complex spatial queries using Overpass QL
- Efficient OSM data parsing and conversion
- Bounding box and feature-based filtering
- Rate limiting and respectful API usage

### Storage Module (`src/storage/`)

Data persistence and caching system (in development).

```
storage/
└── WORK ON ME NEXT WITH URGRANCY  # Development priority marker
```

**Planned Features:**
- Workspace persistence to disk
- Cached API response storage
- User preference management
- Data export/import functionality

### Tools Module (`src/tools/`)

Interactive tools for map manipulation and analysis.

```
tools/
├── mod.rs                     # Tool management and exports
├── tool.rs                    # Base tool trait and common functionality
├── ui.rs                      # Tool UI components and panels
├── feature_picker.rs          # Feature selection and information display
├── measure.rs                 # Distance and area measurement tools
├── pin.rs                     # Map marker placement and management
└── work_space_selector.rs     # Area selection for workspace definition
```

**Tool Categories:**
- **Measurement**: Distance, area, and perimeter calculation
- **Selection**: Rectangle, polygon, and circle selection tools
- **Annotation**: Pin placement and labeling
- **Analysis**: Feature information and attribute display

### Workspace Module (`src/workspace/`)

Core workspace management and data organization.

```
workspace/
├── mod.rs                     # Core workspace functionality and resources
├── commands.rs                # Workspace operation commands
├── renderer.rs                # Workspace-specific rendering logic
├── ui.rs                      # User interface components (analysis panel, chat)
├── worker.rs                  # Background task processing
└── workspace_types.rs         # Data structures and plugin implementation
```

**Key Responsibilities:**
- Multi-layered data organization
- Background processing of API requests
- Real-time data updates and synchronization
- User interface for workspace interaction
- Integration between different data sources

## Build Output (`target/`)

Rust compiler output and build artifacts.

```
target/
├── CACHEDIR.TAG              # Build cache metadata
└── debug/                    # Debug build outputs
    ├── map-rs                # Main executable (debug)
    ├── map-rs.d              # Dependency information
    ├── build/                # Intermediate build files
    ├── deps/                 # Compiled dependencies
    ├── examples/             # Example program outputs
    └── incremental/          # Incremental compilation cache
```

## File Naming Conventions

### Rust Source Files
- **Module files**: `mod.rs` - Main module entry point
- **Implementation files**: Descriptive names matching functionality
- **Plugin files**: `*_plugin.rs` - Bevy plugin implementations
- **Type files**: `*_types.rs` - Data structure definitions
- **Client files**: `*_client.rs` - External service clients

### Asset Files
- **Icons**: SVG format for scalability
- **Fonts**: OpenType format (.otf) for typography
- **Shaders**: WGSL format for WebGPU compatibility  
- **Test data**: Standard format extensions (.geojson, .json)

### Documentation Files
- **README.md**: Project overview and setup
- **API_DOCUMENTATION.md**: High-level API guide
- **DETAILED_API_DOCS.md**: Comprehensive API reference
- **FILE_STRUCTURE.md**: This file structure guide

## Development Workflow

### Adding New Features
1. Create appropriate module structure in `src/`
2. Implement core functionality with proper documentation
3. Add corresponding UI components if needed
4. Update plugin registration in relevant `mod.rs` files
5. Add assets to `assets/` directory if required
6. Update documentation files

### Asset Management
- Place UI icons in `assets/buttons/`
- Add custom fonts to `assets/fonts/`
- Store shader files in `assets/shaders/`
- Keep test data in `assets/test/`

### Module Organization
- Each major feature gets its own module directory
- Use `mod.rs` for module exports and main functionality
- Separate concerns: types, client code, UI, and logic
- Maintain consistent naming conventions

This file structure supports modular development, clear separation of concerns, and easy navigation of the codebase. Each module has a specific purpose and well-defined interfaces with other parts of the system.
