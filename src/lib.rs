use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;
use nalgebra::{Rotation3, Vector3, SVector};
use console_error_panic_hook;

use gloo::console::log;

use std::cell::RefCell;
use std::rc::Rc;

use crate::dom::{request_animation_frame, document};
use crate::webgl_utils::{compile_shader, clear, link_program};

mod dom;
mod webgl_utils;

const NUM_POINTS: usize = 5;
const NUM_VERTICES: usize = NUM_POINTS * 3;

fn draw_arrow(context: &WebGl2RenderingContext, angle: f32) {

    let mut vertex_list : [SVector::<f32, 3>; NUM_POINTS] = [
        Vector3::new(0.0, 0.0, 0.0), 
        Vector3::new(0.0, 3.0 / 3.0, 0.0), 
        Vector3::new(-1.0 / 6.0, 2.0 / 3.0, 0.0), 
        Vector3::new(0.0, 3.0 / 3.0, 0.0), 
        Vector3::new(1.0 / 6.0, 2.0 / 3.0, 0.0),
    ];

    let mut vertices: [f32; NUM_VERTICES] = [0.0; NUM_VERTICES];
    let rotator = Rotation3::from_euler_angles(0.0, 0.0, angle);

    for i in 0..vertex_list.len() {
        vertex_list[i] = rotator * vertex_list[i];
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

    context.draw_arrays(WebGl2RenderingContext::LINE_STRIP, 0, NUM_POINTS as i32);
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let document = document();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es
 
        in vec4 position;

        void main() {
            gl_Position = position;
        }
        "##,
    )?;

    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es
    
        precision highp float;
        out vec4 outColor;
        
        void main() {
            outColor = vec4(1, 1, 1, 1);
        }
        "##,
    )?;

    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    let position_attribute_location = context.get_attrib_location(&program, "position");
    let buffer = context.create_buffer().ok_or("Failed to create buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

    const NUM_POINTS: usize = 5;
    const NUM_VERTICES: usize = NUM_POINTS * 3;

    let mut vertex_list : [SVector::<f32, 3>; NUM_POINTS] = [
        Vector3::new(0.0, 0.0, 0.0), 
        Vector3::new(0.0, 3.0 / 3.0, 0.0), 
        Vector3::new(-1.0 / 6.0, 2.0 / 3.0, 0.0), 
        Vector3::new(0.0, 3.0 / 3.0, 0.0), 
        Vector3::new(1.0 / 6.0, 2.0 / 3.0, 0.0),
    ];


    let mut vertices: [f32; NUM_VERTICES] = [0.0; NUM_VERTICES];
    let rotator = Rotation3::from_euler_angles(0.0, 0.0, 0.0);

    for i in 0..vertex_list.len() {
        vertex_list[i] = rotator * vertex_list[i];
        vertices[i] = vertex_list[i][0];
        vertices[i + 1] = vertex_list[i][1];
        vertices[i + 2] = vertex_list[i][2];
    }

    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(&vertices);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }

    let vertex_array_object = context
        .create_vertex_array()
        .ok_or("Could not create vertex array object")?;
    context.bind_vertex_array(Some(&vertex_array_object));
    context.vertex_attrib_pointer_with_i32(
        position_attribute_location as u32,
        3,
        WebGl2RenderingContext::FLOAT,
        true,
        0,
        0,
    );
    context.enable_vertex_attrib_array(position_attribute_location as u32);
    context.bind_vertex_array(Some(&vertex_array_object));

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
let mut rotation = 0.01; *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // This is the render loop

        clear(&context);

        draw_arrow(&context, rotation);
        rotation -= 0.01;

        draw_arrow(&context, 1.57 + rotation);

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
