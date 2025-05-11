use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        view::RenderLayers,
    },
};
use bevy_map_viewer::TileMapResources;
use rstar::RTreeObject;

use crate::workspace::Workspace;
use bevy_map_viewer::ZoomChangedEvent;

use super::MapFeature;

#[derive(Component)]
pub struct ShapeMarker;
// TODO: Fix the issue where the shapes overlap themselves.
pub fn respawn_shapes(
    mut commands: Commands,
    shapes_query: Query<(Entity, &ShapeMarker)>,
    tile_map_manager: Res<TileMapResources>,
    workspace: Res<Workspace>,
    mut zoom_change: EventReader<ZoomChangedEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !zoom_change.is_empty() {
        zoom_change.clear();

        let mut intersection_candidates: Vec<MapFeature> = Vec::new();
        let ws = if let Some(ws) = workspace.workspace.clone() {
            ws
        } else {
            return;
        };

        // Now all we need is settings to get the needed values to turn to different colours.
        let colors: HashMap<(String, serde_json::Value), Srgba> = ws.get_color_properties();

        let mut hashmap: HashMap<(String, serde_json::Value), MeshConstructor> = HashMap::new();
        info!("{}", colors.len());
        if let Some(selection) = &workspace.workspace {
            for i in workspace.get_rendered_requests() {
                if i.get_processed_data().size() == 0 {
                    continue;
                }
                let map_bundle = i.clone().get_processed_data();
                intersection_candidates.extend(
                    map_bundle
                        .locate_in_envelope_intersecting(&selection.envelope())
                        .cloned()
                        .collect::<Vec<_>>(),
                );
            }
        }

        // TODO: Fix roads
        for feature in intersection_candidates {
            let shape = feature.get_in_world_space(tile_map_manager.clone());
            let shape_vertices: Vec<[f32; 3]> = shape
                .iter()
                .map(|point| [point.x, point.y, 0.0])
                .collect::<Vec<_>>();
            for (item, c) in colors.clone().into_iter() {
                if feature.properties.get(item.0.clone()).is_none() {
                    continue;
                }
                let value = feature.properties.get(item.0.clone()).unwrap();
                if value != &item.1 {
                    continue;
                }
                if let Some(v) = hashmap.get_mut(&item) {
                    v.add_shape(shape_vertices.clone());
                    v.add_color(c);
                } else {
                    hashmap.insert(item.clone(), MeshConstructor::new());

                    if let Some(v) = hashmap.get_mut(&item) {
                        v.add_shape(shape_vertices.clone());
                        v.add_color(c);
                    }
                }
            }
            let default = &(
                "default".to_string(),
                serde_json::Value::String("default".to_string()),
            );
            if hashmap.get(default).is_some() {
                if let Some(v) = hashmap.get_mut(default) {
                    v.add_shape(shape_vertices);
                }
            } else {
                hashmap.insert(default.clone(), MeshConstructor::new());

                if let Some(v) = hashmap.get_mut(default) {
                    v.add_shape(shape_vertices);
                }
            }
        }

        for (entity, _) in shapes_query.iter() {
            commands.entity(entity).despawn();
        }
        for t in hashmap.into_iter() {
            commands.spawn((
                Mesh2d(meshes.add(t.1.to_mesh())),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(t.1.get_color()))),
                Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                ShapeMarker,
                RenderLayers::layer(1),
            ));
        }
    }
}

struct MeshConstructor {
    vertices: Vec<[f32; 3]>,
    indices: Vec<u32>,
    index_offset: u32,
    color: Srgba,
}

impl MeshConstructor {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            index_offset: 0,
            color: Srgba::new(0.5, 0.5, 0.5, 0.5),
        }
    }
    fn add_color(&mut self, color: Srgba) {
        self.color = color;
    }
    fn get_color(&self) -> Srgba {
        self.color
    }
    fn add_shape(&mut self, shape: Vec<[f32; 3]>) {
        let shape_indices: Vec<u32> = (1..shape.len() as u32 - 1)
            .flat_map(|i| vec![0, i, i + 1])
            .collect();

        self.vertices.extend(&shape);
        self.indices
            .extend(shape_indices.iter().map(|i| i + self.index_offset));

        self.index_offset += shape.len() as u32;
    }
    fn to_mesh(&self) -> Mesh {
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices.clone())
        .with_inserted_indices(Indices::U32(self.indices.clone()));
        mesh
    }
}
