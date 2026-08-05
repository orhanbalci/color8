//! Renders `color_from_palette16()` with `ColorBlend::LinearBlend` — 16
//! hand-picked anchor colors turned into one continuous, smooth ramp.

#[path = "support/mod.rs"]
mod support;

use color8::{ColorBlend, Crgb, CrgbPalette16, color_from_palette16};
use support::Canvas;

const SAMPLES: usize = 600;

fn main() {
    // "Dusk" palette: night, through sunset, and back to night.
    let palette = CrgbPalette16::new([
        Crgb::new(11, 12, 33),
        Crgb::new(43, 20, 82),
        Crgb::new(92, 27, 110),
        Crgb::new(154, 33, 105),
        Crgb::new(214, 51, 82),
        Crgb::new(255, 93, 58),
        Crgb::new(255, 150, 60),
        Crgb::new(255, 205, 86),
        Crgb::new(255, 205, 86),
        Crgb::new(255, 150, 60),
        Crgb::new(255, 93, 58),
        Crgb::new(214, 51, 82),
        Crgb::new(154, 33, 105),
        Crgb::new(92, 27, 110),
        Crgb::new(43, 20, 82),
        Crgb::new(11, 12, 33),
    ]);

    let colors: Vec<Crgb> = (0..SAMPLES)
        .map(|i| {
            let index = ((i * 255) / (SAMPLES - 1)) as u8;
            color_from_palette16(&palette, index, 255, ColorBlend::LinearBlend)
        })
        .collect();

    let mut canvas = Canvas::new(
        "PALETTE INTERPOLATION",
        "COLOR_FROM_PALETTE16 - 16 COLORS, SMOOTH RAMP",
    );
    let (x, y, w, h) = canvas.content_rect();
    canvas.draw_strip(&colors, x, y, w, h);
    canvas.save("palette_blend");
}
