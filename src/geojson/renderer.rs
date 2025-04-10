use bevy::{prelude::*, render::view::RenderLayers};
use bevy_map_viewer::TileMapResources;
use bevy_prototype_lyon::{
    draw::{Fill, Stroke},
    entity::ShapeBundle,
    prelude::GeometryBuilder,
    shapes,
};
use rstar::RTreeObject;

use crate::{overpass::OverpassClientResource, tools::ToolResources, types::MapBundle};
use bevy_map_viewer::ZoomChangedEvent;

#[derive(Component)]
pub struct ShapeMarker;
// TODO: Change to a mesh 2d]
// We need to find a way to render it with each different part having its own settings. I recon we could possibly add the data as a child or refrence to the selection? So for the selection we have selection settigns.
pub fn respawn_shapes(
    mut commands: Commands,
    shapes_query: Query<(Entity, &ShapeMarker)>,
    map_bundle: ResMut<MapBundle>,
    tile_map_manager: Res<TileMapResources>,
    overpass_settings: Res<OverpassClientResource>,
    tools: Res<ToolResources>,
    mut zoom_change: EventReader<ZoomChangedEvent>,
) {
    if !zoom_change.is_empty() {
        zoom_change.clear();
        for (entity, _) in shapes_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        let mut batch_commands_closed: Vec<(ShapeBundle, Fill, Stroke, ShapeMarker, RenderLayers)> =
            Vec::new();
        let mut batch_commands_open: Vec<(ShapeBundle, Stroke, ShapeMarker, RenderLayers)> =
            Vec::new();

        let mut intersection_candidates: Vec<&crate::types::MapFeature> = Vec::new();
        if let Some(selection) = &tools.selection_areas.focused_selection {
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
            let elevation = 1.0;

            for (cat, key) in overpass_settings
                .client
                .settings
                .get_true_keys_with_category_with_individual()
                .iter()
            {
                if let Some(cate) = feature.properties.get(cat.to_lowercase()) {
                    if cate.as_str().unwrap() != key {
                        continue;
                    }
                    let color = overpass_settings
                        .client
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
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&shape),
                        transform: Transform::from_xyz(0.0, 0.0, elevation),
                        ..default()
                    },
                    Fill::color(fill_color),
                    Stroke::new(stroke_color, line_width as f32),
                    ShapeMarker,
                    RenderLayers::layer(1),
                ));
            } else {
                let shape = shapes::Polygon {
                    points: points.clone(),
                    closed: false,
                };
                batch_commands_open.push((
                    ShapeBundle {
                        path: GeometryBuilder::build_as(&shape),
                        transform: Transform::from_xyz(0.0, 0.0, elevation),
                        ..default()
                    },
                    Stroke::new(stroke_color, line_width as f32),
                    ShapeMarker,
                    RenderLayers::layer(1),
                ));
            }
        }

        commands.spawn_batch(batch_commands_closed);
        commands.spawn_batch(batch_commands_open);
    }
}
