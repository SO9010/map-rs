use std::thread;

// Thank you for the example: https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/chunking.rs
use bevy::{prelude::*, utils::{HashMap, HashSet}};
use bevy_ecs_tilemap::{map::{TilemapGridSize, TilemapId, TilemapTexture, TilemapTileSize}, tiles::{TileBundle, TilePos, TileStorage}, TilemapBundle, TilemapPlugin};
use crossbeam_channel::{bounded, Receiver, Sender};

use crate::{types::{world_mercator_to_lat_lon, Coord}, EguiBlockInputState, STARTING_DISPLACEMENT, STARTING_LONG_LAT, TILE_QUALITY};
use super::ui::TilesUiPlugin;
#[allow(unused_imports)]
use super::{buffer_to_bevy_image, get_mvt_data, get_rasta_data};

// For this example, don't choose too large a chunk size.
const CHUNK_SIZE: UVec2 = UVec2 { x: 1, y: 1 };

pub struct TileMapPlugin;

// TODO: We probably want to change it so that the left tile changes where the center is.
impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx): (ChunkSenderType, ChunkReceiverType) = bounded(10);
        app.insert_resource(ChunkReceiver(rx))
            .insert_resource(ChunkSender(tx))
            .add_plugins(TilemapPlugin)
            .insert_resource(TileMapResources::default())
            .add_systems(FixedUpdate, (spawn_chunks_around_camera, spawn_to_needed_chunks))
            .add_systems(Update, detect_zoom_level)
            .add_systems(FixedUpdate, (despawn_outofrange_chunks, read_tile_map_receiver))
            .add_plugins(TilesUiPlugin);
    }
}

#[derive(Debug, Resource, Clone, Default)]
pub struct TileMapResources {
    pub zoom_manager: ZoomManager,
    pub chunk_manager: ChunkManager,
    pub location_manager: Location,
}

#[derive(Debug, Clone)]
pub struct ZoomManager {
    pub zoom_level: u32,
    pub last_zoom_level: u32,
    pub last_projection_level: f32,
    pub tile_size: f32
}

impl Default for ZoomManager {
    fn default() -> Self {
        Self {
            zoom_level: 14,
            last_zoom_level: 0,
            last_projection_level: 0.0,
            tile_size: TILE_QUALITY as f32
        }
    }
}

#[derive(Debug, Clone)]
pub enum TileType {
    Raster,
    Vector
}

#[derive(Debug, Clone)]
pub struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>,
    pub to_spawn_chunks: HashMap<IVec2, Vec<u8>>, // Store raw image data
    pub update: bool, // Store raw image data
    pub refrence_long_lat: Coord,
    pub tile_web_origin: HashMap<String, (bool, TileType)>,
    pub tile_web_origin_changed: bool,
}

impl ChunkManager {
    pub fn add_tile_web_origin(&mut self, url: String, enabled: bool, tile_type: TileType) {
        self.tile_web_origin.insert(url, (enabled, tile_type));
    }

    pub fn enable_tile_web_origin(&mut self, url: &str) {
        if let Some((enabled, _)) = self.tile_web_origin.get_mut(url) {
            *enabled = true;
        }
    }
    
    pub fn disable_all_tile_web_origins(&mut self) {
        for (_, (enabled, _)) in self.tile_web_origin.iter_mut() {
            *enabled = false;
        }
    }
    
    pub fn enable_only_tile_web_origin(&mut self, url: &str) {
        self.disable_all_tile_web_origins();
        
        if let Some((enabled, _)) = self.tile_web_origin.get_mut(url) {
            *enabled = true;
            self.tile_web_origin_changed = true;
        }
    }

    pub fn get_enabled_tile_web_origins(&self) -> Option<(&String, (&bool, &TileType))> {
        for (url, (enabled, tile_type)) in &self.tile_web_origin {
            if *enabled {
                return Some((url, (enabled, tile_type)));
            }
        }
        None
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        let mut tile_web_origin = HashMap::default();
        tile_web_origin.insert("https://tile.openstreetmap.org".to_string(), (false, TileType::Raster));
        tile_web_origin.insert("https://mt1.google.com/vt/lyrs=y".to_string(), (true, TileType::Raster));
        tile_web_origin.insert("https://mt1.google.com/vt/lyrs=m".to_string(), (false, TileType::Raster));
        tile_web_origin.insert("https://mt1.google.com/vt/lyrs=s".to_string(), (false, TileType::Raster));
        tile_web_origin.insert("https://tiles.openfreemap.org/planet/20250122_001001_pt".to_string(), (false, TileType::Vector));
        Self {
            spawned_chunks: HashSet::default(),
            to_spawn_chunks: HashMap::default(),
            update: true,
            refrence_long_lat: STARTING_LONG_LAT,
            tile_web_origin,
            tile_web_origin_changed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Location {
    pub location: Coord,
}

impl Default for Location {
    fn default() -> Self {
        Self {
            location: STARTING_DISPLACEMENT,
        }
    }
}

fn detect_zoom_level(
    mut res_manager: ResMut<TileMapResources>,
    mut ortho_projection_query: Query<&mut OrthographicProjection, With<Camera>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    commands: Commands,
    chunk_query: Query<(Entity, &TileMarker)>,
    state: Res<EguiBlockInputState>,
) {
    if let Ok(mut projection) = ortho_projection_query.get_single_mut() {
        if let Ok(mut camera) = camera_query.get_single_mut() {
            if projection.scale != res_manager.zoom_manager.last_projection_level && !state.block_input {
                res_manager.zoom_manager.last_projection_level = projection.scale;
                if projection.scale > 1. && projection.scale != 0. && res_manager.zoom_manager.zoom_level > 3 {
                    res_manager.zoom_manager.last_zoom_level = res_manager.zoom_manager.zoom_level;
                    res_manager.zoom_manager.zoom_level -= 1;

                    despawn_all_chunks(commands, chunk_query);
                    res_manager.chunk_manager.spawned_chunks.clear();
                    res_manager.chunk_manager.to_spawn_chunks.clear();

                    // This ensures that the tile size stays correct
                    res_manager.chunk_manager.refrence_long_lat *= Coord {lat: 2., long: 2.};

                    camera.translation = res_manager.location_manager.location.to_game_coords(res_manager.chunk_manager.refrence_long_lat, res_manager.zoom_manager.zoom_level, res_manager.zoom_manager.tile_size.into()).extend(1.0);
                    
                    projection.scale = 1.0;
                } else if projection.scale < 1.0 && projection.scale != 0. && res_manager.zoom_manager.zoom_level < 19 {
                    res_manager.zoom_manager.last_zoom_level = res_manager.zoom_manager.zoom_level;
                    res_manager.zoom_manager.zoom_level += 1;

                    despawn_all_chunks(commands, chunk_query);
                    res_manager.chunk_manager.spawned_chunks.clear();
                    res_manager.chunk_manager.to_spawn_chunks.clear();

                    // This ensures that the tile size stays correct
                    res_manager.chunk_manager.refrence_long_lat /= Coord {lat: 2., long: 2.};
                    
                    camera.translation = res_manager.location_manager.location.to_game_coords(res_manager.chunk_manager.refrence_long_lat, res_manager.zoom_manager.zoom_level, res_manager.zoom_manager.tile_size.into()).extend(1.0);

                    projection.scale = 1.0;
                }
                res_manager.chunk_manager.update = true;
            } else if res_manager.chunk_manager.tile_web_origin_changed {
                res_manager.chunk_manager.tile_web_origin_changed = false;
                despawn_all_chunks(commands, chunk_query);
                res_manager.chunk_manager.spawned_chunks.clear();
                res_manager.chunk_manager.to_spawn_chunks.clear();
                res_manager.chunk_manager.update = true;
            }
        }
    }
}

pub type ChunkData = (IVec2, Vec<u8>);
pub type ChunkSenderType = Sender<ChunkData>;
pub type ChunkReceiverType = Receiver<ChunkData>;

#[derive(Resource, Deref)]
pub struct ChunkReceiver(Receiver<(IVec2, Vec<u8>)>); // Use Vec<u8> for raw image data

#[derive(Resource, Deref)]
pub struct ChunkSender(Sender<(IVec2, Vec<u8>)>);

#[derive(Component)]
pub struct TileMarker;

fn spawn_chunk(
    commands: &mut Commands,
    tile: Handle<Image>,
    chunk_pos: IVec2,
    tile_size: f32,
) {
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());

    let tile_pos = TilePos { x: 0, y: 0 };
    let tile_entity = commands
        .spawn(TileBundle {
            position: tile_pos,
            tilemap_id: TilemapId(tilemap_entity),
            ..Default::default()
        })
        .insert(TileMarker)
        .id();
    commands.entity(tilemap_entity).add_child(tile_entity);
    tile_storage.set(&tile_pos, tile_entity);

    let transform = Transform::from_translation(Vec3::new(
        chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * tile_size,
        chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * tile_size,
        0.0,
    ));

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: TilemapGridSize::from(TilemapTileSize { x: tile_size, y: tile_size }),
        size: CHUNK_SIZE.into(),
        storage: tile_storage,
        texture: TilemapTexture::Single(tile),
        tile_size: TilemapTileSize { x: tile_size, y: tile_size },
        transform,
        ..Default::default()
    }).insert(TileMarker);
}

fn camera_pos_to_chunk_pos(camera_pos: &Vec2, tile_size: f32) -> IVec2 {
    let chunk_size = Vec2::new(
        CHUNK_SIZE.x as f32 * tile_size,
        CHUNK_SIZE.y as f32 * tile_size,
    );
    let camera_pos = Vec2::new(camera_pos.x, camera_pos.y) / chunk_size;
    camera_pos.floor().as_ivec2()
}


fn chunk_pos_to_world_pos(chunk_pos: IVec2, tile_size: f32) -> Vec2 {
    let chunk_size = Vec2::new(
        CHUNK_SIZE.x as f32 * tile_size,
        CHUNK_SIZE.y as f32 * tile_size,
    );
    Vec2::new(
        chunk_pos.x as f32 * chunk_size.x,
        chunk_pos.y as f32 * chunk_size.y,
    )
}

fn spawn_chunks_around_camera(
    camera_query: Query<&Transform, With<Camera>>,
    chunk_sender: Res<ChunkSender>,  // Use the stored sender
    mut res_manager: ResMut<TileMapResources>,
) {
    if res_manager.chunk_manager.update {
        res_manager.chunk_manager.update = false;

        let chunk_manager_clone = res_manager.chunk_manager.clone();
        let enabled_origins = chunk_manager_clone.get_enabled_tile_web_origins();
        if let Some((url, (_, tile_type))) = enabled_origins {
            for transform in camera_query.iter() {
                let camera_chunk_pos = camera_pos_to_chunk_pos(&transform.translation.xy(), res_manager.zoom_manager.tile_size);
                let range = 4;

            for y in (camera_chunk_pos.y - range)..=(camera_chunk_pos.y + range) {
                for x in (camera_chunk_pos.x - range)..=(camera_chunk_pos.x + range) {
                    let chunk_pos = IVec2::new(x, y);
                    if !res_manager.chunk_manager.spawned_chunks.contains(&chunk_pos) {
                        let tx = chunk_sender.clone(); // Clone existing sender
                        let zoom_manager = res_manager.zoom_manager.clone();
                        let refrence_long_lat = res_manager.chunk_manager.refrence_long_lat;
                        let world_pos = chunk_pos_to_world_pos(chunk_pos, zoom_manager.tile_size);
                        let position = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size);
                        let url = url.clone();
                        let tile_type = tile_type.clone();
                        thread::spawn(move || {
                            let tile_coords = position.to_tile_coords(zoom_manager.zoom_level);

                            match tile_type {
                                TileType::Raster => {
                                    let tile_image = get_rasta_data(tile_coords.x as u64, tile_coords.y as u64, zoom_manager.zoom_level as u64, url.to_string());
                                    if let Err(e) = tx.send((chunk_pos, tile_image)) {
                                        eprintln!("Failed to send chunk data: {:?}", e);
                                    }
                                },
                                TileType::Vector => {
                                    let tile_image = get_mvt_data(tile_coords.x as u64, tile_coords.y as u64, zoom_manager.zoom_level as u64, zoom_manager.tile_size as u32, url.to_string());
                                    if let Err(e) = tx.send((chunk_pos, tile_image)) {
                                        eprintln!("Failed to send chunk data: {:?}", e);
                                    }
                                }
                            }
                        });

                        res_manager.chunk_manager.spawned_chunks.insert(chunk_pos);
                    }
                }
            }
        }
        }
    }
}

fn read_tile_map_receiver(
    map_receiver: Res<ChunkReceiver>,
    mut res_manager: ResMut<TileMapResources>,
) {
    let mut new_chunks = Vec::new();

    while let Ok((chunk_pos, raw_image_data)) = map_receiver.try_recv() {
        if !res_manager.chunk_manager.to_spawn_chunks.contains_key(&chunk_pos) {
            new_chunks.push((chunk_pos, raw_image_data));
        }
    }

    for (pos, data) in new_chunks {
        res_manager.chunk_manager.to_spawn_chunks.insert(pos, data);
    }
}

fn spawn_to_needed_chunks(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut res_manager: ResMut<TileMapResources>,
) {
    let to_spawn_chunks: Vec<(IVec2, Vec<u8>)> = res_manager.chunk_manager.to_spawn_chunks.iter().map(|(pos, data)| (*pos, data.clone())).collect();
    for (chunk_pos, raw_image_data) in to_spawn_chunks {
        let tile_handle = images.add(buffer_to_bevy_image(raw_image_data, res_manager.zoom_manager.tile_size as u32));
        spawn_chunk(&mut commands, tile_handle, chunk_pos, res_manager.zoom_manager.tile_size);
        res_manager.chunk_manager.spawned_chunks.insert(chunk_pos);
    }
    res_manager.chunk_manager.to_spawn_chunks.clear();
}

fn despawn_outofrange_chunks(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    chunks_query: Query<(Entity, &Transform, &TileMarker)>,
    mut res_manager: ResMut<TileMapResources>,
) {
    for camera_transform in camera_query.iter() {
        for (entity, chunk_transform, _) in chunks_query.iter() {
            let chunk_pos = chunk_transform.translation.xy();
            let distance = camera_transform.translation.xy().distance(chunk_pos);
            if distance > res_manager.zoom_manager.tile_size * 10.{
                let x = (chunk_pos.x / (CHUNK_SIZE.x as f32 * res_manager.zoom_manager.tile_size)).floor() as i32;
                let y = (chunk_pos.y / (CHUNK_SIZE.y as f32 * res_manager.zoom_manager.tile_size)).floor() as i32;
                res_manager.chunk_manager.spawned_chunks.remove(&IVec2::new(x, y));
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn despawn_all_chunks(
    mut commands: Commands,
    chunks_query: Query<(Entity, &TileMarker)>,
) {
    for (entity, _) in chunks_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}