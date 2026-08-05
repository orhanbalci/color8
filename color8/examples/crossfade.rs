//! Renders `blend()` at six increasing steps between two colors — a
//! filmstrip standing in for the smooth crossfade it computes at any
//! resolution.

#[path = "support/mod.rs"]
mod support;

use color8::{Crgb, blend};
use support::Canvas;

fn main() {
    let a = Crgb::new(46, 229, 224); // cyan
    let b = Crgb::new(255, 61, 148); // magenta

    let steps = [0u8, 51, 102, 153, 204, 255];
    let colors: Vec<Crgb> = steps.iter().map(|&amt| blend(a, b, amt)).collect();

    let mut canvas = Canvas::new(
        "COLOR BLEND",
        "BLEND() - SIX STEPS FROM ONE COLOR TO ANOTHER",
    );
    let (x, y, w, h) = canvas.content_rect();

    let n = colors.len() as u32;
    let gap = 16u32;
    let cell = ((w - (n - 1) * gap) / n).min(h);
    let total_w = n * cell + (n - 1) * gap;
    let start_x = x + (w - total_w) / 2;
    let start_y = y + (h - cell) / 2;

    canvas.draw_blocks(&colors, start_x, start_y, cell, gap);
    canvas.save("crossfade");
}
