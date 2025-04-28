#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct PostProcessSettings {
    intensity: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl2_padding: vec3<f32>
#endif
}
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // Compute distance from screen center (0.5, 0.5)
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(uv, center);

    // Make the offset strength grow toward the edges
    let strength = settings.intensity * dist;

    
    // Apply clamped UVs to avoid sampling outside [0,1]
    let uv_r = clamp(uv + vec2<f32>( strength, -strength), vec2<f32>(0.0), vec2<f32>(1.0));
    let uv_g = clamp(uv + vec2<f32>(-strength,  0.0), vec2<f32>(0.0), vec2<f32>(1.0));
    let uv_b = clamp(uv + vec2<f32>( 0.0,  strength), vec2<f32>(0.0), vec2<f32>(1.0));
    let color_r = textureSample(screen_texture, texture_sampler, uv_r).r;
    let color_g = textureSample(screen_texture, texture_sampler, uv_g).g;
    let color_b = textureSample(screen_texture, texture_sampler, uv_b).b;

    let darkening_factor = 1.0 - dist; // Closer to the center = brighter, further = darker
    let final_color = vec3<f32>(color_r, color_g, color_b) * darkening_factor;

    return vec4<f32>(final_color, 1.0);
}
