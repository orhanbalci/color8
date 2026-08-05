//! Parsing FastLED's gradient-palette byte format into `CrgbPalette16`/`32`/
//! `256`, direct port of `CRGBPalette16`/`32`/`256`'s
//! `operator=(TProgmemRGBGradientPalette_bytes)` in `colorutils.h` (FastLED
//! 3.6.0), transcribed from the actual 3.6.0-tagged source.
//!
//! The format (as produced by FastLED's `DEFINE_GRADIENT_PALETTE` macro, or
//! any of the online gradient-palette generator tools that target it) is a
//! flat run of 4-byte stops: `index, r, g, b`, starting at `index == 0` and
//! ending with a stop whose `index == 255`. Between stops, colors are
//! linearly interpolated in RGB space — exactly [`fill_gradient_rgb_range`].
//! Bytes copied straight out of an existing FastLED sketch work as-is here.
//!
//! Only `CRGB` gradient palettes exist in FastLED — there is no `CHSV`
//! equivalent of this constructor.
//!
//! Squeezing an arbitrary-resolution gradient down into 16 or 32 slots
//! risks silently erasing a color that only exists for a stop or two (a
//! one-pixel-wide highlight, say). FastLED avoids that by tracking which
//! slot was last written (`lastSlotUsed`) and forcing each new segment to
//! start in a fresh slot when the gradient has fewer stops than the
//! destination has slots (`count < 16`, checked the same way regardless of
//! whether the destination is 16 or 32 entries — this project reproduces
//! that literally, quirky as the shared threshold is). Once a gradient has
//! 16 or more stops, that guarantee is dropped and slots may just overlap.
//! `CrgbPalette256` never needs any of this: with one slot per possible
//! index, nothing is ever lost to begin with.
//!
//! Malformed input (missing the `index == 255` terminator before the slice
//! runs out) is not undefined behavior here, unlike the C: parsing simply
//! stops, and slots that were never reached stay black. An empty or
//! too-short slice returns an all-black palette.

use crate::fill::fill_gradient_rgb_range;
use crate::palette::{CrgbPalette16, CrgbPalette32, CrgbPalette256};
use crate::rgb::Crgb;

const BLACK: Crgb = Crgb::new(0, 0, 0);

fn entry_at(bytes: &[u8], i: usize) -> Option<(u8, Crgb)> {
    let chunk = bytes.chunks_exact(4).nth(i)?;
    Some((chunk[0], Crgb::new(chunk[1], chunk[2], chunk[3])))
}

/// The number of stops up to and including the first `index == 255`
/// terminator, or every parseable stop if the slice never has one.
fn count_stops(bytes: &[u8]) -> usize {
    let total = bytes.len() / 4;
    for i in 0..total {
        if let Some((index, _)) = entry_at(bytes, i) {
            if index == 255 {
                return i + 1;
            }
        }
    }
    total
}

/// Shared body of the `CrgbPalette16`/`32` gradient parsers: `divisor` maps
/// a `0..=255` gradient index down to a slot (`16` for the 16-entry
/// palette, `8` for the 32-entry one — not `256 / N`, since that's what
/// FastLED's own source literally divides by), and `max_slot` is the
/// highest valid slot index (`15`/`31`).
fn build_compact<const N: usize>(bytes: &[u8], divisor: u16, max_slot: i16) -> [Crgb; N] {
    let mut entries = [BLACK; N];

    let Some((_, mut rgbstart)) = entry_at(bytes, 0) else {
        return entries;
    };

    let count = count_stops(bytes);
    let mut last_slot_used: i16 = -1;
    let mut index_start: u16 = 0;
    let mut i = 1;

    while index_start < 255 {
        let Some((index_end_byte, rgbend)) = entry_at(bytes, i) else {
            break;
        };
        let index_end = index_end_byte as u16;

        let mut istart8 = (index_start / divisor) as i16;
        let mut iend8 = (index_end / divisor) as i16;

        if count < 16 {
            if istart8 <= last_slot_used && last_slot_used < max_slot {
                istart8 = last_slot_used + 1;
                if iend8 < istart8 {
                    iend8 = istart8;
                }
            }
            last_slot_used = iend8;
        }

        fill_gradient_rgb_range(&mut entries, istart8 as u16, rgbstart, iend8 as u16, rgbend);

        index_start = index_end;
        rgbstart = rgbend;
        i += 1;
    }

    entries
}

/// Parses a [gradient-palette byte stream](self) into a 16-entry palette.
pub fn crgb_palette16_from_gradient(bytes: &[u8]) -> CrgbPalette16 {
    CrgbPalette16::new(build_compact::<16>(bytes, 16, 15))
}

/// Parses a [gradient-palette byte stream](self) into a 32-entry palette.
pub fn crgb_palette32_from_gradient(bytes: &[u8]) -> CrgbPalette32 {
    CrgbPalette32::new(build_compact::<32>(bytes, 8, 31))
}

/// Parses a [gradient-palette byte stream](self) into a 256-entry palette.
/// Every gradient index already has its own slot, so — unlike the 16- and
/// 32-entry versions — nothing needs to be compacted.
pub fn crgb_palette256_from_gradient(bytes: &[u8]) -> CrgbPalette256 {
    let mut entries = [BLACK; 256];

    if let Some((_, mut rgbstart)) = entry_at(bytes, 0) {
        let mut index_start: u16 = 0;
        let mut i = 1;

        while index_start < 255 {
            let Some((index_end_byte, rgbend)) = entry_at(bytes, i) else {
                break;
            };
            let index_end = index_end_byte as u16;

            fill_gradient_rgb_range(&mut entries, index_start, rgbstart, index_end, rgbend);

            index_start = index_end;
            rgbstart = rgbend;
            i += 1;
        }
    }

    CrgbPalette256::new(entries)
}
