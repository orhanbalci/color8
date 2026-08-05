//! Shared "corporate styling" for the example gallery: one canvas layout,
//! one background/border/text treatment, reused by every example so the
//! generated PNGs read as one consistent set rather than six unrelated
//! scripts.
//!
//! The 8x8 bitmap font below is `font8x8_basic` by Daniel Hepper (public
//! domain, based on the public-domain IBM VGA font by Marcel Sondaar) —
//! see <https://github.com/dhepper/font8x8>. Only the space..'Z' glyphs
//! (0x20..=0x5A) are included, since every label in this gallery is
//! uppercase.
#![allow(dead_code)]

use std::f32::consts::PI;
use std::path::PathBuf;

use color8::Crgb;
use image::{Rgb, RgbImage};

pub const CANVAS_W: u32 = 1000;
pub const CANVAS_H: u32 = 380;

const MARGIN: u32 = 22;
const BORDER: u32 = 3;
const TITLE_H: u32 = 72;
const FOOTER_H: u32 = 34;

const BG_TOP: (u8, u8, u8) = (13, 15, 21);
const BG_BOTTOM: (u8, u8, u8) = (23, 26, 36);
const ACCENT: (u8, u8, u8) = (64, 224, 208);
const TITLE_COLOR: (u8, u8, u8) = (235, 238, 245);
const SUBTITLE_COLOR: (u8, u8, u8) = (138, 148, 163);
const WORDMARK_COLOR: (u8, u8, u8) = (82, 90, 104);

/// 8x8 bitmap glyphs, `space..='Z'`. Each row is one scanline, LSB-first.
const FONT: &[(char, [u8; 8])] = &[
    (' ', [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
    ('!', [0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00]),
    ('"', [0x36, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
    ('#', [0x36, 0x36, 0x7F, 0x36, 0x7F, 0x36, 0x36, 0x00]),
    ('$', [0x0C, 0x3E, 0x03, 0x1E, 0x30, 0x1F, 0x0C, 0x00]),
    ('%', [0x00, 0x63, 0x33, 0x18, 0x0C, 0x66, 0x63, 0x00]),
    ('&', [0x1C, 0x36, 0x1C, 0x6E, 0x3B, 0x33, 0x6E, 0x00]),
    ('\'', [0x06, 0x06, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00]),
    ('(', [0x18, 0x0C, 0x06, 0x06, 0x06, 0x0C, 0x18, 0x00]),
    (')', [0x06, 0x0C, 0x18, 0x18, 0x18, 0x0C, 0x06, 0x00]),
    ('*', [0x00, 0x66, 0x3C, 0xFF, 0x3C, 0x66, 0x00, 0x00]),
    ('+', [0x00, 0x0C, 0x0C, 0x3F, 0x0C, 0x0C, 0x00, 0x00]),
    (',', [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C, 0x06]),
    ('-', [0x00, 0x00, 0x00, 0x3F, 0x00, 0x00, 0x00, 0x00]),
    ('.', [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C, 0x00]),
    ('/', [0x60, 0x30, 0x18, 0x0C, 0x06, 0x03, 0x01, 0x00]),
    ('0', [0x3E, 0x63, 0x73, 0x7B, 0x6F, 0x67, 0x3E, 0x00]),
    ('1', [0x0C, 0x0E, 0x0C, 0x0C, 0x0C, 0x0C, 0x3F, 0x00]),
    ('2', [0x1E, 0x33, 0x30, 0x1C, 0x06, 0x33, 0x3F, 0x00]),
    ('3', [0x1E, 0x33, 0x30, 0x1C, 0x30, 0x33, 0x1E, 0x00]),
    ('4', [0x38, 0x3C, 0x36, 0x33, 0x7F, 0x30, 0x78, 0x00]),
    ('5', [0x3F, 0x03, 0x1F, 0x30, 0x30, 0x33, 0x1E, 0x00]),
    ('6', [0x1C, 0x06, 0x03, 0x1F, 0x33, 0x33, 0x1E, 0x00]),
    ('7', [0x3F, 0x33, 0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x00]),
    ('8', [0x1E, 0x33, 0x33, 0x1E, 0x33, 0x33, 0x1E, 0x00]),
    ('9', [0x1E, 0x33, 0x33, 0x3E, 0x30, 0x18, 0x0E, 0x00]),
    (':', [0x00, 0x0C, 0x0C, 0x00, 0x00, 0x0C, 0x0C, 0x00]),
    (';', [0x00, 0x0C, 0x0C, 0x00, 0x00, 0x0C, 0x0C, 0x06]),
    ('<', [0x18, 0x0C, 0x06, 0x03, 0x06, 0x0C, 0x18, 0x00]),
    ('=', [0x00, 0x00, 0x3F, 0x00, 0x00, 0x3F, 0x00, 0x00]),
    ('>', [0x06, 0x0C, 0x18, 0x30, 0x18, 0x0C, 0x06, 0x00]),
    ('?', [0x1E, 0x33, 0x30, 0x18, 0x0C, 0x00, 0x0C, 0x00]),
    ('@', [0x3E, 0x63, 0x7B, 0x7B, 0x7B, 0x03, 0x1E, 0x00]),
    ('A', [0x0C, 0x1E, 0x33, 0x33, 0x3F, 0x33, 0x33, 0x00]),
    ('B', [0x3F, 0x66, 0x66, 0x3E, 0x66, 0x66, 0x3F, 0x00]),
    ('C', [0x3C, 0x66, 0x03, 0x03, 0x03, 0x66, 0x3C, 0x00]),
    ('D', [0x1F, 0x36, 0x66, 0x66, 0x66, 0x36, 0x1F, 0x00]),
    ('E', [0x7F, 0x46, 0x16, 0x1E, 0x16, 0x46, 0x7F, 0x00]),
    ('F', [0x7F, 0x46, 0x16, 0x1E, 0x16, 0x06, 0x0F, 0x00]),
    ('G', [0x3C, 0x66, 0x03, 0x03, 0x73, 0x66, 0x7C, 0x00]),
    ('H', [0x33, 0x33, 0x33, 0x3F, 0x33, 0x33, 0x33, 0x00]),
    ('I', [0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00]),
    ('J', [0x78, 0x30, 0x30, 0x30, 0x33, 0x33, 0x1E, 0x00]),
    ('K', [0x67, 0x66, 0x36, 0x1E, 0x36, 0x66, 0x67, 0x00]),
    ('L', [0x0F, 0x06, 0x06, 0x06, 0x46, 0x66, 0x7F, 0x00]),
    ('M', [0x63, 0x77, 0x7F, 0x7F, 0x6B, 0x63, 0x63, 0x00]),
    ('N', [0x63, 0x67, 0x6F, 0x7B, 0x73, 0x63, 0x63, 0x00]),
    ('O', [0x1C, 0x36, 0x63, 0x63, 0x63, 0x36, 0x1C, 0x00]),
    ('P', [0x3F, 0x66, 0x66, 0x3E, 0x06, 0x06, 0x0F, 0x00]),
    ('Q', [0x1E, 0x33, 0x33, 0x33, 0x3B, 0x1E, 0x38, 0x00]),
    ('R', [0x3F, 0x66, 0x66, 0x3E, 0x36, 0x66, 0x67, 0x00]),
    ('S', [0x1E, 0x33, 0x07, 0x0E, 0x38, 0x33, 0x1E, 0x00]),
    ('T', [0x3F, 0x2D, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00]),
    ('U', [0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x3F, 0x00]),
    ('V', [0x33, 0x33, 0x33, 0x33, 0x33, 0x1E, 0x0C, 0x00]),
    ('W', [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00]),
    ('X', [0x63, 0x63, 0x36, 0x1C, 0x1C, 0x36, 0x63, 0x00]),
    ('Y', [0x33, 0x33, 0x33, 0x1E, 0x0C, 0x0C, 0x1E, 0x00]),
    ('Z', [0x7F, 0x63, 0x31, 0x18, 0x4C, 0x66, 0x7F, 0x00]),
];

fn glyph(c: char) -> [u8; 8] {
    let c = c.to_ascii_uppercase();
    FONT.iter()
        .find(|(g, _)| *g == c)
        .map(|(_, bits)| *bits)
        .unwrap_or([0; 8])
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

/// A branded canvas: gradient background, accent border, a title/subtitle
/// pair top-left, and a "COLOR8" wordmark bottom-right. Every example
/// builds one of these and draws into [`Canvas::content_rect`].
pub struct Canvas {
    img: RgbImage,
}

impl Canvas {
    pub fn new(title: &str, subtitle: &str) -> Self {
        let mut img = RgbImage::new(CANVAS_W, CANVAS_H);
        for y in 0..CANVAS_H {
            let t = y as f32 / (CANVAS_H - 1) as f32;
            let px = Rgb([
                lerp_u8(BG_TOP.0, BG_BOTTOM.0, t),
                lerp_u8(BG_TOP.1, BG_BOTTOM.1, t),
                lerp_u8(BG_TOP.2, BG_BOTTOM.2, t),
            ]);
            for x in 0..CANVAS_W {
                img.put_pixel(x, y, px);
            }
        }

        let mut canvas = Self { img };
        canvas.draw_border();
        canvas.draw_text(title, MARGIN + 12, MARGIN + 14, 3, TITLE_COLOR);
        canvas.draw_text(
            subtitle,
            MARGIN + 12,
            MARGIN + 14 + 3 * 9 + 8,
            1,
            SUBTITLE_COLOR,
        );
        let wordmark = "COLOR8";
        let wordmark_w = text_width(wordmark, 2);
        canvas.draw_text(
            wordmark,
            CANVAS_W - MARGIN - 10 - wordmark_w,
            CANVAS_H - MARGIN - 8 - 16,
            2,
            WORDMARK_COLOR,
        );
        canvas
    }

    /// The area below the title block and above the footer, inset from the
    /// border — where each example draws its actual content.
    pub fn content_rect(&self) -> (u32, u32, u32, u32) {
        let x = MARGIN + BORDER + 12;
        let y = MARGIN + BORDER + TITLE_H;
        let w = CANVAS_W - 2 * x;
        let h = CANVAS_H - y - (MARGIN + BORDER + FOOTER_H);
        (x, y, w, h)
    }

    fn put(&mut self, x: i64, y: i64, color: (u8, u8, u8)) {
        if x < 0 || y < 0 || x as u32 >= CANVAS_W || y as u32 >= CANVAS_H {
            return;
        }
        self.img
            .put_pixel(x as u32, y as u32, Rgb([color.0, color.1, color.2]));
    }

    fn put_crgb(&mut self, x: i64, y: i64, color: Crgb) {
        self.put(x, y, (color.r, color.g, color.b));
    }

    fn draw_border(&mut self) {
        for t in 0..BORDER {
            for x in MARGIN..(CANVAS_W - MARGIN) {
                self.put(x as i64, (MARGIN + t) as i64, ACCENT);
                self.put(x as i64, (CANVAS_H - MARGIN - 1 - t) as i64, ACCENT);
            }
            for y in MARGIN..(CANVAS_H - MARGIN) {
                self.put((MARGIN + t) as i64, y as i64, ACCENT);
                self.put((CANVAS_W - MARGIN - 1 - t) as i64, y as i64, ACCENT);
            }
        }
    }

    /// Draws `text` (a `space..='Z'`-only string) at `(x, y)`, each glyph
    /// cell scaled up by `scale`.
    pub fn draw_text(&mut self, text: &str, x: u32, y: u32, scale: u32, color: (u8, u8, u8)) {
        let mut cursor = x as i64;
        for ch in text.chars() {
            let bits = glyph(ch);
            for (row, byte) in bits.iter().enumerate() {
                for col in 0..8u32 {
                    if byte & (1 << col) != 0 {
                        for sy in 0..scale {
                            for sx in 0..scale {
                                self.put(
                                    cursor + (col * scale + sx) as i64,
                                    y as i64 + (row as u32 * scale + sy) as i64,
                                    color,
                                );
                            }
                        }
                    }
                }
            }
            cursor += (9 * scale) as i64;
        }
    }

    /// A small caption to the left of a content row (used to label stacked
    /// gradients), right-aligned to `right_edge`.
    pub fn draw_label_right_aligned(&mut self, text: &str, right_edge: u32, y: u32) {
        let w = text_width(text, 1);
        let x = right_edge.saturating_sub(w);
        self.draw_text(text, x, y, 1, SUBTITLE_COLOR);
    }

    /// Stretches `colors` smoothly across a `w`x`h` rectangle at `(x, y)` —
    /// used for gradients and sweeps, where the point is a continuous ramp.
    pub fn draw_strip(&mut self, colors: &[Crgb], x: u32, y: u32, w: u32, h: u32) {
        if colors.is_empty() || w == 0 {
            return;
        }
        for col in 0..w {
            let idx = (col as usize * colors.len()) / w as usize;
            let idx = idx.min(colors.len() - 1);
            let color = colors[idx];
            for row in 0..h {
                self.put_crgb((x + col) as i64, (y + row) as i64, color);
            }
        }
    }

    /// Draws `colors` as discrete `cell`x`cell` blocks with a gap between
    /// them, starting at `(x, y)` — used where individual samples (LEDs,
    /// blend steps) matter more than a smooth ramp.
    pub fn draw_blocks(&mut self, colors: &[Crgb], x: u32, y: u32, cell: u32, gap: u32) {
        let mut cursor = x;
        for &color in colors {
            for row in 0..cell {
                for col in 0..cell {
                    self.put_crgb((cursor + col) as i64, (y + row) as i64, color);
                }
            }
            cursor += cell + gap;
        }
    }

    /// Arranges `colors` as `cell`x`cell` blocks evenly spaced around a
    /// circle of `radius` centered at `(cx, cy)`, starting at 12 o'clock.
    pub fn draw_ring(&mut self, colors: &[Crgb], cx: i64, cy: i64, radius: f32, cell: i64) {
        let n = colors.len();
        if n == 0 {
            return;
        }
        for (i, &color) in colors.iter().enumerate() {
            let angle = (i as f32 / n as f32) * 2.0 * PI - PI / 2.0;
            let px = cx + (radius * angle.cos()).round() as i64;
            let py = cy + (radius * angle.sin()).round() as i64;
            for row in -cell / 2..cell / 2 {
                for col in -cell / 2..cell / 2 {
                    self.put_crgb(px + col, py + row, color);
                }
            }
        }
    }

    pub fn save(&self, name: &str) {
        let gallery = gallery_dir();
        std::fs::create_dir_all(&gallery).expect("create gallery/ directory");
        let path = gallery.join(format!("{name}.png"));
        self.img.save(&path).expect("write PNG");
        println!("wrote {}", path.display());
    }
}

fn text_width(text: &str, scale: u32) -> u32 {
    if text.is_empty() {
        return 0;
    }
    text.chars().count() as u32 * 9 * scale - scale
}

fn gallery_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR is .../color8/color8 (this crate); the repo root,
    // where the gallery lives alongside the two crates, is one level up.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate has a parent directory")
        .join("gallery")
}
