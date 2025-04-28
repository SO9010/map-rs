#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

@group(1) @binding(1)
var texture: texture_2d<f32>;

struct PostProcessSettings {
    on: u32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl2_padding: vec3<f32>
#endif
}
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    if settings.on == 1 {
        let uv = in.uv;
        // Compute distance from screen center (0.5, 0.5)
        let center = vec2<f32>(0.5, 0.5);
        let dist = distance(uv, center);

        // Apply a non-linear scaling to make the edges darker more quickly
        let darkening_factor = 1 - (pow(dist, 1.2) * 1.2); // Squared distance for faster darkening
        let darkening = clamp(darkening_factor, 0.1, 1.0); // Ensure it stays within [0, 1]

        // Sample the original color
        let color = textureSample(screen_texture, texture_sampler, uv).rgb;

        // Apply the darkening factor
        let final_color = color * darkening;

        return vec4<f32>(final_color, 1.0); // Alpha is set to 1.0
    } else {
        return textureSample(screen_texture, texture_sampler, in.uv);
    }
}
