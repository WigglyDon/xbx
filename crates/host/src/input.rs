//! Original one-pad touch model. Captures survive moves; cancellation neutralizes all channels.
use std::collections::BTreeMap;
mod layout;

pub const COUNT: usize = 18;
pub const LABELS: [&str; COUNT] = [
    "L", "R", "A", "B", "X", "Y", "↑", "↓", "←", "→", "Back", "Start", "White", "Black", "LS",
    "RS", "LT", "RT",
];

/// ARGB drawing values owned by the Rust player, consumed by the platform canvas.
pub fn color(index: usize, pressed: bool, part: i32) -> u32 {
    let (fill, ink) = match index {
        2 => (0xff249b45, 0xffffffff), // Xbox A: green
        3 => (0xffd32d35, 0xffffffff), // B: red
        4 => (0xff247bd1, 0xffffffff), // X: blue
        5 => (0xfff2ce32, 0xff171b20), // Y: yellow
        12 => (0xffeeeeee, 0xff171b20),
        13 => (0xff101317, 0xffeeeeee),
        _ => (0xe6323942, 0xffeeeeee),
    };
    match part {
        0 => fill,
        1 => ink,
        _ if pressed => 0xffffffff,
        _ => 0xff8b949e,
    }
}

pub fn label_scale(index: usize) -> f32 {
    if (2..6).contains(&index) { 1.1 } else { 0.68 }
}

pub fn outline_width(pressed: bool) -> f32 {
    if pressed { 3.0 } else { 1.0 }
}
// SDL gamepad button indices. Xbox White/Black use the retained shoulder mapping.
const BUTTON: [usize; 14] = [0, 1, 2, 3, 11, 12, 13, 14, 4, 6, 9, 10, 7, 8];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pad {
    pub buttons: u32,
    pub axes: [i16; 6],
}
#[derive(Clone, Copy)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

/// One platform-supplied safe content rectangle in View coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub density: f32,
}
impl Viewport {
    pub fn new(rect: [f32; 4], density: f32) -> Option<Self> {
        if !rect.iter().all(|x| x.is_finite())
            || rect[0] < 0.0
            || rect[1] < 0.0
            || rect[2] <= 0.0
            || rect[3] <= 0.0
            || !density.is_finite()
            || density <= 0.0
        {
            return None;
        }
        Some(Self {
            left: rect[0],
            top: rect[1],
            width: rect[2],
            height: rect[3],
            density,
        })
    }
    pub fn surface(self) -> [f32; 4] {
        let mut r = game_rect(self.width, self.height, self.density);
        r[0] += self.left;
        r[1] += self.top;
        r
    }
    pub fn control(self, index: usize) -> Circle {
        let mut c = circle(index, self.width, self.height, self.density);
        c.x += self.left;
        c.y += self.top;
        c
    }
}

fn band(h: f32, density: f32) -> f32 {
    (h * 0.48).min(260.0 * density.max(0.5))
}
pub fn game_rect(w: f32, h: f32, density: f32) -> [f32; 4] {
    if w > h * 1.35 {
        return layout::picture(w, h, density);
    }
    let available = (h - band(h, density)).max(1.0);
    let height = available.min(w * 0.75);
    let width = height * 4.0 / 3.0;
    [(w - width) * 0.5, (available - height) * 0.5, width, height]
}
pub fn circle(index: usize, w: f32, h: f32, density: f32) -> Circle {
    if w > h * 1.35 {
        return layout::controls(w, h, density)[index];
    }
    let b = band(h, density);
    let top = h - b;
    let (x, y) = [
        (0.16, 0.77),
        (0.52, 0.77),
        (0.86, 0.65),
        (0.96, 0.47),
        (0.76, 0.47),
        (0.86, 0.29),
        (0.16, 0.23),
        (0.16, 0.55),
        (0.055, 0.39),
        (0.265, 0.39),
        (0.40, 0.30),
        (0.56, 0.30),
        (0.77, 0.10),
        (0.94, 0.10),
        (0.32, 0.90),
        (0.68, 0.90),
        (0.055, 0.10),
        (0.62, 0.10),
    ][index];
    let radius = if index < 2 {
        (b * 0.18).min(w * 0.12)
    } else {
        (w * 0.052).min(b * 0.085)
    };
    let radius = radius.min(w * 0.49).min(h * 0.49);
    Circle {
        x: (x * w).clamp(radius, w - radius),
        y: (top + y * b).clamp(radius, h - radius),
        radius,
    }
}

struct Capture {
    control: usize,
    x: f32,
    y: f32,
    origin_y: f32,
}
#[derive(Default)]
pub struct Touch {
    pointers: BTreeMap<i32, Capture>,
}
impl Touch {
    pub const fn new() -> Self {
        Self {
            pointers: BTreeMap::new(),
        }
    }
    pub fn clear(&mut self) {
        self.pointers.clear();
    }
    pub fn active(&self, control: usize) -> bool {
        self.pointers.values().any(|p| p.control == control)
    }
    pub fn event_in(&mut self, id: i32, phase: i32, position: [f32; 2], view: Viewport) -> Pad {
        self.event(
            id,
            phase,
            [position[0] - view.left, position[1] - view.top],
            view.width,
            view.height,
            view.density,
        )
    }
    pub fn event(
        &mut self,
        id: i32,
        phase: i32,
        position: [f32; 2],
        w: f32,
        h: f32,
        density: f32,
    ) -> Pad {
        let [x, y] = position;
        if phase == 3 {
            self.clear()
        } else if phase == 2 {
            self.pointers.remove(&id);
        } else if x.is_finite() && y.is_finite() && w > 0.0 && h > 0.0 && density.is_finite() {
            if phase == 0 && self.pointers.len() < 16 {
                for i in 0..COUNT {
                    let c = circle(i, w, h, density);
                    if !self.active(i) && (x - c.x).hypot(y - c.y) <= c.radius {
                        self.pointers.insert(
                            id,
                            Capture {
                                control: i,
                                x,
                                y,
                                origin_y: y,
                            },
                        );
                        break;
                    }
                }
            } else if let Some(p) = self.pointers.get_mut(&id) {
                p.x = x;
                p.y = y;
            }
        }
        self.pad(w, h, density)
    }
    pub fn pad(&self, w: f32, h: f32, density: f32) -> Pad {
        let mut pad = Pad::default();
        for p in self.pointers.values() {
            let c = circle(p.control, w, h, density);
            if c.radius <= 0.0 {
                continue;
            }
            if p.control < 2 {
                let x = (p.x - c.x) / c.radius;
                let y = (p.y - c.y) / c.radius;
                let length = x.hypot(y);
                let scale = ((length - 0.12) / 0.88).clamp(0.0, 1.0) / length.max(0.0001);
                pad.axes[p.control * 2] = (x * scale * 32767.0).round() as i16;
                pad.axes[p.control * 2 + 1] = (y * scale * 32767.0).round() as i16;
            } else if p.control >= 16 {
                let amount = (1.0 + (p.origin_y - p.y) / (2.0 * c.radius)).clamp(0.0, 1.0);
                pad.axes[p.control - 12] = (amount * 32767.0).round() as i16;
            } else if (p.x - c.x).hypot(p.y - c.y) <= c.radius * 1.5 {
                pad.buttons |= 1 << BUTTON[p.control - 2];
            }
        }
        pad
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn landscape_controls_leave_a_clear_picture_and_preserve_every_hit() {
        // Both Pixel landscape cutout positions, Fold, and a transient resize.
        for rect in [
            [95.0, 0.0, 2090.0, 1014.0],
            [0.0, 66.0, 2090.0, 1014.0],
            [80.0, 20.0, 1952.0, 1080.0],
            [0.0, 0.0, 280.0, 160.0],
        ] {
            let view = Viewport::new(rect, 2.75).unwrap();
            let [x, y, w, h] = view.surface();
            assert!(w > 0.0 && h > 0.0);
            assert!(h > view.height * 0.6, "picture too small for {rect:?}");
            assert!((w / h - 4.0 / 3.0).abs() < 0.0001);
            assert!(x >= view.left && y >= view.top);
            assert!(x + w <= view.left + view.width + 0.01);
            assert!(y + h <= view.top + view.height + 0.01);
            for i in 0..COUNT {
                let c = view.control(i);
                assert!(c.x - c.radius >= view.left - 0.01);
                assert!(c.y - c.radius >= view.top - 0.01);
                assert!(c.x + c.radius <= view.left + view.width + 0.01);
                assert!(c.y + c.radius <= view.top + view.height + 0.01);
                assert!(
                    c.x + c.radius <= x
                        || c.x - c.radius >= x + w
                        || c.y + c.radius <= y
                        || c.y - c.radius >= y + h
                );
                for j in 0..i {
                    let other = view.control(j);
                    assert!(
                        (c.x - other.x).hypot(c.y - other.y) >= c.radius + other.radius - 0.01,
                        "overlap {i}/{j}"
                    );
                }
                let mut touch = Touch::new();
                let p = touch.event_in(1, 0, [c.x, c.y], view);
                assert!(touch.active(i), "wrong control captured for {i}");
                if i >= 16 {
                    assert_eq!(p.axes[i - 12], 32767);
                } else if i >= 2 {
                    assert_eq!(p.buttons, 1 << BUTTON[i - 2]);
                }
                assert_eq!(touch.event_in(1, 3, [c.x, c.y], view), Pad::default());
            }
        }
    }
    #[test]
    fn safe_geometry_contains_whole_controls_and_transforms_hits() {
        for rect in [
            [0.0, 115.0, 2076.0, 1952.0],
            [40.0, 90.0, 290.0, 620.0],
            [80.0, 20.0, 700.0, 300.0],
        ] {
            let v = Viewport::new(rect, 2.4375).unwrap();
            for i in 0..COUNT {
                let c = v.control(i);
                assert!(
                    c.x - c.radius >= v.left - 0.01 && c.x + c.radius <= v.left + v.width + 0.01
                );
                assert!(
                    c.y - c.radius >= v.top - 0.01 && c.y + c.radius <= v.top + v.height + 0.01
                );
            }
            let r = v.surface();
            assert!(r[0] >= v.left && r[1] >= v.top);
            assert!((r[2] / r[3] - 4.0 / 3.0).abs() < 0.0001);
            let mut touch = Touch::new();
            let a = v.control(2);
            assert_ne!(touch.event_in(1, 0, [a.x, a.y], v).buttons & 1, 0);
            touch.clear();
            assert_eq!(touch.pad(v.width, v.height, v.density), Pad::default());
        }
    }
    #[test]
    fn simultaneous_sticks_button_and_trigger_cancel_cleanly() {
        let mut t = Touch::default();
        let (w, h, d) = (852.0, 755.0, 1.0);
        for (id, i) in [(1, 0), (2, 1), (3, 2), (4, 17)] {
            let c = circle(i, w, h, d);
            t.event(id, 0, [c.x, c.y], w, h, d);
        }
        let l = circle(0, w, h, d);
        let r = circle(1, w, h, d);
        t.event(1, 1, [l.x + l.radius, l.y], w, h, d);
        let p = t.event(2, 1, [r.x, r.y - r.radius], w, h, d);
        assert_eq!(p.axes[0], 32767);
        assert_eq!(p.axes[3], -32767);
        assert_eq!(p.axes[5], 32767);
        assert_ne!(p.buttons & 1, 0);
        t.clear();
        assert_eq!(t.pad(w, h, d), Pad::default());
    }
    #[test]
    fn touch_capture_does_not_jump_controls_and_triggers_have_range() {
        let mut t = Touch::default();
        let (w, h, d) = (400.0, 900.0, 1.0);
        let c = circle(16, w, h, d);
        t.event(4, 0, [c.x, c.y], w, h, d);
        let p = t.event(4, 1, [c.x, c.y + c.radius], w, h, d);
        assert!((16000..=17000).contains(&p.axes[4]));
        assert_eq!(p.buttons, 0);
        let p = t.event(4, 2, [c.x, c.y], w, h, d);
        assert_eq!(p, Pad::default());
    }
}
