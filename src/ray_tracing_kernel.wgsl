@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;

struct Sphere {
    center: vec3<f32>,
    radius: f32,
}

struct Ray {
    direction: vec3<f32>,
    origin: vec3<f32>,
}

fn length_squared(v: vec3<f32>) -> f32 {
    return v.x*v.x + v.y*v.y + v.z*v.z;
}

@compute @workgroup_size(1,1,1)
fn main(@builtin(global_invocation_id) GlobalInvocationID : vec3<u32>) {

    let screen_size: vec2<u32> = textureDimensions(color_buffer);
    let screen_pos : vec2<i32> = vec2<i32>(i32(GlobalInvocationID.x), i32(GlobalInvocationID.y));

    let aspect_ratio = f32(screen_size.x) / f32(screen_size.y);
    let viewport_height = f32(screen_size.y) / 500.0;
    let viewport_width = aspect_ratio * viewport_height;
    let focal_length = 1.0;

    let origin = vec3<f32>(0.0, 0.0, 0.0);
    let horizontal = vec3<f32>(viewport_width, 0.0, 0.0);
    let vertical = vec3<f32>(0.0, viewport_height, 0.0);
    let upper_left_corner = origin - horizontal/2.0 + vertical/2.0 - vec3<f32>(0.0, 0.0, focal_length);


    let u = f32(screen_pos.x) / (f32(screen_size.x) - 1.0);
    let v = f32(screen_pos.y) / (f32(screen_size.y) - 1.0);
    var ray: Ray;
    ray.origin = origin;
    ray.direction = normalize(upper_left_corner + u*horizontal - v*vertical - origin);


    var pixel_color: vec3<f32> = ray_color(ray);
    //let num = f32(screen_pos.x) / f32(screen_size.x);
    //var pixel_color: vec3<f32> = vec3<f32>(num, num, num);

    textureStore(color_buffer, screen_pos, vec4<f32>(pixel_color, 1.0));
}

fn ray_color(ray: Ray) -> vec3<f32> {
    var mySphere: Sphere;
    mySphere.center = vec3<f32>(0.0, 0.0, -1.0);
    mySphere.radius = 0.5;
    
    var pixel_color: vec3<f32>;


    let t = hit(ray, mySphere);
    if (t > 0.0) {
        let N: vec3<f32> = normalize((ray.origin + normalize(ray.direction) * t) - vec3<f32>(0.0, 0.0, -1.0));
        pixel_color = 0.5 * vec3<f32>(N.x+1.0, N.y+1.0, N.z+1.0);
    }
    else {
        let t = 0.5 * (ray.direction.y + 1.0);
        pixel_color = (1.0 - t) * vec3<f32>(1.0, 1.0, 1.0) + t * vec3<f32>(0.5, 0.7, 1.0);
    }

    return pixel_color;
}

fn hit(ray: Ray, sphere: Sphere) -> f32 {
    let oc = ray.origin - sphere.center;
    let a: f32 = length_squared(ray.direction);
    let half_b: f32 = dot(ray.direction, oc);
    let c: f32 = length_squared(oc) - sphere.radius * sphere.radius;
    let discriminant: f32 = half_b*half_b - a * c;

    if discriminant < 0.0 {
        return -1.0;
    } else {
        return (-half_b - sqrt(discriminant) ) / a;
    }
}

