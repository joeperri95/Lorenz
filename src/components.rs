use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

pub trait Renderable {
    fn render(&self, ctx: &CanvasRenderingContext2d) -> Result<(), JsValue>;
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Cursor {
    x: f64,
    y: f64,
    angle: f64,
    colour: &'static str 
}

impl Cursor {
    pub fn new(x: f64, y: f64, angle: f64, colour: &'static str) -> Self {
        Cursor {
            x, y, angle, colour
        }
    }

    pub fn set_rotation(&mut self, angle: f64) {
        self.angle = angle; 
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    pub fn displace(&mut self, distance: f64) {
        self.x = self.x + distance * self.angle.cos(); 
        self.y = self.y + distance * self.angle.sin(); 
    }
    
    pub fn rotate(&mut self, angle: f64) {
        self.angle = self.angle + angle; 
    }

    pub fn render(&self, ctx: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        let stroke_style: JsValue = JsValue::from_str("black");
        let fill_style: JsValue = JsValue::from_str(self.colour);
        ctx.save();
        ctx.translate(self.x, self.y)?;
        ctx.rotate(self.angle - std::f64::consts::FRAC_PI_2)?;
        ctx.begin_path();
        ctx.set_fill_style(&fill_style);
        ctx.set_stroke_style(&stroke_style);
        ctx.move_to(0.0, 20.0);
        ctx.line_to(-10.0, -5.0);
        ctx.line_to(0.0,  0.0);
        ctx.line_to(10.0, -5.0);
        ctx.line_to(0.0, 20.0);
        ctx.stroke();
        ctx.fill();
        ctx.restore();
        Ok(())
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Dot {
    x: f64,
    y: f64,
    radius: f64,
    colour: &'static str 
}

impl Dot {
    pub fn new(x: f64, y: f64, radius: f64, colour: &'static str) -> Self {
        Dot {
            x, y, radius, colour
        }
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }
    
    pub fn render(&self, ctx: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        let stroke_style: JsValue = JsValue::from_str("black");
        let fill_style: JsValue = JsValue::from_str(self.colour);
        ctx.save();
        ctx.begin_path();
        ctx.set_fill_style(&fill_style);
        ctx.set_stroke_style(&stroke_style);
        ctx.arc(self.x, self.y, self.radius, 0.0, 2.0 * std::f64::consts::PI).unwrap();
        ctx.fill();
        ctx.stroke();
        ctx.restore();
        Ok(())
    }
}
