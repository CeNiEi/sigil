struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) center: vec2<f32>,
    @location(2) inner_radius: f32,
    @location(3) thickness: f32,
    @location(4) amplitude: f32,
    @location(5) cycles: f32,
    @location(6) speed: f32,
    @location(7) init: u32
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) center: vec2<f32>,
    @location(2) inner_radius: f32,
    @location(3) thickness: f32,
    @location(4) amplitude: f32,
    @location(5) cycles: f32,
    @location(6) speed: f32,
    @location(7) init: u32
}

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var output: VertexOutput;

    let clip_center = input.center * 2.0 - vec2<f32>(1.0, 1.0);
    output.position = vec4<f32>(input.position + clip_center, 0.0, 1.0);

    output.center = input.center;
    output.speed = input.speed;
    output.cycles = input.cycles;
    output.amplitude = input.amplitude;
    output.inner_radius = input.inner_radius;
    output.thickness = input.thickness;
    output.init = input.init;

    return output;
}

struct Global {
    resolution: vec2<f32>,
    phase: f32
}

@group(0) @binding(0)
var<uniform> global: Global;

@fragment
fn fs_main(
    vertex_output: VertexOutput
) -> @location(0) vec4<f32> {
    if vertex_output.init == 0u {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let frag_coord = vertex_output.position;

    let uv = frag_coord.xy / global.resolution;

    let centered = uv - vertex_output.center;
    let aspect = global.resolution.x / global.resolution.y;
    let pos = vec2<f32>(centered.x * aspect, centered.y);

    let theta = atan2(pos.y, pos.x);

    let phase = vertex_output.cycles * (theta - vertex_output.speed * global.phase);
    let inner_wave = vertex_output.inner_radius + vertex_output.amplitude * sin(phase);


    let dist = length(pos);

    let hue = fract((theta / (2.0 * 3.14159)) + 0.5);

    let color = vec3<f32>(
        0.5 + 0.5 * sin(6.2831 * hue + 0.0),
        0.5 + 0.5 * sin(6.2831 * hue + 2.094),
        0.5 + 0.5 * sin(6.2831 * hue + 4.188)
    );

    if dist >= inner_wave && dist < inner_wave + vertex_output.thickness {
        return vec4<f32>(color, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}

