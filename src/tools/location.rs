use bevy::{prelude::*, window::PrimaryWindow};
use rstar::{RTree, RTreeObject, AABB};

use crate::{tiles::{ChunkManager, ZoomManager}, types::{world_mercator_to_lat_lon, Coord}, EguiBlockInputState};

pub struct PinPlugin;

impl Plugin for PinPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Pins::default())
            .add_systems(Update, (render_pins, handle_pin));
    }
}

#[derive(Resource, Clone)]
pub struct Pins {
    pins: RTree<Pin>,
    pub enabled: bool,
    pub respawn: bool,
}

impl Pins {
    pub fn default() -> Self {
        Pins {
            pins: RTree::new(),
            enabled: false,
            respawn: false,
        }
    }

    pub fn add_pin(&mut self, pin: Pin) {
        self.pins.insert(pin);
    }
}

#[derive(Component, Clone)]
pub struct Pin {
    pub location: Coord,
    // Add the ability to change the icon. Do that in a similar way to the settings for the overpass query.
}

impl RTreeObject for Pin {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.location.lat, self.location.long])
    }
}

impl Pin {
    pub fn get_in_world_space(&self, reference: Coord, zoom: u32, tile_quality: f64) -> Vec2 {
        self.location.to_game_coords(reference, zoom, tile_quality)
    }
}

pub fn handle_pin(
    mut pin: ResMut<Pins>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    zoom_manager: Res<ZoomManager>,
    chunk_manager: Res<ChunkManager>,
    state: Res<EguiBlockInputState>,
) {
    let (camera, camera_transform) = camera.single();
    if pin.enabled && !state.block_input{
        if let Some(position) = q_windows.single().cursor_position() {
            if buttons.just_pressed(MouseButton::Left) {
                let world_pos = camera.viewport_to_world_2d(camera_transform, position).unwrap();
                let pos = world_mercator_to_lat_lon(world_pos.x.into(), world_pos.y.into(), chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size);
                pin.add_pin(Pin {
                    location: Coord::new(pos.lat as f32, pos.long as f32),
                });
                pin.respawn = true;
            }
        }
    }
}

fn render_pins(
    mut commands: Commands,
    selections_query: Query<(Entity, &Transform, &Pin)>,
    zoom_manager: Res<ZoomManager>,
    chunk_manager: Res<ChunkManager>,
    mut pins: ResMut<Pins>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if pins.respawn {
        pins.respawn = false;
        for (entity, _, _) in selections_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        let fill_color = Srgba { red: 0., green: 0., blue: 1., alpha: 0.75 };
        let width = 5.;
        let elevation = 10.0;

        for pin in pins.pins.iter() {
            let loc = pin.get_in_world_space(chunk_manager.refrence_long_lat, zoom_manager.zoom_level, zoom_manager.tile_size.into());
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(width))),
                Transform::from_translation(Vec3::new(loc.x, loc.y, elevation)),
                MeshMaterial2d(materials.add(Color::from(fill_color))),
                pin.clone(),
            ));
        }
        
    }
}


const _PIN_ICON: [&str; 2] = [
    "M12 2a8 8 0 0 0-2.565 15.572L12 21.992l2.565-4.42A8 8 0 0 0 12 2zm1.626 13.77-.391.11L12 18.008l-1.235-2.128-.391-.11a6 6 0 1 1 3.252 0z",
    "M12 7a3 3 0 1 0 3 3 3 3 0 0 0-3-3zm0 4a1 1 0 1 1 1-1 1 1 0 0 1-1 1z",
];
