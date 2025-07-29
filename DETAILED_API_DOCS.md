# Map-RS Detailed API Documentation

## Core Data Structures

### Workspace System

#### `Workspace` Resource
The main resource that manages workspace state and external service connections.

```rust
#[derive(Resource)]
pub struct Workspace {
    pub workspace: Option<WorkspaceData>,
    pub loaded_requests: Arc<Mutex<HashMap<String, WorkspaceRequest>>>,
    pub worker: WorkspaceWorker,
    pub overpass_agent: OverpassClient,
    pub llm_agent: OpenrouterClient,
}
```

**Fields:**
- `workspace`: Current workspace configuration and data
- `loaded_requests`: Thread-safe storage for active data requests  
- `worker`: Background task processor for async operations
- `overpass_agent`: Client for OpenStreetMap Overpass API
- `llm_agent`: Client for AI/LLM integration via OpenRouter

**Methods:**
- `get_rendered_requests()`: Returns currently rendered workspace requests
- `process_request(request: WorkspaceRequest)`: Processes and adds request to workspace

#### `WorkspaceData` Structure
Contains workspace configuration and metadata.

```rust
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceData {
    id: String,
    name: String,
    selection: Selection,
    creation_date: i64,
    last_modified: i64,
    requests: HashSet<String>,
    properties: HashMap<(String, Value), Srgba>,
}
```

**Key Methods:**
- `get_name()`: Returns workspace display name  
- `get_id()`: Returns unique workspace identifier
- `get_selection()`: Returns geographic selection area
- `get_area()`: Calculates total area of workspace region

#### `WorkspaceRequest` Structure
Represents a data request with processing state.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceRequest {
    id: String,
    layer: u32,
    visible: bool,
    request: RequestType,
    raw_data: Vec<u8>,
    processed_data: RTree<MapFeature>,
    llm_analysis: Vec<String>,
    last_query_date: i64,
}
```

**Request Types:**
- `OverpassTurboRequest`: OpenStreetMap data query
- `OpenRouterRequest`: AI/LLM analysis request  
- `OpenMeteoRequest`: Weather data request

### Geographic Data System

#### `MapFeature` Structure
Core geographic feature representation.

```rust
pub struct MapFeature {
    pub id: String,
    pub geometry: GeometryType,
    pub properties: serde_json::Value,
    pub closed: bool,
    pub feature_type: FeatureType,
}
```

**Methods:**
- `get_in_world_space(tile_manager)`: Converts to world coordinates
- `get_bounds()`: Returns feature bounding box
- `get_center()`: Calculates geometric center point

#### `GeometryType` Enum
Supported geometry types for geographic features.

```rust
pub enum GeometryType {
    Point(Point),
    LineString(LineString),
    Polygon(Polygon),
    MultiPoint(MultiPoint),
    MultiLineString(MultiLineString),  
    MultiPolygon(MultiPolygon),
}
```

### Tool System

#### `ToolResources` Resource
Manages interactive tool state and selections.

```rust
#[derive(Resource)]
pub struct ToolResources {
    pub pointer: bool,
    pub measure: bool,
    pub pin: bool,
    pub selection_areas: WorkspaceSelector,
}
```

#### Tool Trait
Base trait for all interactive tools.

```rust
pub trait Tool {
    fn activate(&mut self);
    fn deactivate(&mut self);
    fn handle_input(&mut self, input: &InputEvent) -> ToolResult;
    fn render(&self, ui: &mut egui::Ui);
}
```

### UI System

#### `ChatState` Resource
Manages AI chat interface state.

```rust
#[derive(Resource, Default)]
pub struct ChatState {
    pub input_text: String,
    pub chat_history: Vec<ChatMessage>,
    pub is_processing: bool,
    pub api_token: String,
    pub token_verified: bool,
}
```

#### `PersistentInfoWindows` Resource
Manages feature information popup windows.

```rust
#[derive(Resource, Default)]
pub struct PersistentInfoWindows {
    pub windows: HashMap<String, serde_json::Value>,
}
```

## API Integration

### Overpass API Client

#### `OverpassClient` Structure
```rust
#[derive(Clone)]
pub struct OverpassClient {
    url: String,
    agent: Agent,
    bounds: String,
    settings: Settings,
}
```

**Methods:**
- `new(url: &str)`: Creates new client instance
- `query(query_string: &str)`: Executes Overpass query
- `get_bounds(selection: Selection)`: Converts selection to bounds string
- `build_query(bounds: String, settings: Settings)`: Builds Overpass QL query

### LLM Integration

#### `OpenrouterClient` Structure
```rust
#[derive(Clone)]
pub struct OpenrouterClient {
    pub token: Option<String>,
    pub agent: Agent,
    pub url: String,
}
```

**Methods:**
- `new(url: impl Display, token: Option<String>)`: Creates client
- `send_chat_request(request: Request)`: Sends chat completion request
- `analyze_features(features: &[MapFeature])`: Analyzes geographic features

## System Architecture

### Plugin System
Each major feature is implemented as a Bevy plugin:

1. **WorkspacePlugin**: Core workspace management
   - Systems: `process_requests`, `cleanup_tasks`, `render_workspace_requests`
   - Resources: `Workspace`, `ChatState`, `PersistentInfoWindows`

2. **CameraSystemPlugin**: Camera controls and rendering
   - Systems: Camera movement, zoom handling, coordinate transforms
   - Components: `OldMonitorSettings`, camera entities

3. **RenderPlugin**: Geographic data rendering
   - Systems: GeoJSON tessellation, mesh construction, styling
   - Components: Rendered shapes, materials, visibility

4. **ToolsPlugin**: Interactive tools
   - Systems: Tool activation, input handling, UI rendering
   - Resources: `ToolResources`, tool state management

5. **InteractionSystemPlugin**: User input handling
   - Systems: File drop processing, mouse/keyboard input
   - Events: Input events, interaction state changes

### Data Flow

1. **Input Processing**: User interactions → Tool systems → State updates
2. **Data Requests**: Tool actions → Workspace worker → API clients
3. **Data Processing**: API responses → Parser → Spatial indexing → Storage
4. **Rendering**: Processed data → Tessellation → Mesh generation → GPU
5. **UI Updates**: State changes → UI systems → Visual feedback

### Rendering Pipeline

1. **Tessellation**: Complex polygons → Triangle meshes (using Lyon)
2. **Styling**: Feature properties → Colors, stroke widths, visibility
3. **Batching**: Similar features → Optimized draw calls
4. **Culling**: Viewport bounds → Visible features only
5. **GPU Rendering**: Mesh data → Vertex/fragment shaders → Frame buffer

## Performance Considerations

### Spatial Indexing
- Uses R-tree data structure for efficient spatial queries
- Enables fast intersection testing and nearest neighbor searches
- Supports dynamic updates as data changes

### Memory Management
- Thread-safe data structures for concurrent access
- Efficient memory pooling for frequently allocated objects
- Streaming data processing for large datasets

### Rendering Optimization
- Frustum culling for off-screen features
- Level-of-detail based on zoom level
- Instanced rendering for similar features
- GPU-based tessellation for complex shapes

## Error Handling

### Result Types
Most operations return `Result<T, E>` for graceful error handling:
- API requests: Network errors, rate limiting, invalid responses
- Data processing: Parse errors, invalid geometry, format issues
- File operations: I/O errors, permission issues, corrupted data

### Logging
Uses Bevy's logging system with configurable levels:
- `error!`: Critical failures requiring attention
- `warn!`: Non-critical issues that may affect functionality
- `info!`: General operational information
- `debug!`: Detailed debugging information
- `trace!`: Verbose execution tracing

## Extension Points

### Custom Tools
Implement the `Tool` trait to create new interactive tools:

```rust
pub struct CustomTool {
    // Tool state
}

impl Tool for CustomTool {
    fn activate(&mut self) { /* Activation logic */ }
    fn deactivate(&mut self) { /* Cleanup logic */ }
    fn handle_input(&mut self, input: &InputEvent) -> ToolResult { /* Input handling */ }
    fn render(&self, ui: &mut egui::Ui) { /* UI rendering */ }
}
```

### Custom Data Sources
Add new data sources by implementing request types and processing:

```rust
pub enum CustomRequestType {
    MyApiRequest(MyApiQuery),
}

// Add to RequestType enum and implement processing logic
```

### Custom Rendering
Extend the rendering system with custom shaders and materials:
- Add custom vertex/fragment shaders in `assets/shaders/`
- Implement custom material types
- Add specialized rendering systems for new feature types

This documentation provides a comprehensive overview of the Map-RS API structure and usage patterns. For specific implementation details, refer to the individual module source files.
