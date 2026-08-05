//! Renders `hsv2rgb_rainbow()` across all 256 hues at full saturation and
//! value — the perceptually-corrected rainbow FastLED is named for.

#[path = "support/mod.rs"]
mod support;

use color8::{Chsv, hsv2rgb_rainbow};
use support::Canvas;

fn main() {
    let colors: Vec<_> = (0..=255u16)
        .map(|h| hsv2rgb_rainbow(Chsv::from_hue(h as u8)))
        .collect();

    let mut canvas = Canvas::new("FULL SPECTRUM", "HSV2RGB_RAINBOW() ACROSS ALL 256 HUES");
    let (x, y, w, h) = canvas.content_rect();
    canvas.draw_strip(&colors, x, y, w, h);
    canvas.save("rainbow_sweep");
}
