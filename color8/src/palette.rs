//! `CrgbPalette16`/`32`/`256`, `ChsvPalette16`/`32`/`256`, and
//! `ColorFromPalette` — direct port of the palette-interpolation logic in
//! FastLED 3.6.0's `colorutils.cpp`, transcribed from the actual
//! 3.6.0-tagged source (`github.com/FastLED/FastLED`, tag `3.6.0`). Only
//! the RAM-backed palette overloads are ported — not the
//! `TProgmemRGBPalette*` ones, which color8 has no analogue of.
//!
//! Brightness handling is transcribed exactly as FastLED has it, and it is
//! *not* uniform across these six functions:
//! - `CrgbPalette16`/`32`: `brightness == 0` forces black; otherwise plain
//!   `scale8(x, brightness + 1)` (not `scale8_video`).
//! - `CrgbPalette256`: `scale8_video(x, brightness + 1)` — no
//!   `brightness == 0` special case, unlike the 16/32-entry versions.
//! - `ChsvPalette16`/`32`/`256`: only `val` is brightness-scaled, via plain
//!   `scale8_video(val, brightness)` — no "+1" rounding adjustment, unlike
//!   the `Crgb` versions.
//!
//! `LinearBlendNoWrap` rescales the index via FastLED's `map8(index, 0, N)`,
//! which reduces to `scale8(index, N)`. `N` is 239 for the 16-entry
//! palettes and 247 for the 32-entry ones — *not* 240/248: `scale8`'s
//! internal `+1` means `scale8(index, N)` can reach up to `N`, and landing
//! exactly on the last entry (with no fractional remainder) requires `N` to
//! be one less than the "obvious" `240`/`248`.

use lib8tion::{Fract8, scale8, scale8_video};

use crate::hsv::Chsv;
use crate::rgb::Crgb;

/// How the 16- and 32-entry `color_from_palette*` functions blend between
/// adjacent palette entries. FastLED's `TBlendType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorBlend {
    /// No interpolation: step to the nearest lower entry.
    NoBlend,
    /// Linearly interpolate between entries, wrapping from the last entry
    /// back to the first as the index crosses 255 -> 0.
    LinearBlend,
    /// Linearly interpolate between entries, but rescale the index first so
    /// that the top of the range lands exactly on the last entry instead of
    /// blending toward a wrapped-around entry 0.
    LinearBlendNoWrap,
}

/// A 16-entry RGB color palette, indexed by a `u8` via
/// [`color_from_palette16`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrgbPalette16(pub [Crgb; 16]);

/// A 32-entry RGB color palette, indexed by a `u8` via
/// [`color_from_palette32`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrgbPalette32(pub [Crgb; 32]);

/// A 256-entry RGB color palette: one entry per possible `u8` index, so
/// [`color_from_palette256`] never interpolates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrgbPalette256(pub [Crgb; 256]);

/// A 16-entry HSV color palette, indexed by a `u8` via
/// [`color_from_palette16_hsv`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChsvPalette16(pub [Chsv; 16]);

/// A 32-entry HSV color palette, indexed by a `u8` via
/// [`color_from_palette32_hsv`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChsvPalette32(pub [Chsv; 32]);

/// A 256-entry HSV color palette: one entry per possible `u8` index, so
/// [`color_from_palette256_hsv`] never interpolates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChsvPalette256(pub [Chsv; 256]);

impl CrgbPalette16 {
    /// Builds a palette from its 16 entries.
    #[inline]
    pub const fn new(entries: [Crgb; 16]) -> Self {
        Self(entries)
    }
}

impl CrgbPalette32 {
    /// Builds a palette from its 32 entries.
    #[inline]
    pub const fn new(entries: [Crgb; 32]) -> Self {
        Self(entries)
    }
}

impl CrgbPalette256 {
    /// Builds a palette from its 256 entries.
    #[inline]
    pub const fn new(entries: [Crgb; 256]) -> Self {
        Self(entries)
    }
}

impl ChsvPalette16 {
    /// Builds a palette from its 16 entries.
    #[inline]
    pub const fn new(entries: [Chsv; 16]) -> Self {
        Self(entries)
    }
}

impl ChsvPalette32 {
    /// Builds a palette from its 32 entries.
    #[inline]
    pub const fn new(entries: [Chsv; 32]) -> Self {
        Self(entries)
    }
}

impl ChsvPalette256 {
    /// Builds a palette from its 256 entries.
    #[inline]
    pub const fn new(entries: [Chsv; 256]) -> Self {
        Self(entries)
    }
}

macro_rules! impl_index {
    ($ty:ty, $entry:ty) => {
        impl core::ops::Index<usize> for $ty {
            type Output = $entry;
            #[inline]
            fn index(&self, i: usize) -> &$entry {
                &self.0[i]
            }
        }
    };
}

impl_index!(CrgbPalette16, Crgb);
impl_index!(CrgbPalette32, Crgb);
impl_index!(CrgbPalette256, Crgb);
impl_index!(ChsvPalette16, Chsv);
impl_index!(ChsvPalette32, Chsv);
impl_index!(ChsvPalette256, Chsv);

/// Blends between `entries[hi]` and its (possibly wrapped-around) neighbor
/// by `f2/255`, per FastLED's `scale8_LEAVING_R1_DIRTY` pair-sum. On the
/// portable-C path that primitive is bit-identical to plain `scale8`; the
/// `_LEAVING_R1_DIRTY` naming only matters for an AVR register-zeroing step.
#[inline]
fn interpolate_rgb(entries: &[Crgb], hi: usize, f2: u8, blend: ColorBlend) -> Crgb {
    let e0 = entries[hi];
    if blend == ColorBlend::NoBlend || f2 == 0 {
        return e0;
    }

    let next = if hi + 1 == entries.len() { 0 } else { hi + 1 };
    let e1 = entries[next];
    let f1 = 255 - f2;

    // FastLED assigns the two scale8 terms' sum back into a uint8_t, which
    // truncates rather than saturates.
    Crgb::new(
        scale8(e0.r, Fract8(f1)).wrapping_add(scale8(e1.r, Fract8(f2))),
        scale8(e0.g, Fract8(f1)).wrapping_add(scale8(e1.g, Fract8(f2))),
        scale8(e0.b, Fract8(f1)).wrapping_add(scale8(e1.b, Fract8(f2))),
    )
}

/// Applies a `CrgbPalette16`/`32` ColorFromPalette's brightness stage:
/// `brightness == 0` forces black, otherwise plain `scale8(x, brightness+1)`.
#[inline]
fn apply_brightness_rgb(mut entry: Crgb, brightness: u8) -> Crgb {
    if brightness == 255 {
        return entry;
    }
    if brightness == 0 {
        return Crgb::new(0, 0, 0);
    }
    let b2 = brightness + 1;
    entry.r = scale8(entry.r, Fract8(b2));
    entry.g = scale8(entry.g, Fract8(b2));
    entry.b = scale8(entry.b, Fract8(b2));
    entry
}

/// `ColorFromPalette` against a 16-entry RGB palette.
pub fn color_from_palette16(
    pal: &CrgbPalette16,
    mut index: u8,
    brightness: u8,
    blend: ColorBlend,
) -> Crgb {
    if blend == ColorBlend::LinearBlendNoWrap {
        index = scale8(index, Fract8(239));
    }

    let hi4 = (index >> 4) as usize;
    let lo4 = index & 0x0F;
    let f2 = lo4 << 4;

    let entry = interpolate_rgb(&pal.0, hi4, f2, blend);
    apply_brightness_rgb(entry, brightness)
}

/// `ColorFromPalette` against a 32-entry RGB palette.
pub fn color_from_palette32(
    pal: &CrgbPalette32,
    mut index: u8,
    brightness: u8,
    blend: ColorBlend,
) -> Crgb {
    if blend == ColorBlend::LinearBlendNoWrap {
        index = scale8(index, Fract8(247));
    }

    let hi5 = (index >> 3) as usize;
    let lo3 = index & 0x07;
    let f2 = lo3 << 5;

    let entry = interpolate_rgb(&pal.0, hi5, f2, blend);
    apply_brightness_rgb(entry, brightness)
}

/// `ColorFromPalette` against a 256-entry RGB palette. Every index maps to
/// exactly one entry, so there is nothing to interpolate between.
pub fn color_from_palette256(pal: &CrgbPalette256, index: u8, brightness: u8) -> Crgb {
    let mut entry = pal.0[index as usize];
    if brightness != 255 {
        let b2 = brightness + 1;
        entry.r = scale8_video(entry.r, Fract8(b2));
        entry.g = scale8_video(entry.g, Fract8(b2));
        entry.b = scale8_video(entry.b, Fract8(b2));
    }
    entry
}

/// Blends `entries[hi]` toward its (possibly wrapped-around) neighbor by
/// `f2/255`. Saturation and value blend like RGB channels; hue blends along
/// the shorter of the two directions around the 8-bit hue wheel, and
/// black/white entries (`sat == 0` or `val == 0`) adopt the other entry's
/// hue first, since they have no hue of their own.
#[inline]
fn interpolate_hsv(entries: &[Chsv], hi: usize, f2: u8, blend: ColorBlend) -> Chsv {
    let e0 = entries[hi];
    if blend == ColorBlend::NoBlend || f2 == 0 {
        return e0;
    }

    let next = if hi + 1 == entries.len() { 0 } else { hi + 1 };
    let e1 = entries[next];
    let f1 = 255 - f2;

    let mut hue1 = e0.hue;
    let mut hue2 = e1.hue;
    if e0.sat == 0 || e0.val == 0 {
        hue1 = hue2;
    }
    if e1.sat == 0 || e1.val == 0 {
        hue2 = hue1;
    }

    let sat = scale8(e0.sat, Fract8(f1)).wrapping_add(scale8(e1.sat, Fract8(f2)));
    let val = scale8(e0.val, Fract8(f1)).wrapping_add(scale8(e1.val, Fract8(f2)));

    let deltahue = hue2.wrapping_sub(hue1);
    let hue = if deltahue & 0x80 != 0 {
        let reverse_delta = (256u16 - deltahue as u16) as u8;
        hue1.wrapping_sub(scale8(reverse_delta, Fract8(f2)))
    } else {
        hue1.wrapping_add(scale8(deltahue, Fract8(f2)))
    };

    Chsv::new(hue, sat, val)
}

/// `ColorFromPalette` against a 16-entry HSV palette.
pub fn color_from_palette16_hsv(
    pal: &ChsvPalette16,
    mut index: u8,
    brightness: u8,
    blend: ColorBlend,
) -> Chsv {
    if blend == ColorBlend::LinearBlendNoWrap {
        index = scale8(index, Fract8(239));
    }

    let hi4 = (index >> 4) as usize;
    let lo4 = index & 0x0F;
    let f2 = lo4 << 4;

    let mut entry = interpolate_hsv(&pal.0, hi4, f2, blend);
    if brightness != 255 {
        entry.val = scale8_video(entry.val, Fract8(brightness));
    }
    entry
}

/// `ColorFromPalette` against a 32-entry HSV palette.
pub fn color_from_palette32_hsv(
    pal: &ChsvPalette32,
    mut index: u8,
    brightness: u8,
    blend: ColorBlend,
) -> Chsv {
    if blend == ColorBlend::LinearBlendNoWrap {
        index = scale8(index, Fract8(247));
    }

    let hi5 = (index >> 3) as usize;
    let lo3 = index & 0x07;
    let f2 = lo3 << 5;

    let mut entry = interpolate_hsv(&pal.0, hi5, f2, blend);
    if brightness != 255 {
        entry.val = scale8_video(entry.val, Fract8(brightness));
    }
    entry
}

/// `ColorFromPalette` against a 256-entry HSV palette. Every index maps to
/// exactly one entry, so there is nothing to interpolate between.
pub fn color_from_palette256_hsv(pal: &ChsvPalette256, index: u8, brightness: u8) -> Chsv {
    let mut entry = pal.0[index as usize];
    if brightness != 255 {
        entry.val = scale8_video(entry.val, Fract8(brightness));
    }
    entry
}
