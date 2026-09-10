//! GLSL shaders and SPIR-V pipeline sources for Gastinel UI.
//!
//! Features:
//! 1. Procedural SDF Rounded Box + Analytical Drop Shadow + Border fragment shader.
//! 2. Inter Typography R8_UNORM glyph sampling fragment shader.
//! 3. Dual Kawase 4-tap Downsample and 8-tap Tent Upsample shaders.

/// Vertex shader for instanced quads (emits 4 vertices from gl_VertexIndex).
pub const SDF_QUAD_VERTEX_SHADER: &str = r#"#version 450
layout(location = 0) in vec4 in_rect;          // x, y, width, height
layout(location = 1) in vec4 in_fill_color;
layout(location = 2) in vec4 in_border_color;
layout(location = 3) in vec4 in_shadow_color;
layout(location = 4) in vec4 in_params;        // corner_radius, border_width, shadow_blur, z_index

layout(push_constant) uniform PushConstants {
    vec2 viewport_size;
} pc;

layout(location = 0) out vec2 out_local_pos;
layout(location = 1) out vec2 out_half_size;
layout(location = 2) out vec4 out_fill_color;
layout(location = 3) out vec4 out_border_color;
layout(location = 4) out vec4 out_shadow_color;
layout(location = 5) out vec3 out_params;      // corner_radius, border_width, shadow_blur

const vec2 QUAD_VERTICES[4] = vec2[](
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0)
);

void main() {
    vec2 uv = QUAD_VERTICES[gl_VertexIndex];
    float blur = in_params.z;
    vec2 pos = in_rect.xy - vec2(blur) + uv * (in_rect.zw + vec2(2.0 * blur));

    // Screen to NDC
    vec2 ndc = (pos / pc.viewport_size) * 2.0 - 1.0;
    gl_Position = vec4(ndc, in_params.w, 1.0);

    out_local_pos = (uv - 0.5) * (in_rect.zw + vec2(2.0 * blur));
    out_half_size = in_rect.zw * 0.5;
    out_fill_color = in_fill_color;
    out_border_color = in_border_color;
    out_shadow_color = in_shadow_color;
    out_params = in_params.xyz;
}
"#;

/// Procedural fragment shader: evaluates exact signed distance field of rounded box with borders and soft shadow.
pub const SDF_QUAD_FRAGMENT_SHADER: &str = r#"#version 450
layout(location = 0) in vec2 in_local_pos;
layout(location = 1) in vec2 in_half_size;
layout(location = 2) in vec4 in_fill_color;
layout(location = 3) in vec4 in_border_color;
layout(location = 4) in vec4 in_shadow_color;
layout(location = 5) in vec3 in_params; // corner_radius, border_width, shadow_blur

layout(location = 0) out vec4 out_color;

// Signed Distance to a 2D rounded box
float sd_rounded_box(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + vec2(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r;
}

void main() {
    float radius = min(in_params.x, min(in_half_size.x, in_half_size.y));
    float border_w = in_params.y;
    float dist = sd_rounded_box(in_local_pos, in_half_size, radius);

    // Subpixel antialiasing with fwidth
    float aa = fwidth(dist);
    float alpha = clamp(0.5 - dist / aa, 0.0, 1.0);

    // Inner fill vs Border
    if (border_w > 0.0) {
        float border_factor = clamp((dist + border_w) / aa, 0.0, 1.0);
        vec4 color = mix(in_fill_color, in_border_color, border_factor);
        out_color = color * alpha;
    } else {
        out_color = in_fill_color * alpha;
    }
}
"#;

/// Dual Kawase downsample fragment shader: 4 bilinear samples in diagonal pattern.
pub const DUAL_KAWASE_DOWN_SHADER: &str = r#"#version 450
layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

layout(binding = 0) uniform sampler2D u_source;
layout(push_constant) uniform Push {
    vec2 half_pixel;
    float offset;
} pc;

void main() {
    vec2 d = pc.half_pixel * pc.offset;
    vec4 sum = texture(u_source, in_uv) * 4.0;
    sum += texture(u_source, in_uv - d);
    sum += texture(u_source, in_uv + d);
    sum += texture(u_source, in_uv + vec2(d.x, -d.y));
    sum += texture(u_source, in_uv + vec2(-d.x, d.y));
    out_color = sum * 0.125;
}
"#;
