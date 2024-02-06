use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;
use nalgebra::{Rotation3, Vector3, Matrix3, Matrix4, Orthographic3, Point3};
use console_error_panic_hook;

use gloo::events::{EventListenerOptions, EventListener};
use gloo::console::log;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use rand::random;

use crate::dom::{request_animation_frame, document, window};
use crate::webgl_utils::{compile_shader, clear, link_program};

mod dom;
mod webgl_utils;

fn draw_line_strip(context: &WebGl2RenderingContext, data: &[Vector3<f32>]) {
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

fn lorentz(state: Vector3<f32>, sigma: f32, rho: f32, beta: f32) -> Vector3<f32> {
   Vector3::new(
        sigma * (state.y - state.x), 
        state.x * (rho - state.z) - state.y,
        state.x * state.y - beta * state.z,
    )
}

fn draw_square(context: &WebGl2RenderingContext, angle: &Vector3<f32>, translator: &Vector3<f32>, scaler: &Matrix3<f32>) {
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

fn draw_arrow(context: &WebGl2RenderingContext, angle: &Vector3<f32>, translator: &Vector3<f32>, scaler: &Matrix3<f32>) {
    const NUM_VERTICES: usize = 5;
    const NUM_FLOATS: usize = NUM_VERTICES * 3;

    let mut vertex_list : [Vector3::<f32>; NUM_VERTICES] = [
        Vector3::new(0.0, 0.0, 0.0), 
        Vector3::new(0.0, 3.0 / 3.0, 0.0), 
        Vector3::new(-1.0 / 6.0, 2.0 / 3.0, 0.0), 
        Vector3::new(0.0, 3.0 / 3.0, 0.0), 
        Vector3::new(1.0 / 6.0, 2.0 / 3.0, 0.0),
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

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {

    console_error_panic_hook::set_once();
    let doc= document();
    let canvas = doc.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    
    let rho_slider = doc.get_element_by_id("rho-slider").unwrap();
    let rho_slider: web_sys::HtmlInputElement = rho_slider.dyn_into::<web_sys::HtmlInputElement>()?;

    let beta_slider = doc.get_element_by_id("beta-slider").unwrap();
    let beta_slider: web_sys::HtmlInputElement = beta_slider.dyn_into::<web_sys::HtmlInputElement>()?;

    let sigma_slider = doc.get_element_by_id("sigma-slider").unwrap();
    let sigma_slider: web_sys::HtmlInputElement = sigma_slider.dyn_into::<web_sys::HtmlInputElement>()?;

    let rho: Arc<RefCell<f32>> = RefCell::new(rho_slider.value().parse().unwrap()).into();
    let beta: Arc<RefCell<f32>> = RefCell::new(beta_slider.value().parse().unwrap()).into();
    let sigma: Arc<RefCell<f32>> = RefCell::new(beta_slider.value().parse().unwrap()).into();

    let rho_slider_event_listener_internal = rho.clone();
    let rho_slider_listener = EventListener::new_with_options(&rho_slider, "input", EventListenerOptions::enable_prevent_default(), move |_event| {
        let rho_slider = document().get_element_by_id("rho-slider").unwrap();
        let rho_slider: web_sys::HtmlInputElement = rho_slider.dyn_into::<web_sys::HtmlInputElement>().unwrap();
        log!("new rho value{:?}", rho_slider.value());
        *rho_slider_event_listener_internal.borrow_mut() = rho_slider.value().parse::<f32>().unwrap().into();
    });

    let beta_slider_event_listener_internal = beta.clone();
    let beta_slider_listener = EventListener::new_with_options(&beta_slider, "input", EventListenerOptions::enable_prevent_default(), move |_event| {
        let beta_slider = document().get_element_by_id("beta-slider").unwrap();
        let beta_slider: web_sys::HtmlInputElement = beta_slider.dyn_into::<web_sys::HtmlInputElement>().unwrap();
        log!("new beta value{:?}", beta_slider.value());
        *beta_slider_event_listener_internal.borrow_mut() = beta_slider.value().parse::<f32>().unwrap().into();
    });

    let sigma_slider_event_listener_internal = sigma.clone();
    let sigma_slider_listener = EventListener::new_with_options(&sigma_slider, "input", EventListenerOptions::enable_prevent_default(), move |_event| {
        let sigma_slider = document().get_element_by_id("sigma-slider").unwrap();
        let sigma_slider: web_sys::HtmlInputElement = sigma_slider.dyn_into::<web_sys::HtmlInputElement>().unwrap();
        log!("new sigma value{:?}", sigma_slider.value());
        *sigma_slider_event_listener_internal.borrow_mut() = sigma_slider.value().parse::<f32>().unwrap().into();
    });

    let camera_pos: Arc<RefCell<Vector3<f32>>> = RefCell::new(Vector3::new(-1000.0, -1000.0, 3.0)).into();
    let camera_front: Arc<RefCell<Vector3<f32>>> = RefCell::new(Vector3::new(0.0, 0.0, -1.0)).into();
    let camera_up: Arc<RefCell<Vector3<f32>>> = RefCell::new(Vector3::new(0.0, 1.0, 0.0)).into();

    let camera_front_mousemove_internal = camera_front.clone();
    let mut first_mouse = false;
    let mut last_x = 300;
    let mut last_y = 300;
    let mut pitch = 0.0;
    let mut yaw = 0.0;
    let mouse_listener = EventListener::new_with_options(&window(), "mousemove",  EventListenerOptions::enable_prevent_default(), move |event| {
        let event = event.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
        if first_mouse {
            first_mouse = false;
            last_x = event.x();
            last_y = event.y();
        }

        let x_offset = (event.x() - last_x) as f32;
        let y_offset = (event.y() - last_y) as f32;

        last_x = event.x();
        last_y = event.y();

        const SENSITIVITY: f32 = 0.1;
        yaw += x_offset * SENSITIVITY;
        pitch += y_offset * SENSITIVITY;

        if pitch > 89.0 {
            pitch = 89.0;
        }
        if pitch < -89.0 {
            pitch = -89.0;
        }

        gloo::console::log!(pitch, yaw);
        *camera_front_mousemove_internal.borrow_mut() = Vector3::new(f32::cos(yaw.to_radians()) * f32::cos(pitch.to_radians()), 
                                                                     f32::sin(pitch.to_radians()),
                                                                     f32::sin(yaw.to_radians()) * f32::sin(pitch.to_radians())).normalize().into();
    });

    let camera_pos_keydown_internal = camera_pos.clone();
    let camera_front_keydown_internal = camera_front.clone();
    let camera_up_keydown_internal = camera_up.clone();

    let keydown_listener = EventListener::new_with_options(&window(), "keydown", EventListenerOptions::enable_prevent_default(), move |event| {
        const CAMERA_SPEED: f32 = 1.2;
        let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
        let key = event.key();
        let mut camera_pos = camera_pos_keydown_internal.borrow_mut();
        let camera_front = *camera_front_keydown_internal.borrow();
        let camera_up = *camera_up_keydown_internal.borrow();
        match key.as_str() {
            "w" => {
                *camera_pos = (*camera_pos + CAMERA_SPEED * camera_front).into(); 
            },
            "s" => {
                *camera_pos = (*camera_pos - CAMERA_SPEED * camera_front).into(); 
            },
            "a" => {
                *camera_pos = (*camera_pos - CAMERA_SPEED * camera_front.cross(&camera_up)).into(); 
            },
            "d" => {
                *camera_pos = (*camera_pos + CAMERA_SPEED * camera_front.cross(&camera_up)).into(); 
            },
            _ => {log!("test down {:?}", key)}
        }
    });

    let keyup_listener = EventListener::new_with_options(&window(), "keyup", EventListenerOptions::enable_prevent_default(), move |event| {
        let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
        let key = event.key();
        match key.as_str() {
            _ => {log!("test up {:?}", key)}
        }
    });

    let context = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es
 
        in vec4 position;
        uniform mat4 uMVP;

        void main() {
            gl_Position = uMVP * position;
            gl_PointSize = 100.0;
        }
        "##,
    )?;

    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es
    
        precision highp float;
        out vec4 outColor;
        uniform vec4 uColour;
        
        void main() {
            outColor = uColour;
        }
        "##,
    )?;

    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    let colour_uniform_location = context.get_uniform_location(&program, "uColour");
    let modelviewprojection_uniform_location = context.get_uniform_location(&program, "uMVP");
    let position_attribute_location = context.get_attrib_location(&program, "position");
    let buffer = context.create_buffer().ok_or("Failed to create buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

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
    let mut rotation = 0.01; 

    const SCALER: f32 = 2.0;
    let scaler = Matrix3::new(
        SCALER, 0.0, 0.0,
        0.0, SCALER, 0.0,
        0.0, 0.0, SCALER,
    );

    const DELTA_T: f32 = 0.02;

    // Create the initial points
    const MAX_POINTS: usize = 5000;
    const NUM_TRAJECTORIES: usize = 10;
    let mut data: [Vec<Vector3<f32>>; NUM_TRAJECTORIES] = Default::default(); 
    let mut colours: [Vector3<f32>; NUM_TRAJECTORIES] = Default::default(); 

    data[0].push(Vector3::new(2.0, 1.0, 1.0));
    for i in 1..NUM_TRAJECTORIES {
        data[i].push(Vector3::new(10.0 * random::<f32>(), 10.0 * random::<f32>(), 10.0 * random::<f32>()));
        colours[i] = Vector3::new(random(), random(), random());
    }

    const BOUNDS: f32 = 100.0;

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // This is the render loop
        
        // Camera related
        let camx =  f32::sin(rotation);
        let camz =  f32::cos(rotation);
        // let camx = 0.0;
        // let camz = -1.0;

        let proj = Orthographic3::new(-BOUNDS, BOUNDS, -BOUNDS, BOUNDS, -BOUNDS, BOUNDS);
        // let proj = Matrix4::new_perspective(1.0, 100.0, 0.1, 1000.0);
        let camera_p = camera_pos.borrow();
        let camera_f = camera_front.borrow();
        let up_vector = *camera_p + *camera_f; 
        let view = Matrix4::look_at_rh(&Point3::new(camx, 0.0, camz), &Point3::new(0.0, 0.0, 0.0), &Vector3::new(0.0, 1.0, 0.0));
        /*
        let view = Matrix4::look_at_rh(&Point3::new(camera_p.x, camera_p.y, camera_p.z), 
                                       &Point3::new(up_vector.x, up_vector.y, up_vector.z), 
                                       &*camera_up.borrow());
                                       */
        let model = Matrix4::identity();
        let mvp = proj.as_matrix() * view * model;
        //let mvp = proj * view * model;

        context.uniform_matrix4fv_with_f32_array(modelviewprojection_uniform_location.as_ref(), false, 
                                                 mvp.as_slice()
                                                 );

        // Update the position of the points
        for i in 0..NUM_TRAJECTORIES {
            let new_state = data[i].last().unwrap() + lorentz(*data[i].last().unwrap(), *sigma.borrow(), *rho.borrow(), *beta.borrow()) * DELTA_T;
            data[i].push(new_state);
            if data[i].len() > MAX_POINTS {
                data[i].remove(0);
            }
        }

        clear(&context);

        context.uniform4f(colour_uniform_location.as_ref(), 1.0, 1.0, 1.0, 1.0);
        const STEP_SIZE: f32 = 20.0;
        for i in -5..6 {
            for j in -5..6 {
                // draw_arrow(&context, &Vector3::new(0.0, 0.0, rotation), &Vector3::new(i as f32 * STEP_SIZE, j as f32 * STEP_SIZE, 0.0), &scaler);
            }
        }

        for i in 0..NUM_TRAJECTORIES {
            context.uniform4f(colour_uniform_location.as_ref(), colours[i].x, colours[i].y, colours[i].z , 1.0);
            draw_line_strip(&context, data[i].as_slice());
            draw_square(&context, &Vector3::zeros(), &data[i][0], &scaler);
        }

        rotation -= 0.01;

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    keydown_listener.forget();
    keyup_listener.forget();
    rho_slider_listener.forget();
    beta_slider_listener.forget();
    sigma_slider_listener.forget();
    mouse_listener.forget();

    Ok(())
}
