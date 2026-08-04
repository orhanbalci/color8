//! Differential tests: assert that `color8`'s pure-Rust port produces
//! bit-for-bit identical output to FastLED's actual C reference
//! implementation (FastLED 3.6.0, compiled and linked via the
//! `fastled-ref` helper crate).
//!
//! The HSV conversions are swept exhaustively over all 2^24 inputs — the
//! domain is only 16.7M cases, which is entirely feasible in a
//! release-mode test, and this is where subtle wrongness hides.

use color8::{Chsv, Crgb, hsv2rgb_rainbow, hsv2rgb_spectrum, rgb2hsv_approximate};

// ---------------------------------------------------------------------------
// HSV <-> RGB conversions — exhaustive over the full 2^24 input domain
// ---------------------------------------------------------------------------

#[test]
fn hsv2rgb_rainbow_matches_reference_exhaustive() {
    for hue in 0..=255u8 {
        for sat in 0..=255u8 {
            for val in 0..=255u8 {
                let got = hsv2rgb_rainbow(Chsv::new(hue, sat, val));
                let want = fastled_ref::hsv2rgb_rainbow(hue, sat, val);
                assert_eq!(
                    (got.r, got.g, got.b),
                    want,
                    "hsv2rgb_rainbow(hue={hue},sat={sat},val={val})"
                );
            }
        }
    }
}

#[test]
fn hsv2rgb_spectrum_matches_reference_exhaustive() {
    for hue in 0..=255u8 {
        for sat in 0..=255u8 {
            for val in 0..=255u8 {
                let got = hsv2rgb_spectrum(Chsv::new(hue, sat, val));
                let want = fastled_ref::hsv2rgb_spectrum(hue, sat, val);
                assert_eq!(
                    (got.r, got.g, got.b),
                    want,
                    "hsv2rgb_spectrum(hue={hue},sat={sat},val={val})"
                );
            }
        }
    }
}

#[test]
fn rgb2hsv_approximate_matches_reference_exhaustive() {
    for r in 0..=255u8 {
        for g in 0..=255u8 {
            for b in 0..=255u8 {
                let got = rgb2hsv_approximate(Crgb::new(r, g, b));
                let want = fastled_ref::rgb2hsv_approximate(r, g, b);
                assert_eq!(
                    (got.hue, got.sat, got.val),
                    want,
                    "rgb2hsv_approximate(r={r},g={g},b={b})"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CRGB operators
// ---------------------------------------------------------------------------

/// Builds a color whose three channels are distinct bijective functions of
/// `x`. Sweeping `x` over `0..=255` therefore drives *each* channel through
/// every byte value, so a two-argument sweep over `(x, y)` covers every
/// `(lhs, rhs)` byte pair on every channel independently — while the
/// channels holding different values at each step means a channel mixup
/// (using `r` where `g` was meant) still shows up as a mismatch.
fn spread_a(x: u8) -> (u8, u8, u8) {
    (x, x ^ 0x5A, x.wrapping_add(77))
}

fn spread_b(y: u8) -> (u8, u8, u8) {
    (y, y ^ 0x33, y.wrapping_add(140))
}

fn crgb(t: (u8, u8, u8)) -> Crgb {
    Crgb::new(t.0, t.1, t.2)
}

fn tuple(c: Crgb) -> (u8, u8, u8) {
    (c.r, c.g, c.b)
}

#[test]
fn crgb_binary_operators_match_reference_exhaustive() {
    for x in 0..=255u8 {
        let a = spread_a(x);
        for y in 0..=255u8 {
            let b = spread_b(y);
            let (ca, cb) = (crgb(a), crgb(b));

            assert_eq!(
                tuple(ca + cb),
                fastled_ref::crgb_add(a, b),
                "Crgb::add({a:?},{b:?})"
            );
            assert_eq!(
                tuple(ca - cb),
                fastled_ref::crgb_sub(a, b),
                "Crgb::sub({a:?},{b:?})"
            );
            assert_eq!(
                tuple(ca & cb),
                fastled_ref::crgb_and(a, b),
                "Crgb::bitand({a:?},{b:?})"
            );
            assert_eq!(
                tuple(ca | cb),
                fastled_ref::crgb_or(a, b),
                "Crgb::bitor({a:?},{b:?})"
            );
            assert_eq!(
                tuple(ca.scale8_rgb(cb)),
                fastled_ref::crgb_nscale8_rgb(a, b),
                "Crgb::scale8_rgb({a:?},{b:?})"
            );
        }
    }
}

#[test]
fn crgb_scalar_operators_match_reference_exhaustive() {
    for x in 0..=255u8 {
        let a = spread_a(x);
        let ca = crgb(a);

        assert_eq!(tuple(-ca), fastled_ref::crgb_neg(a), "Crgb::neg({a:?})");

        for d in 0..=255u8 {
            assert_eq!(
                tuple(ca * d),
                fastled_ref::crgb_mul(a, d),
                "Crgb::mul({a:?},{d})"
            );

            // `operator/=` divides directly; d == 0 would be a division by
            // zero in the C too, so it is out of the defined domain.
            if d != 0 {
                assert_eq!(
                    tuple(ca / d),
                    fastled_ref::crgb_div(a, d),
                    "Crgb::div({a:?},{d})"
                );
            }

            let mut scaled = ca;
            scaled.nscale8(d);
            assert_eq!(
                tuple(scaled),
                fastled_ref::crgb_nscale8(a, d),
                "Crgb::nscale8({a:?},{d})"
            );

            let mut scaled_video = ca;
            scaled_video.nscale8_video(d);
            assert_eq!(
                tuple(scaled_video),
                fastled_ref::crgb_nscale8_video(a, d),
                "Crgb::nscale8_video({a:?},{d})"
            );

            // `%` is documented as a synonym for nscale8_video.
            assert_eq!(
                tuple(ca % d),
                fastled_ref::crgb_nscale8_video(a, d),
                "Crgb::rem({a:?},{d})"
            );

            let mut added = ca;
            added.add_to_rgb(d);
            assert_eq!(
                tuple(added),
                fastled_ref::crgb_add_to_rgb(a, d),
                "Crgb::add_to_rgb({a:?},{d})"
            );

            let mut subbed = ca;
            subbed.subtract_from_rgb(d);
            assert_eq!(
                tuple(subbed),
                fastled_ref::crgb_subtract_from_rgb(a, d),
                "Crgb::subtract_from_rgb({a:?},{d})"
            );

            // fadeToBlackBy / fadeLightBy are the same scalings, expressed
            // as a fade: nscale8(255 - d) / nscale8_video(255 - d).
            let mut faded = ca;
            faded.fade_to_black_by(d);
            assert_eq!(
                tuple(faded),
                fastled_ref::crgb_nscale8(a, 255 - d),
                "Crgb::fade_to_black_by({a:?},{d})"
            );

            let mut faded_light = ca;
            faded_light.fade_light_by(d);
            assert_eq!(
                tuple(faded_light),
                fastled_ref::crgb_nscale8_video(a, 255 - d),
                "Crgb::fade_light_by({a:?},{d})"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// fill_gradient_RGB
// ---------------------------------------------------------------------------

#[test]
fn fill_gradient_rgb_matches_reference() {
    let colors = [
        (0u8, 0u8, 0u8),
        (255, 255, 255),
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (1, 2, 3),
        (254, 253, 252),
        (128, 64, 192),
        (17, 200, 91),
        (0, 0, 58),
        (0, 0, 73),
    ];

    // 322 is here deliberately: it is the length at which a short ramp
    // (blue 58 -> 73) truncates its 8.7 fixed-point delta hard enough to
    // land 3 short of the end color. Both this port and the C stop at 70 --
    // pinning it keeps a future "fix" for the undershoot from silently
    // diverging from FastLED.
    for num_leds in [
        1u16, 2, 3, 4, 5, 7, 8, 16, 17, 60, 144, 255, 256, 300, 322, 1000,
    ] {
        for &c1 in &colors {
            for &c2 in &colors {
                let mut leds = vec![Crgb::default(); num_leds as usize];
                color8::fill_gradient_rgb(&mut leds, crgb(c1), crgb(c2));

                let want = fastled_ref::fill_gradient_rgb2(num_leds, c1, c2);
                let got: Vec<_> = leds.iter().copied().map(tuple).collect();

                assert_eq!(got, want, "fill_gradient_rgb(n={num_leds},{c1:?},{c2:?})");
            }
        }
    }
}
