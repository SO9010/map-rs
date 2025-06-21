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
use lyon::{
    math::point,
    path::Path,
    tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
        StrokeVertex, VertexBuffers,
    },
};
use rstar::RTreeObject;

use crate::workspace::Workspace;
use bevy_map_viewer::ZoomChangedEvent;

use super::MapFeature;
#[derive(Component)]
pub struct ShapeMarker;

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
        let mut intersection_candidates: Vec<MapFeature> = Vec::new();
        let ws: crate::workspace::WorkspaceData = if let Some(ws) = workspace.workspace.clone() {
            ws
        } else {
            return;
        };
        if tile_map_manager.zoom_manager.zoom_level < 14 {
            return;
        }
        // Ok so we want to always add a displacement of tile_map_manager.chunk_manager.displacement rather than recalculating.
        // Now all we need is settings to get the needed values to turn to different colours.
        let colors: HashMap<(String, serde_json::Value), Srgba> = ws.get_color_properties();

        let mut hashmap: HashMap<(String, serde_json::Value), MeshConstructor> = HashMap::new();
        info!("{}", colors.len());
        if let Some(selection) = &workspace.workspace {
            for i in workspace.get_rendered_requests() {
                if i.get_processed_data().size() == 0 || !i.get_visible() {
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

        for feature in intersection_candidates {
            let shape = feature.get_in_world_space(tile_map_manager.clone());
            let shape_vertices: Vec<[f32; 3]> =
                shape.iter().map(|point| [point.x, point.y, 0.0]).collect();

            for (item, c) in &colors {
                if let Some(value) = feature.properties.get(item.0.clone()) {
                    if value == &item.1 {
                        let entry = hashmap
                            .entry(item.clone())
                            .or_insert_with(MeshConstructor::new);
                        entry.add_shape(shape_vertices.clone(), feature.closed);
                        entry.add_color(*c);
                    }
                }
            }

            let default_key = (
                "default".to_string(),
                serde_json::Value::String("default".to_string()),
            );
            let entry = hashmap
                .entry(default_key)
                .or_insert_with(MeshConstructor::new);
            entry.add_shape(shape_vertices, feature.closed);
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
    fn add_shape(&mut self, shape: Vec<[f32; 3]>, closed: bool) {
        // Build a path using the provided shape vertices
        let mut builder = Path::builder();
        if let Some(first_point) = shape.first() {
            builder.begin(point(first_point[0], first_point[1]));
            for p in &shape[1..] {
                builder.line_to(point(p[0], p[1]));
            }
            builder.end(closed);
        }
        let path = builder.build();

        // Tessellate the path
        #[derive(Copy, Clone, Debug)]
        struct MyVertex {
            position: [f32; 2],
        }

        let mut geometry: VertexBuffers<MyVertex, u16> = VertexBuffers::new();
        match closed {
            true => {
                let mut tessellator = FillTessellator::new();
                tessellator
                    .tessellate_path(
                        &path,
                        &FillOptions::default(),
                        &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| MyVertex {
                            position: vertex.position().to_array(),
                        }),
                    )
                    .unwrap();
            }
            false => {
                let mut tessellator = StrokeTessellator::new();
                tessellator
                    .tessellate_path(
                        &path,
                        &StrokeOptions::default().with_line_width(1.0),
                        &mut BuffersBuilder::new(&mut geometry, |vertex: StrokeVertex| MyVertex {
                            position: vertex.position().to_array(),
                        }),
                    )
                    .unwrap();
            }
        }

        // Convert tessellated vertices and indices to the format used by MeshConstructor
        self.vertices.extend(
            geometry
                .vertices
                .iter()
                .map(|v| [v.position[0], v.position[1], 0.0]),
        );
        self.indices.extend(
            geometry
                .indices
                .iter()
                .map(|&i| i as u32 + self.index_offset),
        );

        self.index_offset += geometry.vertices.len() as u32;
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
