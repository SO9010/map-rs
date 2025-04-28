use bevy::{
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    ecs::query::QueryItem,
    prelude::*,
    render::{
        RenderApp,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::{RenderLayers, ViewTarget},
    },
};
use bevy_map_viewer::{Coord, MapViewerMarker, MapViewerPlugin, TileMapResources};
use bevy_pancam::{DirectionKeys, PanCam, PanCamPlugin};

use bevy_map_viewer::EguiBlockInputState;

const SHADER_ASSET_PATH: &str = "shaders/full_screen_pass.wgsl";

pub struct CameraSystemPlugin;

impl Plugin for CameraSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanCamPlugin)
            .add_plugins(PostProcessPlugin)
            .add_plugins(MapViewerPlugin {
                starting_location: Coord::new(52.1951, 0.1313),
                starting_zoom: 14,
                tile_quality: 256.0,
                cache_dir: "cache".to_string(),
                starting_url: None,
            })
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (handle_pancam, sync_cameras));
    }
}

#[derive(Component)]
pub struct DrawCamera;

fn setup_camera(mut commands: Commands, res_manager: Option<Res<TileMapResources>>) {
    if let Some(res_manager) = res_manager {
        let starting = res_manager
            .location_manager
            .location
            .to_game_coords(res_manager.clone());

        commands.spawn((
            Camera2d,
            DrawCamera,
            RenderLayers::from_layers(&[1]),
            Camera {
                order: 1,
                ..default()
            },
            Transform {
                translation: Vec3::new(starting.x, starting.y, 1.0),
                ..Default::default()
            },
        ));
        commands.spawn((
            Camera2d,
            MapViewerMarker,
            PostProcessSettings { ..default() },
            RenderLayers::from_layers(&[0]),
            Camera {
                order: 0,
                ..default()
            },
            Transform {
                translation: Vec3::new(starting.x, starting.y, 0.0),
                ..Default::default()
            },
            PanCam {
                grab_buttons: vec![MouseButton::Middle],
                move_keys: DirectionKeys {
                    up: vec![KeyCode::ArrowUp],
                    down: vec![KeyCode::ArrowDown],
                    left: vec![KeyCode::ArrowLeft],
                    right: vec![KeyCode::ArrowRight],
                },
                speed: 400.,
                enabled: true,
                zoom_to_cursor: true,
                min_scale: 0.01,
                max_scale: f32::INFINITY,
                min_x: f32::NEG_INFINITY,
                max_x: f32::INFINITY,
                min_y: f32::NEG_INFINITY,
                max_y: f32::INFINITY,
            },
        ));
    } else {
        error!("TileMapResources not found. Please add the tilemap addon first.");
    }
}
#[allow(clippy::type_complexity)]
fn sync_cameras(
    primary_query: Query<(&Transform, &Projection), With<MapViewerMarker>>,
    mut secondary_query: Query<
        (&mut Transform, &mut Projection),
        (With<DrawCamera>, Without<MapViewerMarker>),
    >,
) {
    if let Ok((primary_transform, primary_projection)) = primary_query.single() {
        if let Ok((mut secondary_transform, mut secondary_projection)) =
            secondary_query.single_mut()
        {
            secondary_transform.translation.x = primary_transform.translation.x;
            secondary_transform.translation.y = primary_transform.translation.y;
            secondary_transform.scale = primary_transform.scale;
            let primary_projection = match primary_projection {
                Projection::Orthographic(projection) => projection,
                _ => panic!("Projection is not orthographic"),
            };
            let secondary_projection = match &mut *secondary_projection {
                Projection::Orthographic(projection) => projection,
                _ => panic!("Projection is not orthographic"),
            };
            secondary_projection.scale = primary_projection.scale;
            secondary_projection.area = primary_projection.area;
            secondary_projection.far = primary_projection.far;
            secondary_projection.near = primary_projection.near;
        }
    }
}

fn handle_pancam(mut query: Query<&mut PanCam>, state: Res<EguiBlockInputState>) {
    if state.is_changed() {
        for mut pancam in &mut query {
            pancam.enabled = !state.block_input;
        }
    }
}

struct PostProcessPlugin;

impl Plugin for PostProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<PostProcessSettings>::default(),
            UniformComponentPlugin::<PostProcessSettings>::default(),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_graph_node::<ViewNodeRunner<PostProcessNode>>(Core2d, PostProcessLabel)
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::Tonemapping,
                    PostProcessLabel,
                    Node2d::EndMainPassPostProcessing,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<PostProcessPipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PostProcessLabel;

#[derive(Default)]
struct PostProcessNode;

impl ViewNode for PostProcessNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static PostProcessSettings,
        &'static DynamicUniformIndex<PostProcessSettings>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, _post_process_settings, settings_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let post_process_pipeline = world.resource::<PostProcessPipeline>();

        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<PostProcessSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "post_process_bind_group",
            &post_process_pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &post_process_pipeline.sampler,
                settings_binding.clone(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct PostProcessPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for PostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "post_process_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    uniform_buffer::<PostProcessSettings>(true),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world.load_asset(SHADER_ASSET_PATH);

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("post_process_pipeline".into()),
                    layout: vec![layout.clone()],
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader,
                        shader_defs: vec![],
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::bevy_default(),
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                    push_constant_ranges: vec![],
                    zero_initialize_workgroup_memory: false,
                });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub struct PostProcessSettings {
    pub on: u32,
}
