//! Renders `fill_gradient_rgb`/`fill_gradient_rgb3`/`fill_gradient_rgb4`
//! stacked, so 2-, 3- and 4-stop gradients can be compared at a glance.

#[path = "support/mod.rs"]
mod support;

use color8::{Crgb, fill_gradient_rgb, fill_gradient_rgb3, fill_gradient_rgb4};
use support::Canvas;

const N: usize = 500;

fn main() {
    let mut sunset = [Crgb::default(); N];
    fill_gradient_rgb(&mut sunset, Crgb::new(255, 94, 58), Crgb::new(255, 205, 86));

    let mut ocean = [Crgb::default(); N];
    fill_gradient_rgb3(
        &mut ocean,
        Crgb::new(0, 168, 204),
        Crgb::new(24, 90, 189),
        Crgb::new(97, 42, 173),
    );

    let mut carnival = [Crgb::default(); N];
    fill_gradient_rgb4(
        &mut carnival,
        Crgb::new(255, 209, 102),
        Crgb::new(255, 87, 87),
        Crgb::new(155, 66, 245),
        Crgb::new(46, 196, 182),
    );

    let mut canvas = Canvas::new("GRADIENT FILLS", "FILL_GRADIENT_RGB - 2, 3 AND 4-STOP");
    let (x, y, w, h) = canvas.content_rect();
    let row_h = h / 3;
    let gap = 14;
    let strip_h = row_h - gap;

    canvas.draw_strip(&sunset, x, y, w, strip_h);
    canvas.draw_strip(&ocean, x, y + row_h, w, strip_h);
    canvas.draw_strip(&carnival, x, y + 2 * row_h, w, strip_h);

    canvas.save("gradient_fills");
}
