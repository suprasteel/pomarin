
// -------------
// Camera

struct CameraUniform {
    view_pos: vec4<f32>;
    view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> camera: CameraUniform;

// todo: do lights in view space
// https://sotrh.github.io/learn-wgpu/intermediate/tutorial10-lighting/#the-normal-matrix
struct Light {
    position: vec3<f32>;
    color: vec3<f32>;
};

[[group(1), binding(0)]]
var<uniform> light: Light;


// -------------
// Vertex shader

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] uv: vec3<f32>;
    [[location(2)]] normal: vec3<f32>;
    [[location(3)]] tan: vec3<f32>;
    [[location(4)]] bt: vec3<f32>;
};

struct InstanceInput {
    [[location(5)]] model_matrix_0: vec4<f32>;
    [[location(6)]] model_matrix_1: vec4<f32>;
    [[location(7)]] model_matrix_2: vec4<f32>;
    [[location(8)]] model_matrix_3: vec4<f32>;

    [[location(9)]] normal_matrix_0: vec3<f32>;
    [[location(10)]] normal_matrix_1: vec3<f32>;
    [[location(11)]] normal_matrix_2: vec3<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] world_position: vec3<f32>;
    [[location(1)]] world_normal: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput { 

    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );
    
    // 
    var out: VertexOutput;
    out.world_normal = normal_matrix * model.normal;
    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    return out;
}

// ---------------
// Fragment shader

struct MaterialColor {
    ambient: vec3<f32>;
    specular: f32;
    diffuse: vec3<f32>;
    _pad: u32;
};

[[group(2), binding(0)]]
var<uniform> material: MaterialColor; // materials to be used

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    let ambient_color = material.ambient * light.color;

    let light_dir = normalize(light.position - in.world_position);

    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = diffuse_strength * material.diffuse * light.color;

    let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    let reflect_dir = reflect(-light_dir, in.world_normal);

    let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let specular_color = specular_strength * material.specular * light.color;

    let result = (ambient_color + diffuse_color + specular_color);

    return vec4<f32>(result, 1.0);
}

