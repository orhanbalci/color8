//! Array fill and gradient helpers.
//!
//! Direct ports of the `fill_*` functions in FastLED's `src/colorutils.cpp`
//! and the `fill_gradient<T>` template in `src/colorutils.h` (FastLED
//! 3.6.0).
//!
//! FastLED takes a raw pointer plus a count; these take a `&mut [T]` and
//! fill it end to end, which makes the length the slice's own business.
//! Where FastLED would compute `numLeds - 1` and underflow on an empty
//! array (running off the end of a 65535-iteration loop), these functions
//! return without writing anything — the only deliberate behavioral
//! deviation from the C, and one that trades undefined behavior for a
//! no-op.

use crate::hsv::{Chsv, hsv2rgb_rainbow};
use crate::rgb::Crgb;

/// A pixel type that a [`Chsv`] can be written into.
///
/// FastLED's `fill_rainbow`/`fill_gradient` are templates that assign a
/// `CHSV` into a `T*` array, relying on `CRGB`'s implicit
/// `CRGB(const CHSV&)` constructor (which calls `hsv2rgb_rainbow`) when
/// `T` is `CRGB`. This trait is that implicit conversion, made explicit.
pub trait FromHsv: Copy {
    /// Converts an HSV pixel into this pixel type.
    fn from_hsv(hsv: Chsv) -> Self;
}

impl FromHsv for Chsv {
    #[inline]
    fn from_hsv(hsv: Chsv) -> Self {
        hsv
    }
}

impl FromHsv for Crgb {
    #[inline]
    fn from_hsv(hsv: Chsv) -> Self {
        hsv2rgb_rainbow(hsv)
    }
}

/// Fills every pixel with a single color.
pub fn fill_solid<T: Copy>(leds: &mut [T], color: T) {
    for led in leds.iter_mut() {
        *led = color;
    }
}

/// Fills with a rainbow, starting at `initial_hue` and stepping the hue by
/// `delta_hue` per pixel (wrapping around the color wheel). Saturation is
/// fixed at 240 and value at 255, matching FastLED.
pub fn fill_rainbow<T: FromHsv>(leds: &mut [T], initial_hue: u8, delta_hue: u8) {
    let mut hsv = Chsv::new(initial_hue, 240, 255);
    for led in leds.iter_mut() {
        *led = T::from_hsv(hsv);
        hsv.hue = hsv.hue.wrapping_add(delta_hue);
    }
}

/// Fills with exactly one full turn around the color wheel, spread evenly
/// across the whole slice, starting at `initial_hue`. Set `reversed` to
/// travel counter-clockwise.
///
/// Unlike [`fill_rainbow`], the hue step is derived from the pixel count
/// rather than given, so the gradient always closes the loop regardless of
/// how many pixels there are.
pub fn fill_rainbow_circular<T: FromHsv>(leds: &mut [T], initial_hue: u8, reversed: bool) {
    if leds.is_empty() {
        return; // avoid div/0
    }

    let mut hsv = Chsv::new(initial_hue, 240, 255);

    // Hue change per LED, kept at 8 extra bits of precision.
    let hue_change = 65535u16 / leds.len() as u16;
    let mut hue_offset = 0u16;

    for led in leds.iter_mut() {
        *led = T::from_hsv(hsv);
        if reversed {
            hue_offset = hue_offset.wrapping_sub(hue_change);
        } else {
            hue_offset = hue_offset.wrapping_add(hue_change);
        }
        hsv.hue = initial_hue.wrapping_add((hue_offset >> 8) as u8);
    }
}

/// Which way around the color wheel a hue gradient travels.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GradientDirection {
    /// Hue always goes clockwise around the color wheel.
    Forward,
    /// Hue always goes counter-clockwise around the color wheel.
    Backward,
    /// Hue goes whichever way is shortest.
    #[default]
    Shortest,
    /// Hue goes whichever way is longest.
    Longest,
}

/// Fills `leds[start_pos ..= end_pos]` with an RGB-space gradient between
/// two colors. Interpolation runs in RGB with 8 extra bits of fractional
/// precision per channel.
///
/// If `end_pos < start_pos` the two endpoints are swapped, matching
/// FastLED. Indices at or past the end of the slice are skipped.
pub fn fill_gradient_rgb_range(
    leds: &mut [Crgb],
    start_pos: u16,
    start_color: Crgb,
    end_pos: u16,
    end_color: Crgb,
) {
    // if the points are in the wrong order, straighten them
    let (start_pos, end_pos, start_color, end_color) = if end_pos < start_pos {
        (end_pos, start_pos, end_color, start_color)
    } else {
        (start_pos, end_pos, start_color, end_color)
    };

    let rdistance87 = (end_color.r as i16 - start_color.r as i16) << 7;
    let gdistance87 = (end_color.g as i16 - start_color.g as i16) << 7;
    let bdistance87 = (end_color.b as i16 - start_color.b as i16) << 7;

    let pixeldistance = end_pos - start_pos;
    // FastLED narrows the u16 distance to i16 here; a span wider than
    // 32767 pixels therefore goes negative, and the gradient runs
    // backwards. Reproduced rather than corrected — it is observable
    // behavior of the version being ported.
    let divisor = if pixeldistance != 0 {
        pixeldistance as i16
    } else {
        1
    };

    let rdelta87 = (rdistance87 / divisor).wrapping_mul(2);
    let gdelta87 = (gdistance87 / divisor).wrapping_mul(2);
    let bdelta87 = (bdistance87 / divisor).wrapping_mul(2);

    let mut r88 = (start_color.r as u16) << 8;
    let mut g88 = (start_color.g as u16) << 8;
    let mut b88 = (start_color.b as u16) << 8;

    for i in start_pos..=end_pos {
        if let Some(led) = leds.get_mut(i as usize) {
            *led = Crgb::new((r88 >> 8) as u8, (g88 >> 8) as u8, (b88 >> 8) as u8);
        }
        r88 = r88.wrapping_add(rdelta87 as u16);
        g88 = g88.wrapping_add(gdelta87 as u16);
        b88 = b88.wrapping_add(bdelta87 as u16);
    }
}

/// Fills the whole slice with an RGB-space gradient from `c1` to `c2`.
pub fn fill_gradient_rgb(leds: &mut [Crgb], c1: Crgb, c2: Crgb) {
    if leds.is_empty() {
        return;
    }
    let last = (leds.len() - 1) as u16;
    fill_gradient_rgb_range(leds, 0, c1, last, c2);
}

/// Fills the whole slice with a three-stop RGB-space gradient.
pub fn fill_gradient_rgb3(leds: &mut [Crgb], c1: Crgb, c2: Crgb, c3: Crgb) {
    if leds.is_empty() {
        return;
    }
    let num_leds = leds.len() as u16;
    let half = num_leds / 2;
    let last = num_leds - 1;
    fill_gradient_rgb_range(leds, 0, c1, half, c2);
    fill_gradient_rgb_range(leds, half, c2, last, c3);
}

/// Fills the whole slice with a four-stop RGB-space gradient.
pub fn fill_gradient_rgb4(leds: &mut [Crgb], c1: Crgb, c2: Crgb, c3: Crgb, c4: Crgb) {
    if leds.is_empty() {
        return;
    }
    let num_leds = leds.len() as u16;
    let onethird = num_leds / 3;
    let twothirds = (num_leds * 2) / 3;
    let last = num_leds - 1;
    fill_gradient_rgb_range(leds, 0, c1, onethird, c2);
    fill_gradient_rgb_range(leds, onethird, c2, twothirds, c3);
    fill_gradient_rgb_range(leds, twothirds, c3, last, c4);
}

/// Fills `leds[start_pos ..= end_pos]` with an HSV-space gradient between
/// two colors, interpolating hue around the color wheel in the direction
/// given by `direction`.
///
/// Interpolating in HSV rather than RGB keeps the intermediate colors
/// saturated — an RGB gradient from red to green passes through muddy
/// olive, an HSV one passes through orange and yellow.
pub fn fill_gradient_range<T: FromHsv>(
    leds: &mut [T],
    start_pos: u16,
    start_color: Chsv,
    end_pos: u16,
    end_color: Chsv,
    direction: GradientDirection,
) {
    // if the points are in the wrong order, straighten them
    let (start_pos, end_pos, mut start_color, mut end_color) = if end_pos < start_pos {
        (end_pos, start_pos, end_color, start_color)
    } else {
        (start_pos, end_pos, start_color, end_color)
    };

    // If we're fading toward black (val=0) or white (sat=0), set the end
    // hue to the start hue — this ramps smoothly to black or white
    // regardless of what hue the endcolor carried, since it doesn't matter.
    if end_color.val == 0 || end_color.sat == 0 {
        end_color.hue = start_color.hue;
    }

    // Similarly, if we're fading in *from* black or white, take the hue
    // from the far end instead.
    if start_color.val == 0 || start_color.sat == 0 {
        start_color.hue = end_color.hue;
    }

    let satdistance87 = (end_color.sat as i16 - start_color.sat as i16) << 7;
    let valdistance87 = (end_color.val as i16 - start_color.val as i16) << 7;

    let huedelta8 = end_color.hue.wrapping_sub(start_color.hue);

    let direction = match direction {
        GradientDirection::Shortest => {
            if huedelta8 > 127 {
                GradientDirection::Backward
            } else {
                GradientDirection::Forward
            }
        }
        GradientDirection::Longest => {
            if huedelta8 < 128 {
                GradientDirection::Backward
            } else {
                GradientDirection::Forward
            }
        }
        other => other,
    };

    let huedistance87 = if direction == GradientDirection::Forward {
        (huedelta8 as i16) << 7
    } else {
        // `(uint8_t)(256 - huedelta8)` truncates to 0 when huedelta8 is 0,
        // so a zero-width backward gradient stays put rather than sweeping
        // the whole wheel.
        let back = (256u16.wrapping_sub(huedelta8 as u16)) as u8;
        -((back as i16) << 7)
    };

    let pixeldistance = end_pos - start_pos;
    let divisor = if pixeldistance != 0 {
        pixeldistance as i16
    } else {
        1
    };

    let huedelta87 = (huedistance87 / divisor).wrapping_mul(2);
    let satdelta87 = (satdistance87 / divisor).wrapping_mul(2);
    let valdelta87 = (valdistance87 / divisor).wrapping_mul(2);

    let mut hue88 = (start_color.hue as u16) << 8;
    let mut sat88 = (start_color.sat as u16) << 8;
    let mut val88 = (start_color.val as u16) << 8;

    for i in start_pos..=end_pos {
        if let Some(led) = leds.get_mut(i as usize) {
            *led = T::from_hsv(Chsv::new(
                (hue88 >> 8) as u8,
                (sat88 >> 8) as u8,
                (val88 >> 8) as u8,
            ));
        }
        hue88 = hue88.wrapping_add(huedelta87 as u16);
        sat88 = sat88.wrapping_add(satdelta87 as u16);
        val88 = val88.wrapping_add(valdelta87 as u16);
    }
}

/// Fills the whole slice with an HSV-space gradient from `c1` to `c2`.
pub fn fill_gradient<T: FromHsv>(leds: &mut [T], c1: Chsv, c2: Chsv, direction: GradientDirection) {
    if leds.is_empty() {
        return;
    }
    let last = (leds.len() - 1) as u16;
    fill_gradient_range(leds, 0, c1, last, c2, direction);
}

/// Fills the whole slice with a three-stop HSV-space gradient.
pub fn fill_gradient3<T: FromHsv>(
    leds: &mut [T],
    c1: Chsv,
    c2: Chsv,
    c3: Chsv,
    direction: GradientDirection,
) {
    if leds.is_empty() {
        return;
    }
    let num_leds = leds.len() as u16;
    let half = num_leds / 2;
    let last = num_leds - 1;
    fill_gradient_range(leds, 0, c1, half, c2, direction);
    fill_gradient_range(leds, half, c2, last, c3, direction);
}

/// Fills the whole slice with a four-stop HSV-space gradient.
pub fn fill_gradient4<T: FromHsv>(
    leds: &mut [T],
    c1: Chsv,
    c2: Chsv,
    c3: Chsv,
    c4: Chsv,
    direction: GradientDirection,
) {
    if leds.is_empty() {
        return;
    }
    let num_leds = leds.len() as u16;
    let onethird = num_leds / 3;
    let twothirds = (num_leds * 2) / 3;
    let last = num_leds - 1;
    fill_gradient_range(leds, 0, c1, onethird, c2, direction);
    fill_gradient_range(leds, onethird, c2, twothirds, c3, direction);
    fill_gradient_range(leds, twothirds, c3, last, c4, direction);
}

/// Scales every pixel down toward black by `fade_by`.
pub fn fade_to_black_by(leds: &mut [Crgb], fade_by: u8) {
    for led in leds.iter_mut() {
        led.nscale8(255 - fade_by);
    }
}

/// Scales every pixel by `scale`, "video" style — a lit pixel never fully
/// extinguishes.
pub fn fade_light_by(leds: &mut [Crgb], fade_by: u8) {
    for led in leds.iter_mut() {
        led.nscale8_video(255 - fade_by);
    }
}

/// Scales every pixel by `scale` using plain-math dimming.
pub fn nscale8(leds: &mut [Crgb], scale: u8) {
    for led in leds.iter_mut() {
        led.nscale8(scale);
    }
}

/// Scales every pixel by `scale` using "video" dimming.
pub fn nscale8_video(leds: &mut [Crgb], scale: u8) {
    for led in leds.iter_mut() {
        led.nscale8_video(scale);
    }
}

/// `fade_video` — FastLED's other name for [`fade_light_by`]. Both are
/// `nscale8_video(255 - fade_by)`.
#[inline]
pub fn fade_video(leds: &mut [Crgb], fade_by: u8) {
    fade_light_by(leds, fade_by);
}

/// `fade_raw` — FastLED's other name for [`fade_to_black_by`]. Both are
/// `nscale8(255 - fade_by)`.
#[inline]
pub fn fade_raw(leds: &mut [Crgb], fade_by: u8) {
    fade_to_black_by(leds, fade_by);
}

/// `nscale8_raw` — FastLED's other name for [`nscale8`].
#[inline]
pub fn nscale8_raw(leds: &mut [Crgb], scale: u8) {
    nscale8(leds, scale);
}

/// Fades every pixel by scaling each channel with the matching channel of
/// `colormask`, so the strip fades *through* a color rather than straight
/// to black. A mask of `(255, 0, 0)` fades everything toward red.
pub fn fade_using_color(leds: &mut [Crgb], colormask: Crgb) {
    for led in leds.iter_mut() {
        led.nscale8_rgb(colormask);
    }
}
