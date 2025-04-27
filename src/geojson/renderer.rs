use bevy::{prelude::*, render::view::RenderLayers};
use bevy_map_viewer::TileMapResources;
use bevy_prototype_lyon::{
    entity::Shape,
    prelude::{ShapeBuilder, ShapeBuilderBase},
    shapes,
};
use rstar::RTreeObject;

use crate::workspace::Workspace;
use bevy_map_viewer::ZoomChangedEvent;

use super::{MapBundle, MapFeature};

#[derive(Component)]
pub struct ShapeMarker;
// TODO: Change to a mesh 2d
// We need to find a way to render it with each different part having its own settings. I recon we could possibly add the data as a child or refrence to the selection? So for the selection we have selection settigns.
pub fn respawn_shapes(
    mut commands: Commands,
    shapes_query: Query<(Entity, &ShapeMarker)>,
    map_bundle: ResMut<MapBundle>,
    tile_map_manager: Res<TileMapResources>,
    workspace: Res<Workspace>,
    mut zoom_change: EventReader<ZoomChangedEvent>,
) {
    if !zoom_change.par_read().is_empty() {
        zoom_change.clear();
        for (entity, _) in shapes_query.iter() {
            commands.entity(entity).despawn();
        }

        let mut batch_commands_closed: Vec<(Shape, ShapeMarker, RenderLayers)> = Vec::new();
        let mut batch_commands_open: Vec<(Shape, ShapeMarker, RenderLayers)> = Vec::new();

        let mut intersection_candidates: Vec<&MapFeature> = Vec::new();
        if let Some(selection) = &workspace.workspace {
            intersection_candidates = map_bundle
                .features
                .locate_in_envelope_intersecting(&selection.envelope())
                .collect::<Vec<_>>();
        }

        for feature in intersection_candidates {
            let mut fill_color = Srgba {
                red: 0.,
                green: 0.5,
                blue: 0.,
                alpha: 0.5,
            };
            let mut stroke_color = Srgba {
                red: 0.,
                green: 0.5,
                blue: 0.,
                alpha: 0.75,
            };
            let line_width = 0.025;

            for (cat, key) in workspace
                .overpass_agent
                .settings
                .get_true_keys_with_category_with_individual()
                .iter()
            {
                if let Some(cate) = feature.properties.get(cat.to_lowercase()) {
                    if cate.as_str().unwrap() != key {
                        continue;
                    }
                    let color = workspace
                        .overpass_agent
                        .settings
                        .categories
                        .get(cat)
                        .unwrap()
                        .items
                        .get(key)
                        .unwrap()
                        .1;
                    stroke_color = Srgba {
                        red: (color.0[0] as f32) / 210.,
                        green: (color.0[1] as f32) / 210.,
                        blue: (color.0[2] as f32) / 210.,
                        alpha: 0.50,
                    };
                    fill_color = Srgba {
                        red: (color.0[0] as f32) / 210.,
                        green: (color.0[1] as f32) / 210.,
                        blue: (color.0[2] as f32) / 210.,
                        alpha: 0.50,
                    };
                }
            }

            let mut points = feature.get_in_world_space(tile_map_manager.clone());

            points.pop();

            if feature.closed {
                let shape = shapes::Polygon {
                    points: points.clone(),
                    closed: true,
                };

                batch_commands_closed.push((
                    ShapeBuilder::with(&shape)
                        .fill(fill_color)
                        .stroke((stroke_color, line_width as f32))
                        .build(),
                    ShapeMarker,
                    RenderLayers::layer(1),
                ));
            } else {
                let shape = shapes::Polygon {
                    points: points.clone(),
                    closed: false,
                };
                batch_commands_open.push((
                    ShapeBuilder::with(&shape)
                        .stroke((stroke_color, line_width as f32))
                        .build(),
                    ShapeMarker,
                    RenderLayers::layer(1),
                ));
            }
        }

        commands.spawn_batch(batch_commands_closed);
        commands.spawn_batch(batch_commands_open);
    }
}
