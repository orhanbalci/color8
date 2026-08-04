//! Blending one color or strip toward another.
//!
//! Direct ports of the `blend`/`nblend` family in FastLED's
//! `src/colorutils.cpp` (FastLED 3.6.0).
//!
//! The `n`-prefixed forms mutate in place ("n" for "in-place" in FastLED's
//! naming); the unprefixed forms return a new value. RGB blending is
//! per-channel and straightforward; HSV blending has to decide which way
//! around the color wheel to travel, so it takes a [`GradientDirection`].
//!
//! These sit on [`lib8tion::blend8_8bit_full_range`] rather than
//! `lib8tion::blend8`: FastLED changed its `blend8` formula after the 3.6.x
//! line, and the two disagree at the top of the range
//! (`blend8(0, 255, 255)` is 255 in 3.6.0 and 254 in current master).

use lib8tion::{Fract8, blend8_8bit_full_range, scale8};

use crate::fill::GradientDirection;
use crate::hsv::Chsv;
use crate::rgb::Crgb;

/// Blends `existing` toward `overlay` in place, by `amount_of_overlay / 256`.
///
/// `0` leaves `existing` untouched; `255` replaces it with `overlay`.
pub fn nblend(existing: &mut Crgb, overlay: Crgb, amount_of_overlay: u8) {
    if amount_of_overlay == 0 {
        return;
    }

    if amount_of_overlay == 255 {
        *existing = overlay;
        return;
    }

    existing.r = blend8_8bit_full_range(existing.r, overlay.r, amount_of_overlay);
    existing.g = blend8_8bit_full_range(existing.g, overlay.g, amount_of_overlay);
    existing.b = blend8_8bit_full_range(existing.b, overlay.b, amount_of_overlay);
}

/// Returns `p1` blended toward `p2` by `amount_of_p2 / 256`, leaving both
/// inputs untouched.
pub fn blend(p1: Crgb, p2: Crgb, amount_of_p2: u8) -> Crgb {
    let mut nu = p1;
    nblend(&mut nu, p2, amount_of_p2);
    nu
}

/// Blends each pixel of `existing` toward the matching pixel of `overlay`,
/// in place.
///
/// FastLED takes two pointers and a count; taking slices means the shorter
/// of the two bounds the work, so mismatched lengths truncate instead of
/// running off the end.
pub fn nblend_slice(existing: &mut [Crgb], overlay: &[Crgb], amount_of_overlay: u8) {
    for (e, o) in existing.iter_mut().zip(overlay.iter()) {
        nblend(e, *o, amount_of_overlay);
    }
}

/// Writes `src1` blended toward `src2` into `dest`, pixel by pixel.
///
/// The number of pixels written is the shortest of the three slices.
pub fn blend_slice(src1: &[Crgb], src2: &[Crgb], dest: &mut [Crgb], amount_of_src2: u8) {
    for ((d, a), b) in dest.iter_mut().zip(src1.iter()).zip(src2.iter()) {
        *d = blend(*a, *b, amount_of_src2);
    }
}

/// Blends `existing` toward `overlay` in place in HSV space, travelling
/// around the color wheel in the direction given by `direction`.
pub fn nblend_hsv(
    existing: &mut Chsv,
    overlay: Chsv,
    amount_of_overlay: u8,
    direction: GradientDirection,
) {
    if amount_of_overlay == 0 {
        return;
    }

    if amount_of_overlay == 255 {
        *existing = overlay;
        return;
    }

    let amount_of_keep = 255 - amount_of_overlay;

    let mut huedelta8 = overlay.hue.wrapping_sub(existing.hue);

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

    if direction == GradientDirection::Forward {
        existing.hue = existing
            .hue
            .wrapping_add(scale8(huedelta8, Fract8(amount_of_overlay)));
    } else {
        huedelta8 = huedelta8.wrapping_neg();
        existing.hue = existing
            .hue
            .wrapping_sub(scale8(huedelta8, Fract8(amount_of_overlay)));
    }

    // Both terms are bytes and FastLED assigns the sum straight back into a
    // uint8_t field, so this truncates rather than saturating.
    existing.sat = scale8(existing.sat, Fract8(amount_of_keep))
        .wrapping_add(scale8(overlay.sat, Fract8(amount_of_overlay)));
    existing.val = scale8(existing.val, Fract8(amount_of_keep))
        .wrapping_add(scale8(overlay.val, Fract8(amount_of_overlay)));
}

/// Returns `p1` blended toward `p2` in HSV space, leaving both inputs
/// untouched.
pub fn blend_hsv(p1: Chsv, p2: Chsv, amount_of_p2: u8, direction: GradientDirection) -> Chsv {
    let mut nu = p1;
    nblend_hsv(&mut nu, p2, amount_of_p2, direction);
    nu
}

/// Blends each pixel of `existing` toward the matching pixel of `overlay`
/// in HSV space, in place.
pub fn nblend_hsv_slice(
    existing: &mut [Chsv],
    overlay: &[Chsv],
    amount_of_overlay: u8,
    direction: GradientDirection,
) {
    for (e, o) in existing.iter_mut().zip(overlay.iter()) {
        nblend_hsv(e, *o, amount_of_overlay, direction);
    }
}

/// Writes `src1` blended toward `src2` into `dest`, pixel by pixel, in HSV
/// space.
pub fn blend_hsv_slice(
    src1: &[Chsv],
    src2: &[Chsv],
    dest: &mut [Chsv],
    amount_of_src2: u8,
    direction: GradientDirection,
) {
    for ((d, a), b) in dest.iter_mut().zip(src1.iter()).zip(src2.iter()) {
        *d = blend_hsv(*a, *b, amount_of_src2, direction);
    }
}
