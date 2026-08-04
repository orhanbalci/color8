//! The `Chsv` pixel type and the HSV↔RGB conversions.
//!
//! Direct ports of FastLED's `src/hsv2rgb.cpp` (FastLED 3.6.0), the
//! portable-C path (`FASTLED_SCALE8_FIXED == 1`, non-AVR). The AVR assembly
//! fast paths and the `_LEAVING_R1_DIRTY` scale8 variants are not ported:
//! on the portable-C path they compute bit-identical results to plain
//! [`lib8tion::scale8`]/[`lib8tion::scale8_video`] — `_LEAVING_R1_DIRTY`
//! only skips an AVR register-zeroing instruction after the multiply, which
//! has no effect on the returned value.

use lib8tion::{Fract8, qadd8, qsub8, scale8, scale8_video, sqrt16};

use crate::rgb::Crgb;

/// An 8-bit HSV pixel: hue, saturation and value, each `0..=255`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Chsv {
    pub hue: u8,
    pub sat: u8,
    pub val: u8,
}

impl Chsv {
    /// Builds a [`Chsv`] from its components.
    #[inline]
    pub const fn new(hue: u8, sat: u8, val: u8) -> Self {
        Self { hue, sat, val }
    }

    /// Builds a fully-saturated, full-brightness [`Chsv`] from just a hue.
    #[inline]
    pub const fn from_hue(hue: u8) -> Self {
        Self::new(hue, 255, 255)
    }
}

impl From<Chsv> for Crgb {
    /// Converts via [`hsv2rgb_rainbow`], matching FastLED's implicit
    /// `CRGB(const CHSV&)` constructor.
    #[inline]
    fn from(hsv: Chsv) -> Crgb {
        hsv2rgb_rainbow(hsv)
    }
}

/// Pre-defined hue values for [`Chsv`], matching FastLED's `HSVHue` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HsvHue {
    /// Red (0°)
    Red = 0,
    /// Orange (45°)
    Orange = 32,
    /// Yellow (90°)
    Yellow = 64,
    /// Green (135°)
    Green = 96,
    /// Aqua (180°)
    Aqua = 128,
    /// Blue (225°)
    Blue = 160,
    /// Purple (270°)
    Purple = 192,
    /// Pink (315°)
    Pink = 224,
}

impl From<HsvHue> for u8 {
    #[inline]
    fn from(hue: HsvHue) -> u8 {
        hue as u8
    }
}

const K255: u8 = 255;
const K171: u8 = 171;
const K170: u8 = 170;
const K85: u8 = 85;

/// Converts an HSV pixel to RGB using FastLED's "rainbow" hue mapping —
/// visually even hue spacing, with yellow boosted so it doesn't look dim
/// relative to the other primaries.
pub fn hsv2rgb_rainbow(hsv: Chsv) -> Crgb {
    // Yellow has a higher inherent brightness than any other color; 'pure'
    // yellow is perceived to be 93% as bright as white. In order to make
    // yellow appear the correct relative brightness, it has to be rendered
    // brighter than all other colors.
    // Level Y1 is a moderate boost, the default. Level Y2 is a strong boost.
    const Y1: bool = true;
    const Y2: bool = false;

    // G2: whether to divide all greens by two. Gscale: what to scale green
    // down by. Both depend GREATLY on your particular LEDs.
    const G2: bool = false;
    const GSCALE: u8 = 0;

    let hue = hsv.hue;
    let sat = hsv.sat;
    let val = hsv.val;

    let offset = hue & 0x1F; // 0..31

    // offset8 = offset * 8
    let offset8 = offset << 3;

    let third = scale8(offset8, Fract8((256u16 / 3) as u8)); // max = 85

    let (mut r, mut g, mut b);

    if hue & 0x80 == 0 {
        // 0XX
        if hue & 0x40 == 0 {
            // 00X — section 0-1
            if hue & 0x20 == 0 {
                // 000 — case 0: R -> O
                r = K255 - third;
                g = third;
                b = 0;
            } else {
                // 001 — case 1: O -> Y
                if Y1 {
                    r = K171;
                    g = K85 + third;
                    b = 0;
                } else {
                    r = 0;
                    g = 0;
                    b = 0;
                }
                if Y2 {
                    let twothirds = scale8(offset8, Fract8(((256u16 * 2) / 3) as u8)); // max=170
                    r = K170 + third;
                    g = K85 + twothirds;
                    b = 0;
                }
            }
        } else {
            // 01X — section 2-3
            if hue & 0x20 == 0 {
                // 010 — case 2: Y -> G
                if Y1 {
                    let twothirds = scale8(offset8, Fract8(((256u16 * 2) / 3) as u8)); // max=170
                    r = K171 - twothirds;
                    g = K170 + third;
                    b = 0;
                } else {
                    r = 0;
                    g = 0;
                    b = 0;
                }
                if Y2 {
                    r = K255 - offset8;
                    g = K255;
                    b = 0;
                }
            } else {
                // 011 — case 3: G -> A
                r = 0;
                g = K255 - third;
                b = third;
            }
        }
    } else {
        // 1XX — section 4-7
        if hue & 0x40 == 0 {
            // 10X
            if hue & 0x20 == 0 {
                // 100 — case 4: A -> B
                let twothirds = scale8(offset8, Fract8(((256u16 * 2) / 3) as u8)); // max=170
                r = 0;
                g = K171 - twothirds;
                b = K85 + twothirds;
            } else {
                // 101 — case 5: B -> P
                r = third;
                g = 0;
                b = K255 - third;
            }
        } else if hue & 0x20 == 0 {
            // 110 — case 6: P -> K
            r = K85 + third;
            g = 0;
            b = K171 - third;
        } else {
            // 111 — case 7: K -> R
            r = K170 + third;
            g = 0;
            b = K85 - third;
        }
    }

    // This is one of the good places to scale the green down, although the
    // client can scale green down as well.
    if G2 {
        g >>= 1;
    }
    if GSCALE != 0 {
        g = scale8_video(g, Fract8(GSCALE));
    }

    // Scale down colors if we're desaturated at all, and add the
    // brightness_floor to r, g, and b.
    if sat != 255 {
        if sat == 0 {
            r = 255;
            g = 255;
            b = 255;
        } else {
            let mut desat = 255 - sat;
            desat = scale8_video(desat, Fract8(desat));

            let satscale = 255 - desat;

            r = scale8(r, Fract8(satscale));
            g = scale8(g, Fract8(satscale));
            b = scale8(b, Fract8(satscale));

            let brightness_floor = desat;
            r = r.wrapping_add(brightness_floor);
            g = g.wrapping_add(brightness_floor);
            b = b.wrapping_add(brightness_floor);
        }
    }

    // Now scale everything down if we're at value < 255.
    if val != 255 {
        let val = scale8_video(val, Fract8(val));
        if val == 0 {
            r = 0;
            g = 0;
            b = 0;
        } else {
            r = scale8(r, Fract8(val));
            g = scale8(g, Fract8(val));
            b = scale8(b, Fract8(val));
        }
    }

    Crgb::new(r, g, b)
}

/// The plain ramp-based HSV-to-RGB conversion (`hsv2rgb_raw_C` in FastLED's
/// `src/hsv2rgb.cpp`) — visually less even than [`hsv2rgb_rainbow`], but
/// simpler and slightly cheaper. Used internally by [`hsv2rgb_spectrum`].
fn hsv2rgb_raw_c(hue: u8, sat: u8, val: u8) -> Crgb {
    let value = val;
    let saturation = sat;

    let invsat = 255 - saturation;
    let brightness_floor = ((value as u16 * invsat as u16) / 256) as u8;

    let color_amplitude = value.wrapping_sub(brightness_floor);

    let section = hue / 0x40; // 0..2
    let offset = hue % 0x40; // 0..63

    let rampup = offset; // 0..63
    let rampdown = (0x40 - 1) - offset; // 63..0

    let rampup_amp_adj = ((rampup as u16 * color_amplitude as u16) / (256 / 4)) as u8;
    let rampdown_amp_adj = ((rampdown as u16 * color_amplitude as u16) / (256 / 4)) as u8;

    let rampup_adj_with_floor = rampup_amp_adj.wrapping_add(brightness_floor);
    let rampdown_adj_with_floor = rampdown_amp_adj.wrapping_add(brightness_floor);

    let (r, g, b);
    if section != 0 {
        if section == 1 {
            // section 1: 0x40..0x7F
            r = brightness_floor;
            g = rampdown_adj_with_floor;
            b = rampup_adj_with_floor;
        } else {
            // section 2: 0x80..0xBF
            r = rampup_adj_with_floor;
            g = brightness_floor;
            b = rampdown_adj_with_floor;
        }
    } else {
        // section 0: 0x00..0x3F
        r = rampdown_adj_with_floor;
        g = rampup_adj_with_floor;
        b = brightness_floor;
    }

    Crgb::new(r, g, b)
}

/// Converts an HSV pixel to RGB using FastLED's "spectrum" hue mapping:
/// evenly-spaced hue with no yellow boost, unlike [`hsv2rgb_rainbow`].
pub fn hsv2rgb_spectrum(hsv: Chsv) -> Crgb {
    let hue2 = scale8(hsv.hue, Fract8(191));
    hsv2rgb_raw_c(hue2, hsv.sat, hsv.val)
}

const HUE_RED: u8 = 0;
const HUE_ORANGE: u8 = 32;
const HUE_YELLOW: u8 = 64;
const HUE_GREEN: u8 = 96;
const HUE_AQUA: u8 = 128;
const HUE_BLUE: u8 = 160;
const HUE_PURPLE: u8 = 192;
const HUE_PINK: u8 = 224;

/// `scale8` by the fixed-point fraction `n/d`, computed at each call site
/// (mirrors FastLED's `FIXFRAC8(N, D)` macro: `((N) * 256) / (D)`).
const fn fixfrac8(n: u16, d: u16) -> u8 {
    ((n * 256) / d) as u8
}

/// Approximates the HSV value of an RGB pixel. This is the inverse of
/// [`hsv2rgb_rainbow`] only in the loosest sense — it's an approximation,
/// not an exact inverse, and considerably more expensive.
///
/// Ported as-is from FastLED 3.6.0's `rgb2hsv_approximate`, including its
/// known orange/yellow hue wraparound quirk (FastLED issue #436, fixed only
/// in later releases): this is a byte-exact port of that release, not a
/// corrected reimplementation.
pub fn rgb2hsv_approximate(rgb: Crgb) -> Chsv {
    let mut r = rgb.r;
    let mut g = rgb.g;
    let mut b = rgb.b;

    // find desaturation
    let mut desat = 255u8;
    if r < desat {
        desat = r;
    }
    if g < desat {
        desat = g;
    }
    if b < desat {
        desat = b;
    }

    // remove saturation from all channels
    r = r.wrapping_sub(desat);
    g = g.wrapping_sub(desat);
    b = b.wrapping_sub(desat);

    // saturation is opposite of desaturation
    let mut s = 255 - desat;

    if s != 255 {
        // undo 'dimming' of saturation
        s = 255 - sqrt16((255 - s) as u16 * 256);
    }

    // at least one channel is now zero; if all three are zero, this was a
    // shade of gray.
    if r as u16 + g as u16 + b as u16 == 0 {
        return Chsv::new(0, 0, 255 - s);
    }

    // scale all channels up to compensate for desaturation
    if s < 255 {
        if s == 0 {
            s = 1;
        }
        let scaleup = 65535u32 / s as u32;
        r = ((r as u32 * scaleup) / 256) as u8;
        g = ((g as u32 * scaleup) / 256) as u8;
        b = ((b as u32 * scaleup) / 256) as u8;
    }

    // `total` is mutated in place below (clamped to 1 if it was 0) exactly
    // as FastLED's C does — that clamped value, not the original, is what
    // the final `v` computation below uses.
    let mut total = r as u16 + g as u16 + b as u16;

    // scale all channels up to compensate for low values
    if total < 255 {
        if total == 0 {
            total = 1;
        }
        let scaleup = 65535u32 / total as u32;
        r = ((r as u32 * scaleup) / 256) as u8;
        g = ((g as u32 * scaleup) / 256) as u8;
        b = ((b as u32 * scaleup) / 256) as u8;
    }

    let v = if total > 255 {
        255
    } else {
        let mut v = qadd8(desat, total as u8);
        if v != 255 {
            v = sqrt16(v as u16 * 256);
        }
        v
    };

    // since this wasn't a pure shade of gray, the interesting question is
    // what hue it is. Start with which channel is highest (ties don't
    // matter).
    let mut highest = r;
    if g > highest {
        highest = g;
    }
    if b > highest {
        highest = b;
    }

    let mut h;
    if highest == r {
        // Red is highest. Hue could be Purple/Pink-Red, Red-Orange, Orange-Yellow.
        if g == 0 {
            h = (HUE_PURPLE as u16 + HUE_PINK as u16) / 2;
            h += scale8(qsub8(r, 128), Fract8(fixfrac8(48, 128))) as u16;
        } else if (r as i32 - g as i32) > g as i32 {
            h = HUE_RED as u16;
            h += scale8(g, Fract8(fixfrac8(32, 85))) as u16;
        } else {
            h = HUE_ORANGE as u16;
            // Transcribed as-is from FastLED 3.6.0: `(g - 85) + (171 - r)`
            // is computed in `int` then narrowed to `u8` when passed to
            // `qsub8` — this can wrap when `r > 171` or `g < 85` (FastLED
            // issue #436, fixed only in later releases). Kept intentionally
            // since this is a byte-exact port of 3.6.0's behavior.
            let inner = (g as i32 - 85) + (171 - r as i32);
            h += scale8(qsub8(inner as u8, 4), Fract8(fixfrac8(32, 85))) as u16;
        }
    } else if highest == g {
        // Green is highest. Hue could be Yellow-Green, Green-Aqua.
        if b == 0 {
            h = HUE_YELLOW as u16;
            let radj = scale8(qsub8(171, r), Fract8(47));
            let gadj = scale8(qsub8(g, 171), Fract8(96));
            let rgadj = radj.wrapping_add(gadj);
            let hueadv = rgadj / 2;
            h += hueadv as u16;
        } else if (g as i32 - b as i32) > b as i32 {
            h = HUE_GREEN as u16;
            h += scale8(b, Fract8(fixfrac8(32, 85))) as u16;
        } else {
            h = HUE_AQUA as u16;
            h += scale8(qsub8(b, 85), Fract8(fixfrac8(8, 42))) as u16;
        }
    } else {
        // Blue is highest. Hue could be Aqua/Blue-Blue, Blue-Purple, Purple-Pink.
        if r == 0 {
            h = HUE_AQUA as u16 + (HUE_BLUE as u16 - HUE_AQUA as u16) / 4;
            h += scale8(qsub8(b, 128), Fract8(fixfrac8(24, 128))) as u16;
        } else if (b as i32 - r as i32) > r as i32 {
            h = HUE_BLUE as u16;
            h += scale8(r, Fract8(fixfrac8(32, 85))) as u16;
        } else {
            h = HUE_PURPLE as u16;
            h += scale8(qsub8(r, 85), Fract8(fixfrac8(32, 85))) as u16;
        }
    }

    let h = (h as u8).wrapping_add(1);

    Chsv::new(h, s, v)
}
