use bevy::{prelude::*, render::view::RenderLayers, window::PrimaryWindow};
use bevy_map_viewer::{Coord, EguiBlockInputState, MapViewerMarker, TileMapResources};
use rstar::{AABB, RTree, RTreeObject};

use super::ToolResources;

pub struct PinPlugin;

impl Plugin for PinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (render_pins, handle_pin));
    }
}

#[derive(Clone)]
pub struct Pins {
    pins: RTree<Pin>,
    pub enabled: bool,
    pub respawn: bool,
}

impl Pins {
    pub fn add_pin(&mut self, pin: Pin) {
        self.pins.insert(pin);
    }
}

impl Default for Pins {
    fn default() -> Self {
        Pins {
            pins: RTree::new(),
            enabled: false,
            respawn: false,
        }
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
    pub fn get_in_world_space(&self, tile_map_resources: TileMapResources) -> Vec2 {
        self.location.to_game_coords(tile_map_resources)
    }
}

pub fn handle_pin(
    mut pin: ResMut<ToolResources>,
    camera: Query<(&Camera, &GlobalTransform), With<MapViewerMarker>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    tile_map_manager: Res<TileMapResources>,
    state: Res<EguiBlockInputState>,
) {
    let (camera, camera_transform) = match camera.single() {
        Ok(result) => result,
        Err(_) => return,
    };
    if pin.pins.enabled && !state.block_input {
        if let Some(position) = q_windows
            .single()
            .expect("Cant get cursor position")
            .cursor_position()
        {
            if buttons.just_pressed(MouseButton::Left) {
                let pos = tile_map_manager.point_to_coord(
                    camera
                        .viewport_to_world_2d(camera_transform, position)
                        .unwrap(),
                );
                pin.pins.add_pin(Pin {
                    location: Coord::new(pos.lat, pos.long),
                });
                pin.pins.respawn = true;
            }
        }
    }
}

fn render_pins(
    mut commands: Commands,
    selections_query: Query<(Entity, &Transform, &Pin)>,
    tile_map_manager: Res<TileMapResources>,
    mut pin: ResMut<ToolResources>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if pin.pins.respawn {
        pin.pins.respawn = false;
        for (entity, _, _) in selections_query.iter() {
            commands.entity(entity).despawn();
        }

        let fill_color = Srgba {
            red: 0.,
            green: 0.,
            blue: 1.,
            alpha: 0.75,
        };
        let width = 5.;
        let elevation = 500.0;

        for pin in pin.pins.pins.iter() {
            let loc = pin.get_in_world_space(tile_map_manager.clone());
            commands.spawn((
                Mesh2d(meshes.add(Circle::new(width))),
                Transform::from_translation(Vec3::new(loc.x, loc.y, elevation)),
                MeshMaterial2d(materials.add(Color::from(fill_color))),
                pin.clone(),
                RenderLayers::layer(1),
            ));
        }
    }
}

const _PIN_ICON: [&str; 2] = [
    "M12 2a8 8 0 0 0-2.565 15.572L12 21.992l2.565-4.42A8 8 0 0 0 12 2zm1.626 13.77-.391.11L12 18.008l-1.235-2.128-.391-.11a6 6 0 1 1 3.252 0z",
    "M12 7a3 3 0 1 0 3 3 3 3 0 0 0-3-3zm0 4a1 1 0 1 1 1-1 1 1 0 0 1-1 1z",
];
