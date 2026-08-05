//! Renders `fill_rainbow_circular()` around a ring — one full turn of the
//! color wheel spread evenly over N pixels, the shape most LED rings
//! actually are.

#[path = "support/mod.rs"]
mod support;

use color8::{Crgb, fill_rainbow_circular};
use support::Canvas;

const N: usize = 60;

fn main() {
    let mut colors = [Crgb::default(); N];
    fill_rainbow_circular(&mut colors, 0, false);

    let mut canvas = Canvas::new("RAINBOW WHEEL", "FILL_RAINBOW_CIRCULAR() AROUND A RING");
    let (x, y, w, h) = canvas.content_rect();
    let cx = (x + w / 2) as i64;
    let cy = (y + h / 2) as i64;
    let radius = (h.min(w) / 2).saturating_sub(24) as f32;
    canvas.draw_ring(&colors, cx, cy, radius, 32);
    canvas.save("rainbow_wheel");
}
