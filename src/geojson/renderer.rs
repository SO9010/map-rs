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
// TODO: Find out how to colour the shapes induvidually
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

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0;

        for feature in intersection_candidates {
            let mut shape = feature.get_in_world_space(tile_map_manager.clone());
            shape.reverse();
            let shape_vertices: Vec<[f32; 3]> = shape
                .iter()
                .map(|point| [point.x as f32, point.y as f32, 0.0])
                .collect::<Vec<_>>();

            let shape_indices: Vec<u32> = (1..shape.len() as u32 - 1)
                .flat_map(|i| vec![0, i, i + 1])
                .collect();

            vertices.extend(shape_vertices);
            indices.extend(shape_indices.iter().map(|i| i + index_offset));

            index_offset += shape.len() as u32;
        }
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone())
        .with_inserted_indices(Indices::U32(indices));
        for (entity, _) in shapes_query.iter() {
            commands.entity(entity).despawn();
        }
        commands.spawn((
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(
                materials.add(ColorMaterial::from_color(Srgba::new(0.6, 0.6, 0.6, 0.5))),
            ),
            Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ShapeMarker,
            RenderLayers::layer(1),
        ));
    }
}
