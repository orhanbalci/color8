//! Thin safe wrappers around a C transcription of FastLED's colorutils
//! reference algorithms (FastLED 3.6.0, the last release before the `fl::`
//! namespace refactor), compiled from `src/shim.c` via `cc` in `build.rs`.
//!
//! This crate exists solely so the `color8` crate's test suite can
//! differentially compare its `#![forbid(unsafe_code)]` Rust port against
//! FastLED's actual C behavior, without requiring `unsafe` in the crate
//! under test.

mod ffi {
    extern "C" {
        pub fn fl_hsv2rgb_rainbow(
            hue: u8,
            sat: u8,
            val: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
        pub fn fl_hsv2rgb_spectrum(
            hue: u8,
            sat: u8,
            val: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
        pub fn fl_rgb2hsv_approximate(
            r: u8,
            g: u8,
            b: u8,
            out_h: *mut u8,
            out_s: *mut u8,
            out_v: *mut u8,
        );

        pub fn fl_crgb_add(
            r1: u8,
            g1: u8,
            b1: u8,
            r2: u8,
            g2: u8,
            b2: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
        pub fn fl_crgb_sub(
            r1: u8,
            g1: u8,
            b1: u8,
            r2: u8,
            g2: u8,
            b2: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
        pub fn fl_crgb_mul(
            r: u8,
            g: u8,
            b: u8,
            d: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
        pub fn fl_crgb_div(
            r: u8,
            g: u8,
            b: u8,
            d: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
        pub fn fl_crgb_and(
            r1: u8,
            g1: u8,
            b1: u8,
            r2: u8,
            g2: u8,
            b2: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
        pub fn fl_crgb_or(
            r1: u8,
            g1: u8,
            b1: u8,
            r2: u8,
            g2: u8,
            b2: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
        pub fn fl_crgb_neg(r: u8, g: u8, b: u8, out_r: *mut u8, out_g: *mut u8, out_b: *mut u8);
        pub fn fl_crgb_add_to_rgb(
            r: u8,
            g: u8,
            b: u8,
            d: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
        pub fn fl_crgb_subtract_from_rgb(
            r: u8,
            g: u8,
            b: u8,
            d: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
        pub fn fl_crgb_nscale8(
            r: u8,
            g: u8,
            b: u8,
            scale: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
        pub fn fl_crgb_nscale8_video(
            r: u8,
            g: u8,
            b: u8,
            scale: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
        pub fn fl_crgb_nscale8_rgb(
            r: u8,
            g: u8,
            b: u8,
            sr: u8,
            sg: u8,
            sb: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );

        pub fn fl_fill_gradient_rgb2(
            num_leds: u16,
            r1: u8,
            g1: u8,
            b1: u8,
            r2: u8,
            g2: u8,
            b2: u8,
            out_r: *mut u8,
            out_g: *mut u8,
            out_b: *mut u8,
        );
    }
}

/// Converts HSV to RGB using FastLED's `hsv2rgb_rainbow` C reference.
/// Returns `(r, g, b)`.
pub fn hsv2rgb_rainbow(hue: u8, sat: u8, val: u8) -> (u8, u8, u8) {
    let (mut r, mut g, mut b) = (0u8, 0u8, 0u8);
    unsafe {
        ffi::fl_hsv2rgb_rainbow(hue, sat, val, &mut r, &mut g, &mut b);
    }
    (r, g, b)
}

/// Converts HSV to RGB using FastLED's `hsv2rgb_spectrum` C reference.
pub fn hsv2rgb_spectrum(hue: u8, sat: u8, val: u8) -> (u8, u8, u8) {
    let (mut r, mut g, mut b) = (0u8, 0u8, 0u8);
    unsafe {
        ffi::fl_hsv2rgb_spectrum(hue, sat, val, &mut r, &mut g, &mut b);
    }
    (r, g, b)
}

/// Converts RGB to HSV using FastLED's `rgb2hsv_approximate` C reference.
pub fn rgb2hsv_approximate(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let (mut h, mut s, mut v) = (0u8, 0u8, 0u8);
    unsafe {
        ffi::fl_rgb2hsv_approximate(r, g, b, &mut h, &mut s, &mut v);
    }
    (h, s, v)
}

/// A plain (r, g, b) triple, used for the CRGB operator reference functions.
pub type Rgb = (u8, u8, u8);

macro_rules! binary_op {
    ($name:ident, $ffi:path) => {
        pub fn $name(a: Rgb, b: Rgb) -> Rgb {
            let (mut r, mut g, mut bl) = (0u8, 0u8, 0u8);
            unsafe {
                $ffi(a.0, a.1, a.2, b.0, b.1, b.2, &mut r, &mut g, &mut bl);
            }
            (r, g, bl)
        }
    };
}

macro_rules! scalar_op {
    ($name:ident, $ffi:path) => {
        pub fn $name(a: Rgb, d: u8) -> Rgb {
            let (mut r, mut g, mut bl) = (0u8, 0u8, 0u8);
            unsafe {
                $ffi(a.0, a.1, a.2, d, &mut r, &mut g, &mut bl);
            }
            (r, g, bl)
        }
    };
}

binary_op!(crgb_add, ffi::fl_crgb_add);
binary_op!(crgb_sub, ffi::fl_crgb_sub);
binary_op!(crgb_and, ffi::fl_crgb_and);
binary_op!(crgb_or, ffi::fl_crgb_or);
scalar_op!(crgb_mul, ffi::fl_crgb_mul);
scalar_op!(crgb_div, ffi::fl_crgb_div);
scalar_op!(crgb_add_to_rgb, ffi::fl_crgb_add_to_rgb);
scalar_op!(crgb_subtract_from_rgb, ffi::fl_crgb_subtract_from_rgb);
scalar_op!(crgb_nscale8, ffi::fl_crgb_nscale8);
scalar_op!(crgb_nscale8_video, ffi::fl_crgb_nscale8_video);

pub fn crgb_neg(a: Rgb) -> Rgb {
    let (mut r, mut g, mut b) = (0u8, 0u8, 0u8);
    unsafe {
        ffi::fl_crgb_neg(a.0, a.1, a.2, &mut r, &mut g, &mut b);
    }
    (r, g, b)
}

pub fn crgb_nscale8_rgb(a: Rgb, scale: Rgb) -> Rgb {
    let (mut r, mut g, mut b) = (0u8, 0u8, 0u8);
    unsafe {
        ffi::fl_crgb_nscale8_rgb(
            a.0, a.1, a.2, scale.0, scale.1, scale.2, &mut r, &mut g, &mut b,
        );
    }
    (r, g, b)
}

/// Fills `num_leds` pixels between `c1` and `c2` using FastLED's
/// `fill_gradient_RGB` (2-stop) C reference. Returns one `(r, g, b)` per
/// pixel.
pub fn fill_gradient_rgb2(num_leds: u16, c1: Rgb, c2: Rgb) -> Vec<Rgb> {
    let n = num_leds as usize;
    let (mut r, mut g, mut b) = (vec![0u8; n], vec![0u8; n], vec![0u8; n]);
    unsafe {
        ffi::fl_fill_gradient_rgb2(
            num_leds,
            c1.0,
            c1.1,
            c1.2,
            c2.0,
            c2.1,
            c2.2,
            r.as_mut_ptr(),
            g.as_mut_ptr(),
            b.as_mut_ptr(),
        );
    }
    (0..n).map(|i| (r[i], g[i], b[i])).collect()
}
