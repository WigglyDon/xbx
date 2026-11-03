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

pub(super) fn legacy_controls(w: f32, h: f32, density: f32) -> [Circle; COUNT] {
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

pub(super) fn keep_clear_picture(w: f32, h: f32, density: f32) -> [f32; 4] {
    let obstacles = legacy_controls(w, h, density).map(|c| {
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

/// Fit the whole native-aspect image before considering touch controls.
pub(super) fn maximum_picture(w: f32, h: f32, aspect: f32) -> [f32; 4] {
    if ![w, h, aspect].iter().all(|n| n.is_finite() && *n > 0.0) {
        return [0.0; 4];
    }
    let width = w.min(h * aspect);
    let height = width / aspect;
    // A width-limited picture leaves one contiguous bottom area for controls.
    [(w - width) * 0.5, 0.0, width, height]
}
fn left(index: usize) -> bool {
    matches!(index, 0 | 6..=10 | 14 | 16)
}

/// Pack the accepted side-column template isotropically into actual spare space.
fn fitted_column(w: f32, h: f32, d: f32, side: bool, minimum: f32) -> Option<[Circle; COUNT]> {
    let template = legacy_controls(640.0, 360.0, 1.0);
    let indices = || (0..COUNT).filter(|i| left(*i) == side);
    let x0 = indices()
        .map(|i| template[i].x - template[i].radius)
        .fold(f32::INFINITY, f32::min);
    let x1 = indices()
        .map(|i| template[i].x + template[i].radius)
        .fold(0.0, f32::max);
    let y0 = indices()
        .map(|i| template[i].y - template[i].radius)
        .fold(f32::INFINITY, f32::min);
    let y1 = indices()
        .map(|i| template[i].y + template[i].radius)
        .fold(0.0, f32::max);
    let margin = 6.0 * d;
    let scale = ((w - 2.0 * margin) / (x1 - x0)).min((h - 2.0 * margin) / (y1 - y0));
    if scale <= 0.0 || indices().any(|i| template[i].radius * scale < minimum) {
        return None;
    }
    let mut result = [Circle::default(); COUNT];
    for i in indices() {
        let c = template[i];
        result[i] = Circle {
            x: (w - (x1 - x0) * scale) * 0.5 + (c.x - x0) * scale,
            y: (h - (y1 - y0) * scale) * 0.5 + (c.y - y0) * scale,
            radius: c.radius * scale,
        };
    }
    Some(result)
}
/// A narrow column stacks the menu/click pair below the stick rather than
/// borrowing picture width. This retains every original-Xbox control.
fn narrow_column(w: f32, h: f32, d: f32, side: bool, minimum: f32) -> Option<[Circle; COUNT]> {
    let margin = 6.0 * d;
    let gap = 4.0 * d;
    let width = w - 2.0 * margin;
    if width <= 0.0 {
        return None;
    }
    let face = (width - std::f32::consts::SQRT_2 * gap) / (2.0 * (1.0 + std::f32::consts::SQRT_2));
    let step = (2.0 * face + gap) / std::f32::consts::SQRT_2;
    let small = (22.0 * d).min((width - gap) / 4.0);
    let stick = (48.0 * d).min(width / 2.0);
    let diamond = 2.0 * (face + step);
    let used = 6.0 * small + diamond + 2.0 * stick + 4.0 * gap;
    let scale = ((h - 2.0 * margin) / used).min(1.0);
    if face.min(small).min(stick) * scale < minimum {
        return None;
    }
    let mut out = [Circle::default(); COUNT];
    let cx = w / 2.0;
    let mut y = (h - used * scale) * 0.5;
    let mut put = |i: usize, x: f32, y: f32, r: f32| {
        out[i] = Circle {
            x: cx + (x - cx) * scale,
            y,
            radius: r * scale,
        };
    };
    put(if side { 16 } else { 17 }, cx, y + small * scale, small);
    y += (2.0 * small + gap) * scale;
    if !side {
        put(12, cx - small - gap / 2.0, y + small * scale, small);
        put(13, cx + small + gap / 2.0, y + small * scale, small);
    }
    y += (2.0 * small + gap) * scale;
    let cy = y + (face + step) * scale;
    if side {
        for (i, x, y) in [
            (6, cx, cy - step * scale),
            (7, cx, cy + step * scale),
            (8, cx - step, cy),
            (9, cx + step, cy),
        ] {
            put(i, x, y, face);
        }
    } else {
        for (i, x, y) in [
            (2, cx, cy + step * scale),
            (3, cx + step, cy),
            (4, cx - step, cy),
            (5, cx, cy - step * scale),
        ] {
            put(i, x, y, face);
        }
    }
    y += (diamond + gap) * scale;
    put(if side { 0 } else { 1 }, cx, y + stick * scale, stick);
    y += (2.0 * stick + gap) * scale;
    put(
        if side { 10 } else { 11 },
        cx - small - gap / 2.0,
        y + small * scale,
        small,
    );
    put(
        if side { 14 } else { 15 },
        cx + small + gap / 2.0,
        y + small * scale,
        small,
    );
    Some(out)
}
fn columns(column_width: f32, h: f32, d: f32, minimum: f32) -> [Circle; COUNT] {
    let mut out = [Circle::default(); COUNT];
    for side in [true, false] {
        let Some(group) = fitted_column(column_width, h, d, side, minimum)
            .or_else(|| narrow_column(column_width, h, d, side, minimum))
        else {
            return [Circle::default(); COUNT];
        };
        for (i, mut c) in group
            .into_iter()
            .enumerate()
            .filter(|(i, _)| left(*i) == side)
        {
            if !side {
                c.x += column_width;
            }
            out[i] = c;
        }
    }
    out
}
pub(super) fn bottom_controls(w: f32, h: f32, density: f32) -> [Circle; COUNT] {
    columns(w * 0.5, h, density.max(0.5), 0.0)
}
pub(super) fn spare_controls(w: f32, h: f32, density: f32) -> [Circle; COUNT] {
    let d = density.max(0.5);
    let picture = maximum_picture(w, h, 4.0 / 3.0);
    if picture[0] > 0.0 {
        let mut out = columns(picture[0], h, d, 12.0 * d);
        for (i, c) in out.iter_mut().enumerate() {
            if !left(i) {
                c.x += picture[2];
            }
        }
        out
    } else {
        let mut out = columns(w * 0.5, (h - picture[3]).max(0.0), d, 12.0 * d);
        for c in &mut out {
            c.y += picture[3];
        }
        out
    }
}
