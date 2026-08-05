//! `CRGBPalette16`/`32`/`256` and `ColorFromPalette` — direct port of the
//! palette-interpolation logic in FastLED 3.6.0's `colorutils.cpp`.
//!
//! Only the `CRGB` palette family is ported so far; the `CHSV` palette
//! family (`CHSVPalette16`/`32`/`256`), which blends hue with wraparound
//! instead of plain `scale8`, is not yet ported.

use lib8tion::{Fract8, scale8};

use crate::rgb::Crgb;

/// How [`color_from_palette16`]/[`color_from_palette32`] blend between
/// adjacent palette entries. FastLED's `TBlendType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorBlend {
    /// No interpolation: step to the nearest lower entry.
    NoBlend,
    /// Linearly interpolate between entries, wrapping from the last entry
    /// back to the first as the index crosses 255 -> 0.
    LinearBlend,
    /// Linearly interpolate between entries, but rescale the index first so
    /// that `index == 255` lands exactly on the last entry instead of
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

impl core::ops::Index<usize> for CrgbPalette16 {
    type Output = Crgb;
    #[inline]
    fn index(&self, i: usize) -> &Crgb {
        &self.0[i]
    }
}

impl core::ops::Index<usize> for CrgbPalette32 {
    type Output = Crgb;
    #[inline]
    fn index(&self, i: usize) -> &Crgb {
        &self.0[i]
    }
}

impl core::ops::Index<usize> for CrgbPalette256 {
    type Output = Crgb;
    #[inline]
    fn index(&self, i: usize) -> &Crgb {
        &self.0[i]
    }
}

/// Blends between `entries[hi]` and its (possibly wrapped-around) neighbor
/// by `f2/255`, per FastLED's `scale8_LEAVING_R1_DIRTY` pair-sum. On the
/// portable-C path that primitive is bit-identical to plain `scale8`; the
/// `_LEAVING_R1_DIRTY` naming only matters for an AVR register-zeroing step.
#[inline]
fn interpolate(entries: &[Crgb], hi: usize, f2: u8, blend: ColorBlend) -> Crgb {
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

/// `ColorFromPalette` against a 16-entry palette.
pub fn color_from_palette16(
    pal: &CrgbPalette16,
    mut index: u8,
    brightness: u8,
    blend: ColorBlend,
) -> Crgb {
    if blend == ColorBlend::LinearBlendNoWrap {
        index = scale8(index, Fract8(240));
    }

    let hi4 = (index >> 4) as usize;
    let lo4 = index & 0x0F;
    let f2 = lo4 << 4;

    let mut entry = interpolate(&pal.0, hi4, f2, blend);
    if brightness != 255 {
        entry.nscale8_video(brightness);
    }
    entry
}

/// `ColorFromPalette` against a 32-entry palette.
pub fn color_from_palette32(
    pal: &CrgbPalette32,
    mut index: u8,
    brightness: u8,
    blend: ColorBlend,
) -> Crgb {
    if blend == ColorBlend::LinearBlendNoWrap {
        index = scale8(index, Fract8(248));
    }

    let hi5 = (index >> 3) as usize;
    let lo3 = index & 0x07;
    let f2 = lo3 << 5;

    let mut entry = interpolate(&pal.0, hi5, f2, blend);
    if brightness != 255 {
        entry.nscale8_video(brightness);
    }
    entry
}

/// `ColorFromPalette` against a 256-entry palette. Every index maps to
/// exactly one entry, so there is nothing to interpolate between.
pub fn color_from_palette256(pal: &CrgbPalette256, index: u8, brightness: u8) -> Crgb {
    let mut entry = pal.0[index as usize];
    if brightness != 255 {
        entry.nscale8_video(brightness);
    }
    entry
}
