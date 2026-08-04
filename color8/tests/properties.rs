//! Property-based tests (via `proptest`).
//!
//! The exhaustive sweeps in `differential.rs` already pin every scalar
//! function to the C reference bit-for-bit, so there is nothing left for
//! random differential testing to find there. What these cover instead:
//!
//!   - *Array-shaped* functions (`fill_*`), whose input space is
//!     length × colors × direction rather than a bounded byte domain, and
//!     so cannot be swept exhaustively.
//!   - *Invariant* properties: algebraic facts that must hold of the Rust
//!     port by construction (saturation bounds, endpoint anchoring,
//!     idempotence). These catch bugs a transcription error could let slip
//!     through *both* sides of a differential check.

use color8::{
    Chsv, Crgb, GradientDirection, fill_gradient, fill_gradient_rgb, fill_rainbow,
    fill_rainbow_circular, fill_solid, hsv2rgb_rainbow, rgb2hsv_approximate,
};
use proptest::prelude::*;

fn any_crgb() -> impl Strategy<Value = Crgb> {
    (any::<u8>(), any::<u8>(), any::<u8>()).prop_map(|(r, g, b)| Crgb::new(r, g, b))
}

fn any_chsv() -> impl Strategy<Value = Chsv> {
    (any::<u8>(), any::<u8>(), any::<u8>()).prop_map(|(h, s, v)| Chsv::new(h, s, v))
}

fn any_direction() -> impl Strategy<Value = GradientDirection> {
    prop_oneof![
        Just(GradientDirection::Forward),
        Just(GradientDirection::Backward),
        Just(GradientDirection::Shortest),
        Just(GradientDirection::Longest),
    ]
}

// ---------------------------------------------------------------------------
// Fill functions — array-shaped, so not exhaustively sweepable
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fill_solid_writes_the_same_color_everywhere(color in any_crgb(), n in 0usize..500) {
        let mut leds = vec![Crgb::default(); n];
        fill_solid(&mut leds, color);
        prop_assert!(leds.iter().all(|&c| c == color));
    }

    #[test]
    fn fill_rainbow_steps_hue_by_delta(initial in any::<u8>(), delta in any::<u8>(), n in 0usize..500) {
        let mut leds = vec![Chsv::default(); n];
        fill_rainbow(&mut leds, initial, delta);

        for (i, led) in leds.iter().enumerate() {
            // Hue advances by `delta` per pixel, wrapping around the wheel.
            let expected_hue = initial.wrapping_add((delta as usize * i) as u8);
            prop_assert_eq!(led.hue, expected_hue, "pixel {}", i);
            prop_assert_eq!(led.sat, 240);
            prop_assert_eq!(led.val, 255);
        }
    }

    #[test]
    fn fill_rainbow_rgb_agrees_with_converting_the_hsv_fill(
        initial in any::<u8>(), delta in any::<u8>(), n in 0usize..300
    ) {
        // The CRGB overload must be exactly the CHSV one put through
        // hsv2rgb_rainbow -- that is what the C++ template does via CRGB's
        // implicit CHSV constructor.
        let mut as_hsv = vec![Chsv::default(); n];
        fill_rainbow(&mut as_hsv, initial, delta);

        let mut as_rgb = vec![Crgb::default(); n];
        fill_rainbow(&mut as_rgb, initial, delta);

        for (i, (h, r)) in as_hsv.iter().zip(as_rgb.iter()).enumerate() {
            prop_assert_eq!(hsv2rgb_rainbow(*h), *r, "pixel {}", i);
        }
    }

    #[test]
    fn fill_rainbow_circular_starts_at_the_initial_hue(
        initial in any::<u8>(), reversed in any::<bool>(), n in 1usize..500
    ) {
        let mut leds = vec![Chsv::default(); n];
        fill_rainbow_circular(&mut leds, initial, reversed);
        prop_assert_eq!(leds[0].hue, initial);
        prop_assert!(leds.iter().all(|c| c.sat == 240 && c.val == 255));
    }

    #[test]
    fn fill_gradient_rgb_anchors_its_start_and_never_overshoots(
        c1 in any_crgb(), c2 in any_crgb(), n in 2usize..500
    ) {
        let mut leds = vec![Crgb::default(); n];
        fill_gradient_rgb(&mut leds, c1, c2);

        // The first pixel is exactly the start color.
        prop_assert_eq!(leds[0], c1);

        // The last pixel is *not* generally c2. FastLED truncates the
        // per-pixel 8.7 fixed-point delta with integer division, so a long
        // ramp accumulates a shortfall and stops short of the end color --
        // e.g. blue 58 -> 73 over 322 pixels lands on 70, both here and in
        // the C. What must hold is that it never overshoots past c2 and
        // never falls outside the interval the ramp is walking.
        let last = leds[n - 1];
        for (got, from, to) in [
            (last.r, c1.r, c2.r),
            (last.g, c1.g, c2.g),
            (last.b, c1.b, c2.b),
        ] {
            let (lo, hi) = (from.min(to), from.max(to));
            prop_assert!(
                got >= lo && got <= hi,
                "endpoint {} escaped the ramp interval [{}, {}]", got, lo, hi
            );
        }
    }

    #[test]
    fn fill_gradient_rgb_is_monotonic_per_channel(c1 in any_crgb(), c2 in any_crgb(), n in 2usize..300) {
        let mut leds = vec![Crgb::default(); n];
        fill_gradient_rgb(&mut leds, c1, c2);

        // A two-stop linear ramp never reverses direction on any channel.
        for ch in 0..3 {
            let get = |c: &Crgb| match ch { 0 => c.r, 1 => c.g, _ => c.b };
            let ascending = get(&c2) >= get(&c1);
            for w in leds.windows(2) {
                let (a, b) = (get(&w[0]), get(&w[1]));
                if ascending {
                    prop_assert!(b >= a, "channel {} fell from {} to {}", ch, a, b);
                } else {
                    prop_assert!(b <= a, "channel {} rose from {} to {}", ch, a, b);
                }
            }
        }
    }

    #[test]
    fn fill_gradient_hsv_anchors_its_start(
        c1 in any_chsv(), c2 in any_chsv(), dir in any_direction(), n in 2usize..300
    ) {
        let mut leds = vec![Chsv::default(); n];
        fill_gradient(&mut leds, c1, c2, dir);

        // Saturation and value always start exactly at c1. Hue does too,
        // except when c1 is black or fully desaturated -- there the C
        // deliberately adopts c2's hue so the ramp stays smooth.
        prop_assert_eq!(leds[0].sat, c1.sat);
        prop_assert_eq!(leds[0].val, c1.val);
        if c1.val != 0 && c1.sat != 0 {
            prop_assert_eq!(leds[0].hue, c1.hue);
        }
    }

    #[test]
    fn fills_never_touch_memory_outside_the_slice(color in any_crgb(), n in 0usize..64) {
        // Every fill must write within bounds -- these would panic on an
        // out-of-range index rather than corrupting memory, since the port
        // is #![forbid(unsafe_code)], but the assertion documents intent.
        let mut leds = vec![Crgb::default(); n];
        fill_solid(&mut leds, color);
        fill_rainbow(&mut leds, 0, 7);
        fill_rainbow_circular(&mut leds, 0, false);
        fill_gradient_rgb(&mut leds, Crgb::new(0, 0, 0), color);
        prop_assert_eq!(leds.len(), n);
    }
}

// ---------------------------------------------------------------------------
// blend / nblend
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn blend_hits_both_endpoints_exactly(a in any_crgb(), b in any_crgb()) {
        // This is why color8 uses lib8tion's full-range blend8 rather than
        // its rounding one: at amount 255 the result must be exactly `b`.
        prop_assert_eq!(color8::blend(a, b, 0), a);
        prop_assert_eq!(color8::blend(a, b, 255), b);
    }

    #[test]
    fn blend_stays_between_its_endpoints(a in any_crgb(), b in any_crgb(), amount in any::<u8>()) {
        let r = color8::blend(a, b, amount);
        for (got, x, y) in [(r.r, a.r, b.r), (r.g, a.g, b.g), (r.b, a.b, b.b)] {
            prop_assert!(
                got >= x.min(y) && got <= x.max(y),
                "blend result {} escaped [{}, {}]", got, x.min(y), x.max(y)
            );
        }
    }

    #[test]
    fn blend_of_identical_colors_is_that_color(a in any_crgb(), amount in any::<u8>()) {
        prop_assert_eq!(color8::blend(a, a, amount), a);
    }

    #[test]
    fn nblend_agrees_with_blend(a in any_crgb(), b in any_crgb(), amount in any::<u8>()) {
        let mut mutated = a;
        color8::nblend(&mut mutated, b, amount);
        prop_assert_eq!(mutated, color8::blend(a, b, amount));
    }

    #[test]
    fn nblend_slice_is_pointwise_nblend(
        a in prop::collection::vec(any_crgb(), 0..64),
        b in prop::collection::vec(any_crgb(), 0..64),
        amount in any::<u8>(),
    ) {
        let mut slice = a.clone();
        color8::nblend_slice(&mut slice, &b, amount);

        // Only the overlapping prefix is touched; the tail is left alone.
        let overlap = a.len().min(b.len());
        for i in 0..overlap {
            prop_assert_eq!(slice[i], color8::blend(a[i], b[i], amount), "pixel {}", i);
        }
        for i in overlap..a.len() {
            prop_assert_eq!(slice[i], a[i], "untouched tail pixel {}", i);
        }
    }

    #[test]
    fn blend_hsv_hits_both_endpoints(a in any_chsv(), b in any_chsv(), dir in any_direction()) {
        prop_assert_eq!(color8::blend_hsv(a, b, 0, dir), a);
        prop_assert_eq!(color8::blend_hsv(a, b, 255, dir), b);
    }

    #[test]
    fn blend_hsv_forward_and_backward_agree_at_the_endpoints(a in any_chsv(), b in any_chsv()) {
        // Direction only decides which way around the wheel to travel, so
        // it cannot change where the travel starts or ends.
        for amount in [0u8, 255] {
            let f = color8::blend_hsv(a, b, amount, GradientDirection::Forward);
            let bw = color8::blend_hsv(a, b, amount, GradientDirection::Backward);
            prop_assert_eq!(f, bw, "amount {}", amount);
        }
    }

    #[test]
    fn fade_using_color_never_brightens(
        leds in prop::collection::vec(any_crgb(), 0..64),
        mask in any_crgb(),
    ) {
        let mut faded = leds.clone();
        color8::fade_using_color(&mut faded, mask);
        for (before, after) in leds.iter().zip(faded.iter()) {
            prop_assert!(after.r <= before.r && after.g <= before.g && after.b <= before.b);
        }
    }

    #[test]
    fn fade_using_color_with_white_mask_is_the_identity(
        leds in prop::collection::vec(any_crgb(), 0..64),
    ) {
        // scale8's full-scale factor is 255, so a white mask round-trips.
        let mut faded = leds.clone();
        color8::fade_using_color(&mut faded, Crgb::new(255, 255, 255));
        prop_assert_eq!(faded, leds);
    }
}

// ---------------------------------------------------------------------------
// Invariant properties — algebraic facts about the port itself
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn add_saturates_and_never_decreases_a_channel(a in any_crgb(), b in any_crgb()) {
        let r = a + b;
        prop_assert!(r.r >= a.r && r.g >= a.g && r.b >= a.b);
        prop_assert_eq!(r.r, a.r.saturating_add(b.r));
        prop_assert_eq!(r.g, a.g.saturating_add(b.g));
        prop_assert_eq!(r.b, a.b.saturating_add(b.b));
    }

    #[test]
    fn sub_saturates_and_never_increases_a_channel(a in any_crgb(), b in any_crgb()) {
        let r = a - b;
        prop_assert!(r.r <= a.r && r.g <= a.g && r.b <= a.b);
        prop_assert_eq!(r.r, a.r.saturating_sub(b.r));
        prop_assert_eq!(r.g, a.g.saturating_sub(b.g));
        prop_assert_eq!(r.b, a.b.saturating_sub(b.b));
    }

    #[test]
    fn bitand_and_bitor_bracket_their_inputs(a in any_crgb(), b in any_crgb()) {
        let lo = a & b;
        let hi = a | b;
        prop_assert_eq!(lo.r, a.r.min(b.r));
        prop_assert_eq!(hi.r, a.r.max(b.r));
        prop_assert!(lo.r <= hi.r && lo.g <= hi.g && lo.b <= hi.b);
    }

    #[test]
    fn neg_is_its_own_inverse(a in any_crgb()) {
        prop_assert_eq!(-(-a), a);
    }

    #[test]
    fn nscale8_by_255_is_the_identity(a in any_crgb()) {
        // scale8's fixed-point convention makes 255 the full-scale factor,
        // so scaling by it must round-trip exactly.
        let mut scaled = a;
        scaled.nscale8(255);
        prop_assert_eq!(scaled, a);
    }

    #[test]
    fn nscale8_by_zero_is_black(a in any_crgb()) {
        let mut scaled = a;
        scaled.nscale8(0);
        prop_assert!(scaled.is_black());
    }

    #[test]
    fn nscale8_video_keeps_lit_channels_lit(a in any_crgb(), scale in 1u8..=255) {
        // The defining property of "video" scaling: a nonzero channel never
        // dims all the way to zero unless the scale factor itself is zero.
        let mut scaled = a;
        scaled.nscale8_video(scale);
        prop_assert_eq!(scaled.r == 0, a.r == 0);
        prop_assert_eq!(scaled.g == 0, a.g == 0);
        prop_assert_eq!(scaled.b == 0, a.b == 0);
    }

    #[test]
    fn nscale8_never_brightens(a in any_crgb(), scale in any::<u8>()) {
        let mut scaled = a;
        scaled.nscale8(scale);
        prop_assert!(scaled.r <= a.r && scaled.g <= a.g && scaled.b <= a.b);
    }

    #[test]
    fn fade_to_black_by_255_reaches_black(a in any_crgb()) {
        let mut faded = a;
        faded.fade_to_black_by(255);
        prop_assert!(faded.is_black());
    }

    #[test]
    fn hsv_with_zero_value_renders_black(hue in any::<u8>(), sat in any::<u8>()) {
        prop_assert!(hsv2rgb_rainbow(Chsv::new(hue, sat, 0)).is_black());
    }

    #[test]
    fn hsv_with_zero_saturation_renders_gray(hue in any::<u8>(), val in any::<u8>()) {
        // Fully desaturated means all three channels agree, whatever the hue.
        let c = hsv2rgb_rainbow(Chsv::new(hue, 0, val));
        prop_assert_eq!(c.r, c.g);
        prop_assert_eq!(c.g, c.b);
    }

    #[test]
    fn rgb2hsv_reports_gray_as_unsaturated(v in any::<u8>()) {
        let hsv = rgb2hsv_approximate(Crgb::new(v, v, v));
        prop_assert_eq!(hsv.sat, 0, "gray {} reported saturation {}", v, hsv.sat);
    }

    #[test]
    fn rgb2hsv_never_panics(c in any_crgb()) {
        // The port does a lot of narrowing arithmetic that must wrap rather
        // than overflow-panic in debug builds.
        let _ = rgb2hsv_approximate(c);
    }
}
