//! `Crgb` — an 8-bit RGB pixel and its arithmetic/scaling operators.
//!
//! Direct port of the `CRGB` operators in FastLED's `src/pixeltypes.h`
//! (FastLED 3.6.0, the last release before the `fl::` namespace refactor).
//! Ordering operators (`<`, `>`, `<=`, `>=`), `getLuma`/`getAverageLight`,
//! `maximizeBrightness`, `lerp8`/`lerp16`, `getParity`/`setParity` and the
//! packed-`u32` color-code round trip are not yet ported.

use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Div, DivAssign, Mul, MulAssign, Neg,
    Rem, RemAssign, Shr, ShrAssign, Sub, SubAssign,
};

use lib8tion::{Fract8, qadd8, qmul8, qsub8};

/// An 8-bit RGB pixel.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Crgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Crgb {
    /// Builds a [`Crgb`] from its components.
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// `true` if all three channels are zero.
    #[inline]
    pub const fn is_black(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0
    }

    /// Adds a constant to each channel, saturating at `0xFF`.
    /// Not `AddAssign<u8>` because that overload can't be usefully
    /// disambiguated from `AddAssign<Crgb>` at the call site.
    #[inline]
    pub fn add_to_rgb(&mut self, d: u8) -> &mut Self {
        self.r = qadd8(self.r, d);
        self.g = qadd8(self.g, d);
        self.b = qadd8(self.b, d);
        self
    }

    /// Subtracts a constant from each channel, saturating at `0x00`.
    #[inline]
    pub fn subtract_from_rgb(&mut self, d: u8) -> &mut Self {
        self.r = qsub8(self.r, d);
        self.g = qsub8(self.g, d);
        self.b = qsub8(self.b, d);
        self
    }

    /// Scales down to `scaledown/256` of the current brightness using
    /// "plain math" dimming: low light levels may dim all the way to black.
    #[inline]
    pub fn nscale8(&mut self, scaledown: u8) -> &mut Self {
        lib8tion::nscale8x3(&mut self.r, &mut self.g, &mut self.b, Fract8(scaledown));
        self
    }

    /// Scales down to `scaledown/256` of the current brightness, "video"
    /// style: a nonzero channel never dims all the way to zero.
    #[inline]
    pub fn nscale8_video(&mut self, scaledown: u8) -> &mut Self {
        lib8tion::nscale8x3_video(&mut self.r, &mut self.g, &mut self.b, Fract8(scaledown));
        self
    }

    /// Scales each channel by the matching channel of `scaledown`.
    #[inline]
    pub fn nscale8_rgb(&mut self, scaledown: Crgb) -> &mut Self {
        self.r = lib8tion::scale8(self.r, Fract8(scaledown.r));
        self.g = lib8tion::scale8(self.g, Fract8(scaledown.g));
        self.b = lib8tion::scale8(self.b, Fract8(scaledown.b));
        self
    }

    /// Returns a copy of this pixel scaled down by `scaledown/256` ("plain
    /// math" dimming — see [`Crgb::nscale8`]).
    #[inline]
    pub fn scale8(&self, scaledown: u8) -> Crgb {
        let mut out = *self;
        out.nscale8(scaledown);
        out
    }

    /// Returns a copy of this pixel with each channel scaled by the
    /// matching channel of `scaledown`.
    #[inline]
    pub fn scale8_rgb(&self, scaledown: Crgb) -> Crgb {
        let mut out = *self;
        out.nscale8_rgb(scaledown);
        out
    }

    /// `fadeToBlackBy` — a synonym for [`Crgb::nscale8`] as a fade instead
    /// of a scale: `nscale8(255 - fadefactor)`.
    #[inline]
    pub fn fade_to_black_by(&mut self, fadefactor: u8) -> &mut Self {
        self.nscale8(255 - fadefactor)
    }

    /// `fadeLightBy` — a synonym for [`Crgb::nscale8_video`] as a fade
    /// instead of a scale: `nscale8_video(255 - fadefactor)`.
    #[inline]
    pub fn fade_light_by(&mut self, fadefactor: u8) -> &mut Self {
        self.nscale8_video(255 - fadefactor)
    }
}

impl Add for Crgb {
    type Output = Crgb;
    #[inline]
    fn add(self, rhs: Crgb) -> Crgb {
        Crgb::new(
            qadd8(self.r, rhs.r),
            qadd8(self.g, rhs.g),
            qadd8(self.b, rhs.b),
        )
    }
}

impl AddAssign for Crgb {
    #[inline]
    fn add_assign(&mut self, rhs: Crgb) {
        *self = *self + rhs;
    }
}

impl Sub for Crgb {
    type Output = Crgb;
    #[inline]
    fn sub(self, rhs: Crgb) -> Crgb {
        Crgb::new(
            qsub8(self.r, rhs.r),
            qsub8(self.g, rhs.g),
            qsub8(self.b, rhs.b),
        )
    }
}

impl SubAssign for Crgb {
    #[inline]
    fn sub_assign(&mut self, rhs: Crgb) {
        *self = *self - rhs;
    }
}

impl Mul<u8> for Crgb {
    type Output = Crgb;
    #[inline]
    fn mul(self, d: u8) -> Crgb {
        Crgb::new(qmul8(self.r, d), qmul8(self.g, d), qmul8(self.b, d))
    }
}

impl MulAssign<u8> for Crgb {
    #[inline]
    fn mul_assign(&mut self, d: u8) {
        *self = *self * d;
    }
}

impl Div<u8> for Crgb {
    type Output = Crgb;
    #[inline]
    fn div(self, d: u8) -> Crgb {
        Crgb::new(self.r / d, self.g / d, self.b / d)
    }
}

impl DivAssign<u8> for Crgb {
    #[inline]
    fn div_assign(&mut self, d: u8) {
        *self = *self / d;
    }
}

impl Shr<u8> for Crgb {
    type Output = Crgb;
    #[inline]
    fn shr(self, d: u8) -> Crgb {
        Crgb::new(self.r >> d, self.g >> d, self.b >> d)
    }
}

impl ShrAssign<u8> for Crgb {
    #[inline]
    fn shr_assign(&mut self, d: u8) {
        *self = *self >> d;
    }
}

/// Combines two pixels, taking the smaller value of each channel.
impl BitAnd for Crgb {
    type Output = Crgb;
    #[inline]
    fn bitand(self, rhs: Crgb) -> Crgb {
        Crgb::new(self.r.min(rhs.r), self.g.min(rhs.g), self.b.min(rhs.b))
    }
}

impl BitAndAssign for Crgb {
    #[inline]
    fn bitand_assign(&mut self, rhs: Crgb) {
        *self = *self & rhs;
    }
}

impl BitAndAssign<u8> for Crgb {
    #[inline]
    fn bitand_assign(&mut self, d: u8) {
        self.r = self.r.min(d);
        self.g = self.g.min(d);
        self.b = self.b.min(d);
    }
}

/// Combines two pixels, taking the larger value of each channel.
impl BitOr for Crgb {
    type Output = Crgb;
    #[inline]
    fn bitor(self, rhs: Crgb) -> Crgb {
        Crgb::new(self.r.max(rhs.r), self.g.max(rhs.g), self.b.max(rhs.b))
    }
}

impl BitOrAssign for Crgb {
    #[inline]
    fn bitor_assign(&mut self, rhs: Crgb) {
        *self = *self | rhs;
    }
}

impl BitOrAssign<u8> for Crgb {
    #[inline]
    fn bitor_assign(&mut self, d: u8) {
        self.r = self.r.max(d);
        self.g = self.g.max(d);
        self.b = self.b.max(d);
    }
}

/// `%` is a synonym for [`Crgb::nscale8_video`] — "scale down by a
/// percentage".
impl Rem<u8> for Crgb {
    type Output = Crgb;
    #[inline]
    fn rem(self, scaledown: u8) -> Crgb {
        let mut out = self;
        out.nscale8_video(scaledown);
        out
    }
}

impl RemAssign<u8> for Crgb {
    #[inline]
    fn rem_assign(&mut self, scaledown: u8) {
        self.nscale8_video(scaledown);
    }
}

/// Inverts each channel.
impl Neg for Crgb {
    type Output = Crgb;
    #[inline]
    fn neg(self) -> Crgb {
        Crgb::new(255 - self.r, 255 - self.g, 255 - self.b)
    }
}
