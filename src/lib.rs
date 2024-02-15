use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use futures_util::{future::ready, stream::StreamExt};
use web_sys::WebGl2RenderingContext;
use nalgebra::{Vector3, Matrix4, Point3};
use console_error_panic_hook;

use gloo::events::{EventListenerOptions, EventListener};
use gloo::console::log;
use gloo::timers::future::IntervalStream;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use rand::random;

use crate::dom::{request_animation_frame, document, body, window};
use crate::webgl_utils::{compile_shader, clear, link_program};
use crate::drawing::{draw_line_strip, draw_line};

mod dom;
mod drawing;
mod webgl_utils;

// Constants
const SENSITIVITY: f32 = 0.1;
const BOUNDS: f32 = 10000.0;
const DELTA_T: f32 = 0.002;
const CAMERA_SPEED: f32 = 2.0;
const CAMERA_ROTATION: f32 = 0.5;
const MAX_POINTS: usize = 500;
const NUM_TRAJECTORIES: usize = 20;
const RANDOM_RANGE: f32 = 100.0;
const SPAWN_INTERVAL: u32 = 5_00;
const ORIGINAL_CAMERA_POS: Vector3<f32> = Vector3::new(0.0, 0.0, 500.0);

const VERTEX_SHADER_TEXT: &'static str = 
r##"#version 300 es

in vec4 position;
uniform mat4 uMVP;

void main() {
    gl_Position = uMVP * position;
    gl_PointSize = 100.0;
}
"##;

const FRAGMENT_SHADER_TEXT: &'static str =
r##"#version 300 es

precision highp float;
out vec4 outColor;
uniform vec4 uColour;

void main() {
    outColor = uColour;
}
"##;

fn lorentz(state: Vector3<f32>, sigma: f32, rho: f32, beta: f32) -> Vector3<f32> {
   Vector3::new(
        sigma * (state.y - state.x), 
        state.x * (rho - state.z) - state.y,
        state.x * state.y - beta * state.z,
    )
}

fn toggle_pause(paused: &Arc<RefCell<bool>>) {
    let pause_button = document().get_element_by_id("pause-button").unwrap();
    let pause_button: web_sys::HtmlButtonElement = pause_button.dyn_into::<web_sys::HtmlButtonElement>().unwrap();
    let paused_local = *paused.borrow();
    if paused_local {
        pause_button.set_inner_text("pause");
        *paused.borrow_mut() = false.into();
    } else {
        pause_button.set_inner_text("resume");
        *paused.borrow_mut() = true.into();
    }
}

fn spawn_random_trajectory(data: &mut Vec<Vec<Vector3<f32>>>, colours: &mut Vec<Vector3<f32>>) {
    let max = data.len();
    data.push(Vec::default());
    data[max].push(Vector3::new(RANDOM_RANGE * (random::<f32>() - 0.5), RANDOM_RANGE * (random::<f32>() - 0.5), RANDOM_RANGE * (random::<f32>() - 0.5)));
    colours.push(Vector3::new(random(), random(), random()));
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {

    console_error_panic_hook::set_once();
    let doc= document();
    let canvas = doc.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    
    // Event listener setup
    let pause_button = doc.get_element_by_id("pause-button").unwrap();
    let pause_button: web_sys::HtmlButtonElement = pause_button.dyn_into::<web_sys::HtmlButtonElement>()?;

    let rho_slider = doc.get_element_by_id("rho-slider").unwrap();
    let rho_slider: web_sys::HtmlInputElement = rho_slider.dyn_into::<web_sys::HtmlInputElement>()?;

    let beta_slider = doc.get_element_by_id("beta-slider").unwrap();
    let beta_slider: web_sys::HtmlInputElement = beta_slider.dyn_into::<web_sys::HtmlInputElement>()?;

    let sigma_slider = doc.get_element_by_id("sigma-slider").unwrap();
    let sigma_slider: web_sys::HtmlInputElement = sigma_slider.dyn_into::<web_sys::HtmlInputElement>()?;

    let paused: Arc<RefCell<bool>> = RefCell::new(false).into();
    let rho: Arc<RefCell<f32>> = RefCell::new(rho_slider.value().parse().unwrap()).into();
    let beta: Arc<RefCell<f32>> = RefCell::new(beta_slider.value().parse().unwrap()).into();
    let sigma: Arc<RefCell<f32>> = RefCell::new(beta_slider.value().parse().unwrap()).into();

    let paused_button_event_listener_internal = paused.clone();
    let pause_button_listener = EventListener::new_with_options(&pause_button, "click", EventListenerOptions::enable_prevent_default(), move |_event| {
        toggle_pause(&paused_button_event_listener_internal);
    });

    let rho_slider_event_listener_internal = rho.clone();
    let rho_slider_listener = EventListener::new_with_options(&rho_slider, "input", EventListenerOptions::enable_prevent_default(), move |_event| {
        let rho_slider = document().get_element_by_id("rho-slider").unwrap();
        let rho_slider: web_sys::HtmlInputElement = rho_slider.dyn_into::<web_sys::HtmlInputElement>().unwrap();
        let rho_slider_label = document().get_element_by_id("rho-slider-label").unwrap();
        let rho_slider_label: web_sys::HtmlLabelElement = rho_slider_label.dyn_into::<web_sys::HtmlLabelElement>().unwrap();
        log!("new rho value{:?}", rho_slider.value());
        *rho_slider_event_listener_internal.borrow_mut() = rho_slider.value().parse::<f32>().unwrap().into();
        rho_slider_label.set_inner_text(&format!("rho = {}", rho_slider.value()));
    });

    let beta_slider_event_listener_internal = beta.clone();
    let beta_slider_listener = EventListener::new_with_options(&beta_slider, "input", EventListenerOptions::enable_prevent_default(), move |_event| {
        let beta_slider = document().get_element_by_id("beta-slider").unwrap();
        let beta_slider: web_sys::HtmlInputElement = beta_slider.dyn_into::<web_sys::HtmlInputElement>().unwrap();
        let beta_slider_label = document().get_element_by_id("beta-slider-label").unwrap();
        let beta_slider_label: web_sys::HtmlLabelElement = beta_slider_label.dyn_into::<web_sys::HtmlLabelElement>().unwrap();
        log!("new beta value{:?}", beta_slider.value());
        *beta_slider_event_listener_internal.borrow_mut() = beta_slider.value().parse::<f32>().unwrap().into();
        beta_slider_label.set_inner_text(&format!("beta = {}", beta_slider.value()));
    });

    let sigma_slider_event_listener_internal = sigma.clone();
    let sigma_slider_listener = EventListener::new_with_options(&sigma_slider, "input", EventListenerOptions::enable_prevent_default(), move |_event| {
        let sigma_slider = document().get_element_by_id("sigma-slider").unwrap();
        let sigma_slider: web_sys::HtmlInputElement = sigma_slider.dyn_into::<web_sys::HtmlInputElement>().unwrap();
        let sigma_slider_label = document().get_element_by_id("sigma-slider-label").unwrap();
        let sigma_slider_label: web_sys::HtmlLabelElement = sigma_slider_label.dyn_into::<web_sys::HtmlLabelElement>().unwrap();
        log!("new sigma value{:?}", sigma_slider.value());
        *sigma_slider_event_listener_internal.borrow_mut() = sigma_slider.value().parse::<f32>().unwrap().into();
        sigma_slider_label.set_inner_text(&format!("sigma = {}", sigma_slider.value()));
    });

    let camera_pos: Arc<RefCell<Vector3<f32>>> = RefCell::new(ORIGINAL_CAMERA_POS).into();
    let camera_front: Arc<RefCell<Vector3<f32>>> = RefCell::new(Vector3::new(0.0, 0.0, -1.0)).into();
    let camera_up: Arc<RefCell<Vector3<f32>>> = RefCell::new(Vector3::new(0.0, 1.0, 0.0)).into();

    let mut pitch = 0.0;
    let mut yaw = -90.0;
    let camera_front_mousemove_internal = camera_front.clone();
    let mouse_move_listener = EventListener::new_with_options(&body(), "mousemove",  EventListenerOptions::enable_prevent_default(), move |event| {
        if let Some(_) = doc.pointer_lock_element() {
            let event = event.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();

            yaw -= event.movement_x() as f32 * SENSITIVITY;
            pitch += event.movement_y() as f32 * SENSITIVITY;

            if pitch > 89.0 {
                pitch = 89.0;
            } else if pitch < -89.0 {
                pitch = -89.0;
            }

            *camera_front_mousemove_internal.borrow_mut() = Vector3::new(f32::cos(yaw.to_radians()) * f32::cos(pitch.to_radians()), 
                                                                         f32::sin(pitch.to_radians()),
                                                                         f32::sin(yaw.to_radians()) * f32::cos(pitch.to_radians()))
                                                                         .into();
        }
    });

    let canvas_click_listener = EventListener::new_with_options(&canvas, "mousedown", EventListenerOptions::enable_prevent_default(), move |_event| {
        body().request_pointer_lock();
    });

    let camera_pos_keydown_internal = camera_pos.clone();
    let camera_front_keydown_internal = camera_front.clone();
    let camera_up_keydown_internal = camera_up.clone();

    let paused_keyboard_listener = paused.clone();
    let keydown_listener = EventListener::new_with_options(&window(), "keydown", EventListenerOptions::enable_prevent_default(), move |event| {
        let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
        let key = event.key();
        let mut camera_pos = camera_pos_keydown_internal.borrow_mut();
        let mut camera_front = camera_front_keydown_internal.borrow_mut();
        let camera_up = *camera_up_keydown_internal.borrow();
        match key.as_str() {
            "w" => {
                *camera_pos = (*camera_pos + CAMERA_SPEED * *camera_front).into(); 
            },
            "s" => {
                *camera_pos = (*camera_pos - CAMERA_SPEED * *camera_front).into(); 
            },
            "a" => {
                *camera_pos = (*camera_pos + CAMERA_SPEED * camera_front.cross(&camera_up)).into(); 
            },
            "d" => {
                *camera_pos = (*camera_pos - CAMERA_SPEED * camera_front.cross(&camera_up)).into(); 
            },
            "Shift" => {
                *camera_pos = (*camera_pos - CAMERA_SPEED * &camera_up).into(); 
            },
            "Control" => {
                *camera_pos = (*camera_pos + CAMERA_SPEED * &camera_up).into(); 
            },
            "ArrowUp" => {
                *camera_front = (*camera_front - Vector3::new(0.0, CAMERA_ROTATION.to_radians(), 0.0)).normalize();
            },
            "ArrowDown" => {
                *camera_front = (*camera_front + Vector3::new(0.0, CAMERA_ROTATION.to_radians(), 0.0)).normalize();
            },
            "ArrowRight" => {
                *camera_front = (*camera_front - Vector3::new(CAMERA_ROTATION.to_radians(), 0.0, 0.0)).normalize();
            },
            "ArrowLeft" => {
                *camera_front = (*camera_front + Vector3::new(CAMERA_ROTATION.to_radians(), 0.0, 0.0)).normalize();
            },
            " " => {
                toggle_pause(&paused_keyboard_listener);
            },
            _ => {log!("Unused key down", key)}
        }
    });

    // WebGL setup
    let context = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    let vert_shader = compile_shader(&context, WebGl2RenderingContext::VERTEX_SHADER, VERTEX_SHADER_TEXT)?;
    let frag_shader = compile_shader(&context, WebGl2RenderingContext::FRAGMENT_SHADER, FRAGMENT_SHADER_TEXT)?;
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

    // Create the initial points
    let data: Arc<RefCell<Vec<Vec<Vector3<f32>>>>> = Default::default();
    let colours: Arc<RefCell<Vec<Vector3<f32>>>> = Default::default(); 
    for _ in 0..NUM_TRAJECTORIES {
        spawn_random_trajectory(&mut data.borrow_mut(), &mut colours.borrow_mut());
    }

    let data_render_loop_internal = data.clone();
    let colours_render_loop_internal = colours.clone();
    // This is the render loop
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if ! *paused.borrow() {
            // Camera related
            let proj = Matrix4::new_perspective(1.0, 100.0, 0.1, 1000.0);
            let camera_p = camera_pos.borrow();
            let camera_f = camera_front.borrow();
            let target_vector = *camera_p + *camera_f; 
            let view = Matrix4::look_at_rh(&Point3::new(camera_p.x, camera_p.y, camera_p.z), 
                                           &Point3::new(target_vector.x, target_vector.y, target_vector.z), 
                                           &*camera_up.borrow());
            let model = Matrix4::identity();
            let mvp = proj * view * model;

            context.uniform_matrix4fv_with_f32_array(modelviewprojection_uniform_location.as_ref(), false, 
                                                     mvp.as_slice()
                                                     );

            let mut data_internal = data_render_loop_internal.borrow_mut();
            let colours_internal = colours_render_loop_internal.borrow();
            // Update the position of the points
            for i in 0..data_internal.len() {
                let new_state = data_internal[i].last().unwrap() + lorentz(*data_internal[i].last().unwrap(), *sigma.borrow(), *rho.borrow(), *beta.borrow()) * DELTA_T;
                data_internal[i].push(new_state);
                if data_internal[i].len() > MAX_POINTS {
                    data_internal[i].remove(0);
                }
            }

            clear(&context);

            // draw trajectories
            for i in 0..data_internal.len(){
                context.uniform4f(colour_uniform_location.as_ref(), colours_internal[i].x, colours_internal[i].y, colours_internal[i].z , 1.0);
                draw_line_strip(&context, data_internal[i].as_slice());
                //draw_square(&context, &Vector3::zeros(), &data_internal[i][data[i].len()-1], &(Matrix3::identity() * 2.0));
            }

            // draw axes
            context.uniform4f(colour_uniform_location.as_ref(), 1.0, 1.0, 1.0, 1.0);
            draw_line(&context, Vector3::new(-BOUNDS, 0.0, 0.0), Vector3::new(BOUNDS, 0.0, 0.0));
            draw_line(&context, Vector3::new(0.0, -BOUNDS, 0.0), Vector3::new(0.0, BOUNDS, 0.0));
            draw_line(&context, Vector3::new(0.0, 0.0, -BOUNDS), Vector3::new(0.0, 0.0, BOUNDS));
        }

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    let data_spawn_loop_internal = data.clone();
    let colours_spawn_loop_internal = colours.clone();
    spawn_local(async move {
        let interval = IntervalStream::new(SPAWN_INTERVAL);
        interval.for_each(|_| {
            spawn_random_trajectory(&mut data_spawn_loop_internal.borrow_mut(), &mut colours_spawn_loop_internal.borrow_mut());
            ready(())
        }).await;
    });

    request_animation_frame(g.borrow().as_ref().unwrap());

    // cleanup event listeners
    keydown_listener.forget();
    rho_slider_listener.forget();
    beta_slider_listener.forget();
    sigma_slider_listener.forget();
    mouse_move_listener.forget();
    canvas_click_listener.forget();
    pause_button_listener.forget();

    Ok(())
}
