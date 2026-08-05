//! Renders `heat_color()` across the entire `u8` range as a single smooth
//! strip — FastLED's black -> red -> yellow -> white fire ramp.

#[path = "support/mod.rs"]
mod support;

use color8::heat_color;
use support::Canvas;

fn main() {
    let colors: Vec<_> = (0..=255u16).map(|t| heat_color(t as u8)).collect();

    let mut canvas = Canvas::new("FIRE GRADIENT", "HEAT_COLOR() OVER THE FULL U8 RANGE");
    let (x, y, w, h) = canvas.content_rect();
    canvas.draw_strip(&colors, x, y, w, h);
    canvas.save("heat_sweep");
}
