//! `color8` — fast 8-bit color primitives for `no_std` embedded targets.
//!
//! This is a Rust port of [FastLED's `colorutils`](https://github.com/FastLED/FastLED):
//! the `CRGB`/`CHSV` pixel types with their arithmetic and scaling
//! operators, the HSV↔RGB conversions, and the array fill/gradient
//! helpers used throughout FastLED's effects.
//!
//! The port targets **FastLED 3.6.0** — the last release before upstream
//! moved everything into the `fl::` namespace and split `colorutils.cpp`
//! apart. Every function here is validated bit-for-bit against a C
//! transcription of that release (see the `fastled-ref` test crate).
//!
//! All functions are pure integer math with defined overflow behavior
//! (saturating, wrapping or truncating, matching the semantics of the
//! original C) — nothing here panics or allocates.
#![no_std]
#![forbid(unsafe_code)]

pub mod blend;
pub mod fill;
pub mod heat;
pub mod hsv;
pub mod palette;
pub mod rgb;

pub use blend::{
    blend, blend_hsv, blend_hsv_slice, blend_slice, nblend, nblend_hsv, nblend_hsv_slice,
    nblend_slice,
};
pub use fill::{
    FromHsv, GradientDirection, fade_light_by, fade_raw, fade_to_black_by, fade_using_color,
    fade_video, fill_gradient, fill_gradient_range, fill_gradient_rgb, fill_gradient_rgb_range,
    fill_gradient_rgb3, fill_gradient_rgb4, fill_gradient3, fill_gradient4, fill_rainbow,
    fill_rainbow_circular, fill_solid, nscale8, nscale8_raw, nscale8_video,
};
pub use heat::heat_color;
pub use hsv::{Chsv, HsvHue, hsv2rgb_rainbow, hsv2rgb_spectrum, rgb2hsv_approximate};
pub use palette::{
    ChsvPalette16, ChsvPalette32, ChsvPalette256, ColorBlend, CrgbPalette16, CrgbPalette32,
    CrgbPalette256, color_from_palette16, color_from_palette16_hsv, color_from_palette32,
    color_from_palette32_hsv, color_from_palette256, color_from_palette256_hsv,
};
pub use rgb::Crgb;
