//! Landscape geometry adapted from public PSX crates/player/src/input.rs
//! at 1e4ba86fa0a5fc1470837b6bcc06bd20b6642c11 (Layout::ps2 and clear_picture).
//! Copyright (c) 2026 Machine Rhythm contributors. MIT: LICENSES/PSX-MIT.txt.
//! Xbox mappings, circles and Black/White/trigger placement are xbx adaptations.
use super::{COUNT, Circle};

#[derive(Clone, Copy, Default)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}
impl Rect {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}

pub(super) fn controls(w: f32, h: f32, density: f32) -> [Circle; COUNT] {
    // Keep PSX's side columns and analog / inner menu arrangement. Work in dp;
    // scale down transient small windows without clipping or overlapping hits.
    let d = density.max(0.5);
    let width = (w / d).max(640.0);
    let height = (h / d).max(360.0);
    let sx = w / width;
    let sy = h / height;
    let stick = 96.0;
    let face = 148.0;
    let main_y = height - 16.0 - stick;
    let face_y = main_y - 8.0 - face;
    let cap = (face - 4.0 * std::f32::consts::SQRT_2) / (1.0 + std::f32::consts::SQRT_2);
    let step = (face - cap) / 2.0;
    let left = 16.0 + face / 2.0;
    let right = width - left;
    let middle = face_y + face / 2.0;
    let trigger_y = face_y - 28.0;
    let entries = [
        (64.0, main_y + 48.0, 48.0),
        (width - 64.0, main_y + 48.0, 48.0),
        (right, middle + step, cap / 2.0), // A
        (right + step, middle, cap / 2.0), // B
        (right - step, middle, cap / 2.0), // X
        (right, middle - step, cap / 2.0), // Y
        (left, middle - step, cap / 2.0),
        (left, middle + step, cap / 2.0),
        (left - step, middle, cap / 2.0),
        (left + step, middle, cap / 2.0),
        (144.0, main_y + 24.0, 24.0),         // Back
        (width - 144.0, main_y + 24.0, 24.0), // Start
        (width - 146.0, trigger_y, 22.0),     // White
        (width - 94.0, trigger_y, 22.0),      // Black
        (144.0, main_y + 72.0, 24.0),         // left stick click
        (width - 144.0, main_y + 72.0, 24.0),
        (40.0, trigger_y, 24.0),         // LT
        (width - 42.0, trigger_y, 22.0), // RT
    ];
    entries.map(|(x, y, radius)| Circle {
        x: x * sx,
        y: y * sy,
        radius: radius * sx.min(sy),
    })
}

pub(super) fn picture(w: f32, h: f32, density: f32) -> [f32; 4] {
    let obstacles = controls(w, h, density).map(|c| {
        // Include the outline and a small clear gap, bounded by the safe area.
        let r = c.radius + 2.0 * density.min(w / 640.0).min(h / 360.0);
        Rect::new((c.x - r).max(0.0), (c.y - r).max(0.0), r * 2.0, r * 2.0)
    });
    let r = clear_picture(w, h, &obstacles);
    [r.x, r.y, r.w, r.h]
}

/// Largest horizontally centered clear 4:3 picture, at its highest placement.
/// Grow about the display center, stopping beside or above each obstruction.
/// Only used on layout changes, never during emulation frames.
fn clear_picture(w: f32, h: f32, obstacles: &[Rect]) -> Rect {
    let center = w / 2.;
    let mut centered = Rect::default();
    for y in std::iter::once(0.)
        .chain(obstacles.iter().map(|r| r.y + r.h))
        .filter(|y| *y >= 0. && *y <= h)
    {
        let mut width = w.min((h - y) * 4. / 3.).max(0.);
        for r in obstacles {
            if r.y + r.h <= y + 0.01 {
                continue;
            }
            let beside = (center - r.x - r.w).max(r.x - center).max(0.) * 2.;
            let above = (r.y - y).max(0.) * 4. / 3.;
            width = width.min(beside.max(above));
        }
        if width > centered.w || (width == centered.w && y < centered.y) {
            centered = Rect::new((w - width) / 2., y, width, width * 3. / 4.);
        }
    }
    if centered.w > 0. {
        return centered;
    }
    // Preserve the public PSX fallback for a fully occupied center column.
    let mut best = Rect::default();
    for x in std::iter::once(0.)
        .chain(obstacles.iter().map(|r| r.x + r.w))
        .filter(|x| *x >= 0. && *x <= w)
    {
        for y in std::iter::once(0.)
            .chain(obstacles.iter().map(|r| r.y + r.h))
            .filter(|y| *y >= 0. && *y <= h)
        {
            let mut width = (w - x).min((h - y) * 4. / 3.).max(0.);
            for r in obstacles {
                if r.x + r.w <= x + 0.01 || r.y + r.h <= y + 0.01 {
                    continue;
                }
                let limit = (r.x - x).max((r.y - y) * 4. / 3.).max(0.);
                width = width.min(limit);
            }
            if width > best.w || (width == best.w && y < best.y) {
                best = Rect::new(x, y, width, width * 3. / 4.);
            }
        }
    }
    best
}
