//! Renders four of FastLED's built-in palettes stacked: three plain
//! `CrgbPalette16` presets, plus one parsed at runtime from a
//! gradient-palette byte stream via `rainbow_gradient_palette16()` —
//! demonstrating both `color8::presets` and `color8::gradient_palette` in
//! one image.

#[path = "support/mod.rs"]
mod support;

use color8::{
    ColorBlend, Crgb, CrgbPalette16, HEAT_COLORS, OCEAN_COLORS, PARTY_COLORS, color_from_palette16,
    rainbow_gradient_palette16,
};
use support::Canvas;

const SAMPLES: usize = 600;

fn ramp(pal: &CrgbPalette16, samples: usize) -> Vec<Crgb> {
    (0..samples)
        .map(|i| {
            let index = ((i * 255) / (samples - 1)) as u8;
            color_from_palette16(pal, index, 255, ColorBlend::LinearBlend)
        })
        .collect()
}

fn main() {
    let rainbow_gp = rainbow_gradient_palette16();

    let rows = [
        ("HEAT_COLORS", &HEAT_COLORS),
        ("OCEAN_COLORS", &OCEAN_COLORS),
        ("PARTY_COLORS", &PARTY_COLORS),
        ("RAINBOW-GP", &rainbow_gp),
    ];

    let mut canvas = Canvas::new(
        "PALETTE PRESETS",
        "BUILT-IN PALETTES, INCLUDING ONE PARSED FROM GRADIENT BYTES AT RUNTIME",
    );
    let (x, y, w, h) = canvas.content_rect();
    let row_h = h / rows.len() as u32;
    let gap = 10;
    let strip_h = row_h - gap;

    for (i, (_, pal)) in rows.iter().enumerate() {
        let colors = ramp(pal, SAMPLES);
        canvas.draw_strip(&colors, x, y + i as u32 * row_h, w, strip_h);
    }

    canvas.save("presets");
}
