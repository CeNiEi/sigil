struct VertexInput {
    @location(0) position: vec2<f32>
}

@vertex
fn vs_main(
    input: VertexInput,
) -> @builtin(position) vec4<f32> {
    return vec4<f32>(input.position, 0.0, 1.0);
}

struct Sine {
    center: vec2<f32>,
    inner_radius: f32,
    outer_radius: f32,
    amplitude: f32,
    cycles: f32,
    speed: f32,
}

struct Global {
    resolution: vec2<f32>,
    phase: f32
}

@group(0) @binding(0)
var<uniform> global: Global;

@group(1) @binding(0)
var<uniform> sine: Sine;


@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = frag_coord.xy / global.resolution;

    let centered = uv - sine.center;
    let aspect = global.resolution.x / global.resolution.y;
    let pos = vec2<f32>(centered.x * aspect, centered.y);

    // let amp = 0.05;
    // let num = 8.0;
    // let speed = 0.005;

    let theta = atan2(pos.y, pos.x);

    let first_phase = sine.cycles * (theta - sine.speed * global.phase);
    let first_outer_wave = sine.outer_radius + sine.amplitude * sin(first_phase);
    let first_inner_wave = sine.inner_radius + sine.amplitude * sin(first_phase);

    // let phase_shift = 3.14159265;
    // let second_phase = sine.cycles * (theta - sine.speed * global.phase) + phase_shift;
    // let second_outer_wave = sine.outer_radius + sine.amplitude * sin(second_phase);
    // let second_inner_wave = sine.inner_radius + sine.amplitude * sin(second_phase);

    let dist = length(pos);

    let hue = fract((theta / (2.0 * 3.14159)) + 0.5);

    let color = vec3<f32>(
        0.5 + 0.5 * sin(6.2831 * hue + 0.0),
        0.5 + 0.5 * sin(6.2831 * hue + 2.094),
        0.5 + 0.5 * sin(6.2831 * hue + 4.188)
    );

    if dist >= first_inner_wave && dist < first_outer_wave {
        return vec4<f32>(color, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}

