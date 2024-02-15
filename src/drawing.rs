use web_sys::WebGl2RenderingContext;
use nalgebra::{Rotation3, Vector3, Matrix3};

pub fn draw_line(context: &WebGl2RenderingContext, start: Vector3<f32>, end: Vector3<f32>) {
    const NUM_VERTICES: usize = 2; 

    let vertices: Vec<f32> = vec!(start.x, start.y, start.z, end.x, end.y, end.z);

    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(vertices.as_slice());

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    context.draw_arrays(WebGl2RenderingContext::LINE_STRIP, 0, NUM_VERTICES as i32);
}

pub fn draw_line_strip(context: &WebGl2RenderingContext, data: &[Vector3<f32>]) {
    let num_vertices = data.len(); 

    let mut vertices: Vec<f32> = Vec::new();

    for i in 0..num_vertices * 3 {
        vertices.push(data[i / 3][i % 3]);
    }

    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(vertices.as_slice());

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    context.draw_arrays(WebGl2RenderingContext::LINE_STRIP, 0, num_vertices as i32);
}

pub fn draw_square(context: &WebGl2RenderingContext, angle: &Vector3<f32>, translator: &Vector3<f32>, scaler: &Matrix3<f32>) {
    const NUM_VERTICES: usize = 6;
    const NUM_FLOATS: usize = NUM_VERTICES * 3;

    let mut vertex_list : [Vector3::<f32>; NUM_VERTICES] = [
        Vector3::new(0.0, 0.0, 0.0), 
        Vector3::new(0.0, 1.0, 0.0), 
        Vector3::new(1.0, 1.0, 0.0), 
        Vector3::new(0.0, 0.0, 0.0), 
        Vector3::new(1.0, 0.0, 0.0), 
        Vector3::new(1.0, 1.0, 0.0), 
    ];

    let mut vertices: [f32; NUM_FLOATS] = [0.0; NUM_FLOATS];

    let rotator = Rotation3::from_euler_angles(angle[0], angle[1], angle[2]);

    for i in 0..vertex_list.len() {
        vertex_list[i] = rotator * (scaler * vertex_list[i]) + translator;
    }

    for i in 0..vertices.len() {
        vertices[i] = vertex_list[i / 3][i % 3];
    }

    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(&vertices);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }

    context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, NUM_VERTICES as i32);
}

pub fn draw_arrow(context: &WebGl2RenderingContext, angle: &Vector3<f32>, translator: &Vector3<f32>, scaler: &Matrix3<f32>) {
    const NUM_VERTICES: usize = 9;
    const NUM_FLOATS: usize = NUM_VERTICES * 3;

    let mut vertex_list : [Vector3::<f32>; NUM_VERTICES] = [
        Vector3::new(0.0, 0.0, 0.0), 
        Vector3::new(0.0, 3.0 / 3.0, 0.0), 
        Vector3::new(-1.0 / 6.0, 2.0 / 3.0, 0.0), 
        Vector3::new(0.0, 3.0 / 3.0, 0.0), 
        Vector3::new(1.0 / 6.0, 2.0 / 3.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0), 
        Vector3::new(0.0, 2.0 / 3.0, -1.0 / 6.0), 
        Vector3::new(0.0, 1.0, 0.0), 
        Vector3::new(0.0, 2.0 / 3.0, 1.0 / 6.0), 
    ];

    let mut vertices: [f32; NUM_FLOATS] = [0.0; NUM_FLOATS];

    let rotator = Rotation3::from_euler_angles(angle[0], angle[1], angle[2]);

    for i in 0..vertex_list.len() {
        vertex_list[i] = rotator * (scaler * vertex_list[i]) + translator;
    }

    for i in 0..vertices.len() {
        vertices[i] = vertex_list[i / 3][i % 3];
    }

    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(&vertices);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }

    context.draw_arrays(WebGl2RenderingContext::LINE_STRIP, 0, NUM_VERTICES as i32);
}

pub fn draw_arrow_points(context: &WebGl2RenderingContext, start: &Vector3<f32>, end: &Vector3<f32>, clip: f32) {
    const NUM_VERTICES: usize = 9;
    const NUM_FLOATS: usize = NUM_VERTICES * 3;

    let mut vertex_list : [Vector3::<f32>; NUM_VERTICES] = [
        Vector3::new(0.0, 0.0, 0.0), 
        Vector3::new(0.0, 1.0, 0.0), 
        Vector3::new(-1.0 / 6.0, 2.0 / 3.0, 0.0), 
        Vector3::new(0.0, 1.0, 0.0), 
        Vector3::new(1.0 / 6.0, 2.0 / 3.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0), 
        Vector3::new(0.0, 2.0 / 3.0, -1.0 / 6.0), 
        Vector3::new(0.0, 1.0, 0.0), 
        Vector3::new(0.0, 2.0 / 3.0, 1.0 / 6.0), 
    ];

    let mut vertices: [f32; NUM_FLOATS] = [0.0; NUM_FLOATS];

    let rotator = Rotation3::rotation_between(start, end).unwrap();
    let mut magnitude = (end - start).magnitude();

    if magnitude > clip {
        magnitude = clip;
    }

    let scaler = magnitude * Matrix3::identity();
    for i in 0..vertex_list.len() {
        vertex_list[i] = rotator * (scaler * vertex_list[i]) + start;
    }

    for i in 0..vertices.len() {
        vertices[i] = vertex_list[i / 3][i % 3];
    }

    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(&vertices);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }

    context.draw_arrays(WebGl2RenderingContext::LINE_STRIP, 0, NUM_VERTICES as i32);
}
