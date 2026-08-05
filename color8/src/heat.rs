//! `HeatColor` — direct port of `src/colorutils.cpp` in FastLED 3.6.0.

use lib8tion::{Fract8, scale8_video};

use crate::rgb::Crgb;

/// Maps a "heat" value (0 = black, 255 = white-hot) to a color along
/// FastLED's black -> red -> yellow -> white heat ramp, as used by fire
/// effects such as `Fire2012`.
#[inline]
pub fn heat_color(temperature: u8) -> Crgb {
    // Scale 'heat' down from 0-255 to 0-191, which can then be easily
    // divided into three equal 'thirds' of 64 units each.
    let t192 = scale8_video(temperature, Fract8(191));

    // calculate a value that ramps up from zero to 255 in each 'third' of
    // the scale.
    let heatramp = (t192 & 0x3F) << 2; // 0..63 scaled up to 0..252

    if t192 & 0x80 != 0 {
        // we're in the hottest third
        Crgb::new(255, 255, heatramp)
    } else if t192 & 0x40 != 0 {
        // we're in the middle third
        Crgb::new(255, heatramp, 0)
    } else {
        // we're in the coolest third
        Crgb::new(heatramp, 0, 0)
    }
}
