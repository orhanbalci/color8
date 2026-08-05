// Reference implementation transcribed verbatim (portable-C path, non-AVR)
// from FastLED's colorutils sources, used ONLY to differentially test the
// `color8` Rust crate's port against FastLED's actual behavior.
//
// Sources — all from the FastLED 3.6.0 tag:
//   - src/hsv2rgb.cpp        (hsv2rgb_rainbow, hsv2rgb_raw_C, rgb2hsv_approximate)
//   - src/pixeltypes.h       (CRGB operators)
//   - src/colorutils.cpp     (fill_gradient_RGB, nblend, fadeUsingColor, HeatColor,
//                              ColorFromPalette — CRGB and CHSV overloads)
//   - src/colorutils.h       (CRGBPalette16/32/256's
//                              operator=(TProgmemRGBGradientPalette_bytes))
//   - src/lib8tion/scale8.h  (scale8, scale8_video — FASTLED_SCALE8_FIXED == 1)
//   - src/lib8tion/math8.h   (qadd8, qsub8, qmul8, blend8 — BLEND8_C branch)
//
// The AVR-only assembly fast paths and the `_LEAVING_R1_DIRTY` variants are
// skipped: on the portable-C path (FASTLED_SCALE8_FIXED == 1), those
// variants compute bit-identical results to plain scale8/scale8_video —
// `_LEAVING_R1_DIRTY` only skips an AVR register-zeroing instruction after
// the multiply, which has no effect on the returned value.

#include <stdint.h>

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;

// ---------------------------------------------------------------------------
// scale8 primitives — platforms/shared/scale8.h (FASTLED_SCALE8_FIXED == 1)
// ---------------------------------------------------------------------------

static u8 fl_scale8(u8 i, u8 scale) {
    return (u8)((((u16)i) * (1 + (u16)scale)) >> 8);
}

static u8 fl_scale8_video(u8 i, u8 scale) {
    u8 j = (u8)((((int)i * (int)scale) >> 8) + ((i && scale) ? 1 : 0));
    return j;
}

// ---------------------------------------------------------------------------
// hsv2rgb_rainbow — src/hsv2rgb.cpp.hpp
// ---------------------------------------------------------------------------

#define K255 255
#define K171 171
#define K170 170
#define K85  85

void fl_hsv2rgb_rainbow(u8 hue, u8 sat, u8 val, u8 *out_r, u8 *out_g, u8 *out_b) {
    // Yellow has a higher inherent brightness than any other color; 'pure'
    // yellow is perceived to be 93% as bright as white. In order to make
    // yellow appear the correct relative brightness, it has to be rendered
    // brighter than all other colors.
    // Level Y1 is a moderate boost, the default. Level Y2 is a strong boost.
    const u8 Y1 = 1;
    const u8 Y2 = 0;

    // G2: whether to divide all greens by two. Gscale: what to scale green
    // down by. Both depend GREATLY on your particular LEDs.
    const u8 G2 = 0;
    const u8 Gscale = 0;

    u8 offset = hue & 0x1F; // 0..31

    // offset8 = offset * 8
    u8 offset8 = offset;
    offset8 = (u8)(offset8 << 3);

    u8 third = fl_scale8(offset8, (256 / 3)); // max = 85

    u8 r, g, b;

    if (!(hue & 0x80)) {
        // 0XX
        if (!(hue & 0x40)) {
            // 00X — section 0-1
            if (!(hue & 0x20)) {
                // 000 — case 0: R -> O
                r = K255 - third;
                g = third;
                b = 0;
            } else {
                // 001 — case 1: O -> Y
                if (Y1) {
                    r = K171;
                    g = K85 + third;
                    b = 0;
                }
                if (Y2) {
                    r = K170 + third;
                    u8 twothirds = fl_scale8(offset8, ((256 * 2) / 3)); // max=170
                    g = K85 + twothirds;
                    b = 0;
                }
            }
        } else {
            // 01X — section 2-3
            if (!(hue & 0x20)) {
                // 010 — case 2: Y -> G
                if (Y1) {
                    u8 twothirds = fl_scale8(offset8, ((256 * 2) / 3)); // max=170
                    r = K171 - twothirds;
                    g = K170 + third;
                    b = 0;
                }
                if (Y2) {
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
        if (!(hue & 0x40)) {
            // 10X
            if (!(hue & 0x20)) {
                // 100 — case 4: A -> B
                r = 0;
                u8 twothirds = fl_scale8(offset8, ((256 * 2) / 3)); // max=170
                g = K171 - twothirds;
                b = K85 + twothirds;
            } else {
                // 101 — case 5: B -> P
                r = third;
                g = 0;
                b = K255 - third;
            }
        } else {
            if (!(hue & 0x20)) {
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
    }

    // This is one of the good places to scale the green down, although the
    // client can scale green down as well.
    if (G2) g = (u8)(g >> 1);
    if (Gscale) g = fl_scale8_video(g, Gscale);

    // Scale down colors if we're desaturated at all, and add the
    // brightness_floor to r, g, and b.
    if (sat != 255) {
        if (sat == 0) {
            r = 255;
            b = 255;
            g = 255;
        } else {
            u8 desat = (u8)(255 - sat);
            desat = fl_scale8_video(desat, desat);

            u8 satscale = (u8)(255 - desat);

            r = fl_scale8(r, satscale);
            g = fl_scale8(g, satscale);
            b = fl_scale8(b, satscale);

            u8 brightness_floor = desat;
            r = (u8)(r + brightness_floor);
            g = (u8)(g + brightness_floor);
            b = (u8)(b + brightness_floor);
        }
    }

    // Now scale everything down if we're at value < 255.
    if (val != 255) {
        val = fl_scale8_video(val, val);
        if (val == 0) {
            r = 0;
            g = 0;
            b = 0;
        } else {
            r = fl_scale8(r, val);
            g = fl_scale8(g, val);
            b = fl_scale8(b, val);
        }
    }

    *out_r = r;
    *out_g = g;
    *out_b = b;
}

// ---------------------------------------------------------------------------
// hsv2rgb_raw_C / hsv2rgb_spectrum — src/hsv2rgb.cpp (FastLED 3.6.0)
// ---------------------------------------------------------------------------

static void fl_hsv2rgb_raw_c(u8 hue, u8 sat, u8 val, u8 *out_r, u8 *out_g, u8 *out_b) {
    u8 value = val;
    u8 saturation = sat;

    u8 invsat = (u8)(255 - saturation);
    u8 brightness_floor = (u8)(((u16)value * (u16)invsat) / 256);

    u8 color_amplitude = (u8)(value - brightness_floor);

    u8 section = (u8)(hue / 0x40); // 0..2
    u8 offset = (u8)(hue % 0x40);  // 0..63

    u8 rampup = offset;
    u8 rampdown = (u8)((0x40 - 1) - offset);

    u8 rampup_amp_adj = (u8)(((u16)rampup * (u16)color_amplitude) / (256 / 4));
    u8 rampdown_amp_adj = (u8)(((u16)rampdown * (u16)color_amplitude) / (256 / 4));

    u8 rampup_adj_with_floor = (u8)(rampup_amp_adj + brightness_floor);
    u8 rampdown_adj_with_floor = (u8)(rampdown_amp_adj + brightness_floor);

    u8 r, g, b;
    if (section) {
        if (section == 1) {
            r = brightness_floor;
            g = rampdown_adj_with_floor;
            b = rampup_adj_with_floor;
        } else {
            r = rampup_adj_with_floor;
            g = brightness_floor;
            b = rampdown_adj_with_floor;
        }
    } else {
        r = rampdown_adj_with_floor;
        g = rampup_adj_with_floor;
        b = brightness_floor;
    }

    *out_r = r;
    *out_g = g;
    *out_b = b;
}

void fl_hsv2rgb_spectrum(u8 hue, u8 sat, u8 val, u8 *out_r, u8 *out_g, u8 *out_b) {
    u8 hue2 = fl_scale8(hue, 191);
    fl_hsv2rgb_raw_c(hue2, sat, val, out_r, out_g, out_b);
}

// ---------------------------------------------------------------------------
// rgb2hsv_approximate — src/hsv2rgb.cpp (FastLED 3.6.0, transcribed as-is —
// this version has the known FastLED#436 orange/yellow wraparound quirk,
// kept intentionally since this is a byte-exact port of that release)
// ---------------------------------------------------------------------------

static u8 fl_qadd8(u8 i, u8 j) {
    u32 t = i + j;
    if (t > 255) t = 255;
    return (u8)t;
}

static u8 fl_qsub8(u8 i, u8 j) {
    int t = i - j;
    if (t < 0) t = 0;
    return (u8)t;
}

// sqrt16 — src/lib8tion.h (platforms/math8.h). Needed by rgb2hsv_approximate.
static u8 fl_sqrt16(u16 x) {
    if (x <= 1) return (u8)x;

    u8 low = 1;
    u8 hi, mid;

    if (x > 7904) {
        hi = 255;
    } else {
        hi = (u8)((x >> 5) + 8);
    }

    do {
        mid = (u8)((low + hi) >> 1);
        if ((u16)(mid * mid) > x) {
            hi = (u8)(mid - 1);
        } else {
            if (mid == 255) return 255;
            low = (u8)(mid + 1);
        }
    } while (hi >= low);

    return (u8)(low - 1);
}

#define HUE_RED    0
#define HUE_ORANGE 32
#define HUE_YELLOW 64
#define HUE_GREEN  96
#define HUE_AQUA   128
#define HUE_BLUE   160
#define HUE_PURPLE 192
#define HUE_PINK   224

#define FIXFRAC8(N, D) (((N) * 256) / (D))

void fl_rgb2hsv_approximate(u8 in_r, u8 in_g, u8 in_b, u8 *out_h, u8 *out_s, u8 *out_v) {
    u8 r = in_r;
    u8 g = in_g;
    u8 b = in_b;
    u8 h, s, v;

    u8 desat = 255;
    if (r < desat) desat = r;
    if (g < desat) desat = g;
    if (b < desat) desat = b;

    r = (u8)(r - desat);
    g = (u8)(g - desat);
    b = (u8)(b - desat);

    s = (u8)(255 - desat);

    if (s != 255) {
        s = (u8)(255 - fl_sqrt16((u16)((255 - s) * 256)));
    }

    if ((u16)((u16)r + (u16)g + (u16)b) == 0) {
        *out_h = 0;
        *out_s = 0;
        *out_v = (u8)(255 - s);
        return;
    }

    if (s < 255) {
        if (s == 0) s = 1;
        u32 scaleup = 65535u / (u32)s;
        r = (u8)(((u32)r * scaleup) / 256);
        g = (u8)(((u32)g * scaleup) / 256);
        b = (u8)(((u32)b * scaleup) / 256);
    }

    u16 total = (u16)((u16)r + (u16)g + (u16)b);

    if (total < 255) {
        if (total == 0) total = 1;
        u32 scaleup = 65535u / (u32)total;
        r = (u8)(((u32)r * scaleup) / 256);
        g = (u8)(((u32)g * scaleup) / 256);
        b = (u8)(((u32)b * scaleup) / 256);
    }

    if (total > 255) {
        v = 255;
    } else {
        v = fl_qadd8(desat, (u8)total);
        if (v != 255) v = fl_sqrt16((u16)((u16)v * 256));
    }

    u8 highest = r;
    if (g > highest) highest = g;
    if (b > highest) highest = b;

    if (highest == r) {
        if (g == 0) {
            h = (u8)((HUE_PURPLE + HUE_PINK) / 2);
            h = (u8)(h + fl_scale8(fl_qsub8(r, 128), FIXFRAC8(48, 128)));
        } else if ((u8)(r - g) > g) {
            h = HUE_RED;
            h = (u8)(h + fl_scale8(g, FIXFRAC8(32, 85)));
        } else {
            h = HUE_ORANGE;
            // Transcribed as-is from FastLED 3.6.0: `(g - 85) + (171 - r)` is
            // computed in `int` (both operands promote from uint8_t) then
            // narrowed to uint8_t when passed to qsub8 — this narrowing can
            // wrap when r > 171 or g < 85, which is FastLED issue #436,
            // fixed only in later releases. Kept here intentionally: this
            // shim targets 3.6.0's exact byte-for-byte behavior.
            int inner = (int)((int)g - 85) + (int)(171 - (int)r);
            h = (u8)(h + fl_scale8(fl_qsub8((u8)inner, 4), FIXFRAC8(32, 85)));
        }
    } else if (highest == g) {
        if (b == 0) {
            h = HUE_YELLOW;
            u8 radj = fl_scale8(fl_qsub8(171, r), 47);
            u8 gadj = fl_scale8(fl_qsub8(g, 171), 96);
            u8 rgadj = (u8)(radj + gadj);
            u8 hueadv = (u8)(rgadj / 2);
            h = (u8)(h + hueadv);
        } else {
            if ((u8)(g - b) > b) {
                h = HUE_GREEN;
                h = (u8)(h + fl_scale8(b, FIXFRAC8(32, 85)));
            } else {
                h = HUE_AQUA;
                h = (u8)(h + fl_scale8(fl_qsub8(b, 85), FIXFRAC8(8, 42)));
            }
        }
    } else {
        if (r == 0) {
            h = (u8)(HUE_AQUA + ((HUE_BLUE - HUE_AQUA) / 4));
            h = (u8)(h + fl_scale8(fl_qsub8(b, 128), FIXFRAC8(24, 128)));
        } else if ((u8)(b - r) > r) {
            h = HUE_BLUE;
            h = (u8)(h + fl_scale8(r, FIXFRAC8(32, 85)));
        } else {
            h = HUE_PURPLE;
            h = (u8)(h + fl_scale8(fl_qsub8(r, 85), FIXFRAC8(32, 85)));
        }
    }

    h = (u8)(h + 1);

    *out_h = h;
    *out_s = s;
    *out_v = v;
}

// ---------------------------------------------------------------------------
// CRGB operators — src/pixeltypes.h (FastLED 3.6.0)
// ---------------------------------------------------------------------------

static u8 fl_qmul8(u8 i, u8 j) {
    u32 p = (u32)i * (u32)j;
    if (p > 255) p = 255;
    return (u8)p;
}

void fl_crgb_add(u8 r1, u8 g1, u8 b1, u8 r2, u8 g2, u8 b2, u8 *out_r, u8 *out_g, u8 *out_b) {
    *out_r = fl_qadd8(r1, r2);
    *out_g = fl_qadd8(g1, g2);
    *out_b = fl_qadd8(b1, b2);
}

void fl_crgb_sub(u8 r1, u8 g1, u8 b1, u8 r2, u8 g2, u8 b2, u8 *out_r, u8 *out_g, u8 *out_b) {
    *out_r = fl_qsub8(r1, r2);
    *out_g = fl_qsub8(g1, g2);
    *out_b = fl_qsub8(b1, b2);
}

void fl_crgb_mul(u8 r, u8 g, u8 b, u8 d, u8 *out_r, u8 *out_g, u8 *out_b) {
    *out_r = fl_qmul8(r, d);
    *out_g = fl_qmul8(g, d);
    *out_b = fl_qmul8(b, d);
}

void fl_crgb_div(u8 r, u8 g, u8 b, u8 d, u8 *out_r, u8 *out_g, u8 *out_b) {
    *out_r = (u8)(r / d);
    *out_g = (u8)(g / d);
    *out_b = (u8)(b / d);
}

void fl_crgb_and(u8 r1, u8 g1, u8 b1, u8 r2, u8 g2, u8 b2, u8 *out_r, u8 *out_g, u8 *out_b) {
    *out_r = r1 < r2 ? r1 : r2;
    *out_g = g1 < g2 ? g1 : g2;
    *out_b = b1 < b2 ? b1 : b2;
}

void fl_crgb_or(u8 r1, u8 g1, u8 b1, u8 r2, u8 g2, u8 b2, u8 *out_r, u8 *out_g, u8 *out_b) {
    *out_r = r1 > r2 ? r1 : r2;
    *out_g = g1 > g2 ? g1 : g2;
    *out_b = b1 > b2 ? b1 : b2;
}

void fl_crgb_neg(u8 r, u8 g, u8 b, u8 *out_r, u8 *out_g, u8 *out_b) {
    *out_r = (u8)(255 - r);
    *out_g = (u8)(255 - g);
    *out_b = (u8)(255 - b);
}

void fl_crgb_add_to_rgb(u8 r, u8 g, u8 b, u8 d, u8 *out_r, u8 *out_g, u8 *out_b) {
    *out_r = fl_qadd8(r, d);
    *out_g = fl_qadd8(g, d);
    *out_b = fl_qadd8(b, d);
}

void fl_crgb_subtract_from_rgb(u8 r, u8 g, u8 b, u8 d, u8 *out_r, u8 *out_g, u8 *out_b) {
    *out_r = fl_qsub8(r, d);
    *out_g = fl_qsub8(g, d);
    *out_b = fl_qsub8(b, d);
}

void fl_crgb_nscale8(u8 r, u8 g, u8 b, u8 scale, u8 *out_r, u8 *out_g, u8 *out_b) {
    *out_r = fl_scale8(r, scale);
    *out_g = fl_scale8(g, scale);
    *out_b = fl_scale8(b, scale);
}

void fl_crgb_nscale8_video(u8 r, u8 g, u8 b, u8 scale, u8 *out_r, u8 *out_g, u8 *out_b) {
    *out_r = fl_scale8_video(r, scale);
    *out_g = fl_scale8_video(g, scale);
    *out_b = fl_scale8_video(b, scale);
}

void fl_crgb_nscale8_rgb(u8 r, u8 g, u8 b, u8 sr, u8 sg, u8 sb, u8 *out_r, u8 *out_g, u8 *out_b) {
    *out_r = fl_scale8(r, sr);
    *out_g = fl_scale8(g, sg);
    *out_b = fl_scale8(b, sb);
}

// ---------------------------------------------------------------------------
// fill_gradient_RGB core (2-stop) — src/colorutils.cpp (FastLED 3.6.0)
// ---------------------------------------------------------------------------

void fl_fill_gradient_rgb2(u16 num_leds, u8 r1, u8 g1, u8 b1, u8 r2, u8 g2, u8 b2,
                            u8 *out_r, u8 *out_g, u8 *out_b) {
    if (num_leds == 0) return;
    u16 startpos = 0;
    u16 endpos = (u16)(num_leds - 1);

    int16_t rdistance87 = (int16_t)(((int16_t)r2 - (int16_t)r1) << 7);
    int16_t gdistance87 = (int16_t)(((int16_t)g2 - (int16_t)g1) << 7);
    int16_t bdistance87 = (int16_t)(((int16_t)b2 - (int16_t)b1) << 7);

    u16 pixeldistance = (u16)(endpos - startpos);
    int16_t divisor = pixeldistance ? (int16_t)pixeldistance : 1;

    int16_t rdelta87 = (int16_t)(rdistance87 / divisor);
    int16_t gdelta87 = (int16_t)(gdistance87 / divisor);
    int16_t bdelta87 = (int16_t)(bdistance87 / divisor);

    rdelta87 = (int16_t)(rdelta87 * 2);
    gdelta87 = (int16_t)(gdelta87 * 2);
    bdelta87 = (int16_t)(bdelta87 * 2);

    int32_t r88 = (int32_t)r1 << 8;
    int32_t g88 = (int32_t)g1 << 8;
    int32_t b88 = (int32_t)b1 << 8;

    for (u16 i = startpos; i <= endpos; ++i) {
        out_r[i] = (u8)(r88 >> 8);
        out_g[i] = (u8)(g88 >> 8);
        out_b[i] = (u8)(b88 >> 8);
        r88 += rdelta87;
        g88 += gdelta87;
        b88 += bdelta87;
        if (i == endpos) break; // guard against u16 wraparound when endpos == 65535
    }
}

// ---------------------------------------------------------------------------
// blend8 — src/lib8tion/math8.h (BLEND8_C == 1, FASTLED_SCALE8_FIXED == 1)
//
// NOTE: this is NOT the same formula as the `blend8` in current FastLED
// master (which the `lib8tion` Rust crate ports). 3.6.0 seeds `partial`
// with `(a << 8) | b` and does no rounding add; master's 8-bit variant
// seeds with `a << 8` and adds 0x80 before shifting. They disagree — e.g.
// blend8(0, 255, 255) is 255 here and 254 there. `nblend` sits directly on
// this, so color8 carries its own transcription rather than reusing
// lib8tion::blend8.
// ---------------------------------------------------------------------------

static u8 fl_blend8(u8 a, u8 b, u8 amountOfB) {
    u16 partial;
    u8 result;

    partial = (u16)((a << 8) | b); // A*256 + B

    partial = (u16)(partial + (u16)(b * amountOfB));
    partial = (u16)(partial - (u16)(a * amountOfB));

    result = (u8)(partial >> 8);

    return result;
}

u8 fl_blend8_360(u8 a, u8 b, u8 amount_of_b) { return fl_blend8(a, b, amount_of_b); }

// ---------------------------------------------------------------------------
// nblend / blend — src/colorutils.cpp (FastLED 3.6.0)
// ---------------------------------------------------------------------------

void fl_nblend_rgb(u8 er, u8 eg, u8 eb, u8 or_, u8 og, u8 ob, u8 amount_of_overlay,
                   u8 *out_r, u8 *out_g, u8 *out_b) {
    if (amount_of_overlay == 0) {
        *out_r = er;
        *out_g = eg;
        *out_b = eb;
        return;
    }

    if (amount_of_overlay == 255) {
        *out_r = or_;
        *out_g = og;
        *out_b = ob;
        return;
    }

    *out_r = fl_blend8(er, or_, amount_of_overlay);
    *out_g = fl_blend8(eg, og, amount_of_overlay);
    *out_b = fl_blend8(eb, ob, amount_of_overlay);
}

// direction: 0 = FORWARD_HUES, 1 = BACKWARD_HUES, 2 = SHORTEST_HUES,
//            3 = LONGEST_HUES (matching the TGradientDirectionCode enum order)
void fl_nblend_hsv(u8 eh, u8 es, u8 ev, u8 oh, u8 os, u8 ov, u8 amount_of_overlay,
                   int direction, u8 *out_h, u8 *out_s, u8 *out_v) {
    if (amount_of_overlay == 0) {
        *out_h = eh;
        *out_s = es;
        *out_v = ev;
        return;
    }

    if (amount_of_overlay == 255) {
        *out_h = oh;
        *out_s = os;
        *out_v = ov;
        return;
    }

    u8 amount_of_keep = (u8)(255 - amount_of_overlay);

    u8 huedelta8 = (u8)(oh - eh);

    if (direction == 2 /* SHORTEST_HUES */) {
        direction = 0;
        if (huedelta8 > 127) {
            direction = 1;
        }
    }

    if (direction == 3 /* LONGEST_HUES */) {
        direction = 0;
        if (huedelta8 < 128) {
            direction = 1;
        }
    }

    u8 hue;
    if (direction == 0 /* FORWARD_HUES */) {
        hue = (u8)(eh + fl_scale8(huedelta8, amount_of_overlay));
    } else /* BACKWARD_HUES */ {
        huedelta8 = (u8)(-huedelta8);
        hue = (u8)(eh - fl_scale8(huedelta8, amount_of_overlay));
    }

    // Both terms are uint8_t and the sum is assigned back to a uint8_t
    // member, so it truncates rather than saturating.
    u8 sat = (u8)(fl_scale8(es, amount_of_keep) + fl_scale8(os, amount_of_overlay));
    u8 val = (u8)(fl_scale8(ev, amount_of_keep) + fl_scale8(ov, amount_of_overlay));

    *out_h = hue;
    *out_s = sat;
    *out_v = val;
}

// ---------------------------------------------------------------------------
// fadeUsingColor — src/colorutils.cpp (FastLED 3.6.0)
// ---------------------------------------------------------------------------

void fl_fade_using_color(u8 r, u8 g, u8 b, u8 fr, u8 fg, u8 fb, u8 *out_r, u8 *out_g,
                         u8 *out_b) {
    *out_r = fl_scale8(r, fr);
    *out_g = fl_scale8(g, fg);
    *out_b = fl_scale8(b, fb);
}

// ---------------------------------------------------------------------------
// HeatColor — src/colorutils.cpp (FastLED 3.6.0)
// ---------------------------------------------------------------------------

void fl_heat_color(u8 temperature, u8 *out_r, u8 *out_g, u8 *out_b) {
    // Scale 'heat' down from 0-255 to 0-191, which can then be easily
    // divided into three equal 'thirds' of 64 units each.
    u8 t192 = fl_scale8_video(temperature, 191);

    // calculate a value that ramps up from zero to 255 in each 'third' of
    // the scale.
    u8 heatramp = (u8)(t192 & 0x3F); // 0..63
    heatramp = (u8)(heatramp << 2);  // scale up to 0..252

    // now figure out which third of the spectrum we're in:
    if (t192 & 0x80) {
        // we're in the hottest third
        *out_r = 255;    // full red
        *out_g = 255;    // full green
        *out_b = heatramp; // ramp up blue
    } else if (t192 & 0x40) {
        // we're in the middle third
        *out_r = 255;     // full red
        *out_g = heatramp; // ramp up green
        *out_b = 0;        // no blue
    } else {
        // we're in the coolest third
        *out_r = heatramp; // ramp up red
        *out_g = 0;         // no green
        *out_b = 0;         // no blue
    }
}

// ---------------------------------------------------------------------------
// ColorFromPalette — src/colorutils.cpp (FastLED 3.6.0), transcribed from
// the actual 3.6.0-tagged source (github.com/FastLED/FastLED, tag 3.6.0),
// RAM-palette (CRGBPalette16/32/256, CHSVPalette16/32/256) overloads only —
// not the PROGMEM (TProgmemRGBPalette*) ones, which color8 has no analogue
// of.
//
// blend: 0 = NOBLEND, 1 = LINEARBLEND, 2 = LINEARBLEND_NOWRAP (this repo's
// own numbering for the TBlendType parameter, not FastLED's enum values —
// same convention as fl_nblend_hsv's `direction` parameter above).
//
// scale8_LEAVING_R1_DIRTY is transcribed as plain fl_scale8, and
// scale8_video_LEAVING_R1_DIRTY as fl_scale8_video: on the portable-C path
// they compute identical values to the non-DIRTY versions: the AVR variant
// only skips a register-zeroing instruction after the multiply (see the
// file header).
//
// LINEARBLEND_NOWRAP rescales the index via `map8(index, 0, N)`, which is
// `scale8(index, N)` (map8's rangeStart is 0, so it contributes nothing) —
// N is 239 for the 16-entry palettes and 247 for the 32-entry ones, *not*
// 240/248: map8's `rangeWidth = rangeEnd - rangeStart` is exclusive of the
// endpoint that scale8's `+1` in the fixed-point formula would otherwise
// reach.
//
// Brightness handling differs by palette size and color space, and is
// transcribed exactly rather than unified:
//   - CRGBPalette16/32: brightness == 0 forces black; otherwise plain
//     scale8(x, brightness + 1) (not scale8_video).
//   - CRGBPalette256: scale8_video(x, brightness + 1) — no brightness == 0
//     special case, unlike the 16/32-entry versions.
//   - CHSVPalette16/32/256: only `val` is brightness-scaled, via plain
//     scale8_video(val, brightness) — no "+1" rounding adjustment, unlike
//     the CRGB versions.
// ---------------------------------------------------------------------------

void fl_color_from_palette16(const u8 *pr, const u8 *pg, const u8 *pb, u8 index,
                              u8 brightness, int blend, u8 *out_r, u8 *out_g, u8 *out_b) {
    if (blend == 2) {
        index = fl_scale8(index, 239);
    }
    u8 hi4 = (u8)(index >> 4);
    u8 lo4 = (u8)(index & 0x0F);

    u8 r1 = pr[hi4], g1 = pg[hi4], b1 = pb[hi4];

    if (blend != 0 && lo4) {
        u8 next = (hi4 == 15) ? 0 : (u8)(hi4 + 1);
        u8 r2 = pr[next], g2 = pg[next], b2 = pb[next];
        u8 f2 = (u8)(lo4 << 4);
        u8 f1 = (u8)(255 - f2);
        r1 = (u8)(fl_scale8(r1, f1) + fl_scale8(r2, f2));
        g1 = (u8)(fl_scale8(g1, f1) + fl_scale8(g2, f2));
        b1 = (u8)(fl_scale8(b1, f1) + fl_scale8(b2, f2));
    }

    if (brightness != 255) {
        if (brightness == 0) {
            r1 = 0;
            g1 = 0;
            b1 = 0;
        } else {
            u8 b2 = (u8)(brightness + 1);
            r1 = fl_scale8(r1, b2);
            g1 = fl_scale8(g1, b2);
            b1 = fl_scale8(b1, b2);
        }
    }

    *out_r = r1;
    *out_g = g1;
    *out_b = b1;
}

void fl_color_from_palette32(const u8 *pr, const u8 *pg, const u8 *pb, u8 index,
                              u8 brightness, int blend, u8 *out_r, u8 *out_g, u8 *out_b) {
    if (blend == 2) {
        index = fl_scale8(index, 247);
    }
    u8 hi5 = (u8)(index >> 3);
    u8 lo3 = (u8)(index & 0x07);

    u8 r1 = pr[hi5], g1 = pg[hi5], b1 = pb[hi5];

    if (blend != 0 && lo3) {
        u8 next = (hi5 == 31) ? 0 : (u8)(hi5 + 1);
        u8 r2 = pr[next], g2 = pg[next], b2 = pb[next];
        u8 f2 = (u8)(lo3 << 5);
        u8 f1 = (u8)(255 - f2);
        r1 = (u8)(fl_scale8(r1, f1) + fl_scale8(r2, f2));
        g1 = (u8)(fl_scale8(g1, f1) + fl_scale8(g2, f2));
        b1 = (u8)(fl_scale8(b1, f1) + fl_scale8(b2, f2));
    }

    if (brightness != 255) {
        if (brightness == 0) {
            r1 = 0;
            g1 = 0;
            b1 = 0;
        } else {
            u8 b2 = (u8)(brightness + 1);
            r1 = fl_scale8(r1, b2);
            g1 = fl_scale8(g1, b2);
            b1 = fl_scale8(b1, b2);
        }
    }

    *out_r = r1;
    *out_g = g1;
    *out_b = b1;
}

void fl_color_from_palette256(const u8 *pr, const u8 *pg, const u8 *pb, u8 index,
                               u8 brightness, u8 *out_r, u8 *out_g, u8 *out_b) {
    u8 r1 = pr[index], g1 = pg[index], b1 = pb[index];

    if (brightness != 255) {
        u8 b2 = (u8)(brightness + 1);
        r1 = fl_scale8_video(r1, b2);
        g1 = fl_scale8_video(g1, b2);
        b1 = fl_scale8_video(b1, b2);
    }

    *out_r = r1;
    *out_g = g1;
    *out_b = b1;
}

// ---------------------------------------------------------------------------
// ColorFromPalette (CHSV) — src/colorutils.cpp (FastLED 3.6.0)
// ---------------------------------------------------------------------------

void fl_color_from_palette16_hsv(const u8 *ph, const u8 *ps, const u8 *pv, u8 index,
                                  u8 brightness, int blend, u8 *out_h, u8 *out_s, u8 *out_v) {
    if (blend == 2) {
        index = fl_scale8(index, 239);
    }
    u8 hi4 = (u8)(index >> 4);
    u8 lo4 = (u8)(index & 0x0F);

    u8 hue1 = ph[hi4], sat1 = ps[hi4], val1 = pv[hi4];

    if (blend != 0 && lo4) {
        u8 next = (hi4 == 15) ? 0 : (u8)(hi4 + 1);
        u8 hue2 = ph[next], sat2 = ps[next], val2 = pv[next];
        u8 f2 = (u8)(lo4 << 4);
        u8 f1 = (u8)(255 - f2);

        if (sat1 == 0 || val1 == 0) hue1 = hue2;
        if (sat2 == 0 || val2 == 0) hue2 = hue1;

        sat1 = fl_scale8(sat1, f1);
        val1 = fl_scale8(val1, f1);
        sat2 = fl_scale8(sat2, f2);
        val2 = fl_scale8(val2, f2);
        sat1 = (u8)(sat1 + sat2);
        val1 = (u8)(val1 + val2);

        u8 deltahue = (u8)(hue2 - hue1);
        if (deltahue & 0x80) {
            hue1 = (u8)(hue1 - fl_scale8((u8)(256 - deltahue), f2));
        } else {
            hue1 = (u8)(hue1 + fl_scale8(deltahue, f2));
        }
    }

    if (brightness != 255) {
        val1 = fl_scale8_video(val1, brightness);
    }

    *out_h = hue1;
    *out_s = sat1;
    *out_v = val1;
}

void fl_color_from_palette32_hsv(const u8 *ph, const u8 *ps, const u8 *pv, u8 index,
                                  u8 brightness, int blend, u8 *out_h, u8 *out_s, u8 *out_v) {
    if (blend == 2) {
        index = fl_scale8(index, 247);
    }
    u8 hi5 = (u8)(index >> 3);
    u8 lo3 = (u8)(index & 0x07);

    u8 hue1 = ph[hi5], sat1 = ps[hi5], val1 = pv[hi5];

    if (blend != 0 && lo3) {
        u8 next = (hi5 == 31) ? 0 : (u8)(hi5 + 1);
        u8 hue2 = ph[next], sat2 = ps[next], val2 = pv[next];
        u8 f2 = (u8)(lo3 << 5);
        u8 f1 = (u8)(255 - f2);

        if (sat1 == 0 || val1 == 0) hue1 = hue2;
        if (sat2 == 0 || val2 == 0) hue2 = hue1;

        sat1 = fl_scale8(sat1, f1);
        val1 = fl_scale8(val1, f1);
        sat2 = fl_scale8(sat2, f2);
        val2 = fl_scale8(val2, f2);
        sat1 = (u8)(sat1 + sat2);
        val1 = (u8)(val1 + val2);

        u8 deltahue = (u8)(hue2 - hue1);
        if (deltahue & 0x80) {
            hue1 = (u8)(hue1 - fl_scale8((u8)(256 - deltahue), f2));
        } else {
            hue1 = (u8)(hue1 + fl_scale8(deltahue, f2));
        }
    }

    if (brightness != 255) {
        val1 = fl_scale8_video(val1, brightness);
    }

    *out_h = hue1;
    *out_s = sat1;
    *out_v = val1;
}

void fl_color_from_palette256_hsv(const u8 *ph, const u8 *ps, const u8 *pv, u8 index,
                                   u8 brightness, u8 *out_h, u8 *out_s, u8 *out_v) {
    u8 hue1 = ph[index], sat1 = ps[index], val1 = pv[index];

    if (brightness != 255) {
        val1 = fl_scale8_video(val1, brightness);
    }

    *out_h = hue1;
    *out_s = sat1;
    *out_v = val1;
}

// ---------------------------------------------------------------------------
// Gradient-palette parsing — CRGBPalette16/32/256's
// operator=(TProgmemRGBGradientPalette_bytes) — src/colorutils.h (FastLED 3.6.0)
// ---------------------------------------------------------------------------

typedef struct {
    u8 index, r, g, b;
} fl_grad_entry;

static int fl_grad_entry_at(const u8 *bytes, int byte_count, int i, fl_grad_entry *out) {
    int total = byte_count / 4;
    if (i >= total) return 0;
    const u8 *p = bytes + i * 4;
    out->index = p[0];
    out->r = p[1];
    out->g = p[2];
    out->b = p[3];
    return 1;
}

static int fl_grad_count_stops(const u8 *bytes, int byte_count) {
    int total = byte_count / 4;
    fl_grad_entry e;
    for (int i = 0; i < total; ++i) {
        fl_grad_entry_at(bytes, byte_count, i, &e);
        if (e.index == 255) return i + 1;
    }
    return total;
}

// fill_gradient_RGB(CRGB* leds, uint16_t startpos, CRGB startcolor, uint16_t
// endpos, CRGB endcolor) — src/colorutils.cpp — into an existing n-length
// array, positions beyond n silently dropped (matches color8's
// fill_gradient_rgb_range, which uses `leds.get_mut`).
static void fl_fill_gradient_rgb_range(u8 *out_r, u8 *out_g, u8 *out_b, int n, u16 startpos, u8 sr,
                                        u8 sg, u8 sb, u16 endpos, u8 er, u8 eg, u8 eb) {
    if (endpos < startpos) {
        u16 tp = endpos;
        endpos = startpos;
        startpos = tp;
        u8 tr = sr, tg = sg, tb = sb;
        sr = er;
        sg = eg;
        sb = eb;
        er = tr;
        eg = tg;
        eb = tb;
    }

    int16_t rdistance87 = (int16_t)(((int16_t)er - (int16_t)sr) << 7);
    int16_t gdistance87 = (int16_t)(((int16_t)eg - (int16_t)sg) << 7);
    int16_t bdistance87 = (int16_t)(((int16_t)eb - (int16_t)sb) << 7);

    u16 pixeldistance = (u16)(endpos - startpos);
    int16_t divisor = pixeldistance ? (int16_t)pixeldistance : 1;

    int16_t rdelta87 = (int16_t)((rdistance87 / divisor) * 2);
    int16_t gdelta87 = (int16_t)((gdistance87 / divisor) * 2);
    int16_t bdelta87 = (int16_t)((bdistance87 / divisor) * 2);

    int32_t r88 = (int32_t)sr << 8;
    int32_t g88 = (int32_t)sg << 8;
    int32_t b88 = (int32_t)sb << 8;

    for (u16 i = startpos; i <= endpos; ++i) {
        if (i < (u16)n) {
            out_r[i] = (u8)(r88 >> 8);
            out_g[i] = (u8)(g88 >> 8);
            out_b[i] = (u8)(b88 >> 8);
        }
        r88 += rdelta87;
        g88 += gdelta87;
        b88 += bdelta87;
        if (i == endpos) break;
    }
}

static void fl_gradient_palette_compact(const u8 *bytes, int byte_count, u8 *out_r, u8 *out_g,
                                         u8 *out_b, int n, int divisor, int max_slot) {
    for (int i = 0; i < n; ++i) {
        out_r[i] = 0;
        out_g[i] = 0;
        out_b[i] = 0;
    }

    fl_grad_entry e0;
    if (!fl_grad_entry_at(bytes, byte_count, 0, &e0)) return;
    u8 sr = e0.r, sg = e0.g, sb = e0.b;

    int count = fl_grad_count_stops(bytes, byte_count);
    int last_slot_used = -1;
    int index_start = 0;
    int i = 1;

    while (index_start < 255) {
        fl_grad_entry e;
        if (!fl_grad_entry_at(bytes, byte_count, i, &e)) break;
        int index_end = e.index;

        int istart8 = index_start / divisor;
        int iend8 = index_end / divisor;

        if (count < 16) {
            if (istart8 <= last_slot_used && last_slot_used < max_slot) {
                istart8 = last_slot_used + 1;
                if (iend8 < istart8) iend8 = istart8;
            }
            last_slot_used = iend8;
        }

        fl_fill_gradient_rgb_range(out_r, out_g, out_b, n, (u16)istart8, sr, sg, sb, (u16)iend8,
                                    e.r, e.g, e.b);

        index_start = index_end;
        sr = e.r;
        sg = e.g;
        sb = e.b;
        i++;
    }
}

void fl_crgb_palette16_from_gradient(const u8 *bytes, int byte_count, u8 *out_r, u8 *out_g,
                                      u8 *out_b) {
    fl_gradient_palette_compact(bytes, byte_count, out_r, out_g, out_b, 16, 16, 15);
}

void fl_crgb_palette32_from_gradient(const u8 *bytes, int byte_count, u8 *out_r, u8 *out_g,
                                      u8 *out_b) {
    fl_gradient_palette_compact(bytes, byte_count, out_r, out_g, out_b, 32, 8, 31);
}

void fl_crgb_palette256_from_gradient(const u8 *bytes, int byte_count, u8 *out_r, u8 *out_g,
                                       u8 *out_b) {
    for (int i = 0; i < 256; ++i) {
        out_r[i] = 0;
        out_g[i] = 0;
        out_b[i] = 0;
    }

    fl_grad_entry e0;
    if (!fl_grad_entry_at(bytes, byte_count, 0, &e0)) return;
    u8 sr = e0.r, sg = e0.g, sb = e0.b;

    int index_start = 0;
    int i = 1;
    while (index_start < 255) {
        fl_grad_entry e;
        if (!fl_grad_entry_at(bytes, byte_count, i, &e)) break;
        int index_end = e.index;

        fl_fill_gradient_rgb_range(out_r, out_g, out_b, 256, (u16)index_start, sr, sg, sb,
                                    (u16)index_end, e.r, e.g, e.b);

        index_start = index_end;
        sr = e.r;
        sg = e.g;
        sb = e.b;
        i++;
    }
}
