//! Original one-pad touch model. Captures survive moves; cancellation neutralizes all channels.
use std::collections::BTreeMap;

pub const COUNT: usize = 18;
pub const LABELS: [&str; COUNT] = [
    "L", "R", "A", "B", "X", "Y", "↑", "↓", "←", "→", "Back", "Start", "White", "Black", "LS",
    "RS", "LT", "RT",
];
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

fn band(h: f32, density: f32) -> f32 {
    (h * 0.48).min(260.0 * density.max(0.5))
}
pub fn game_rect(w: f32, h: f32, density: f32) -> [f32; 4] {
    let available = (h - band(h, density)).max(1.0);
    let height = available.min(w * 0.75);
    let width = height * 4.0 / 3.0;
    [(w - width) * 0.5, (available - height) * 0.5, width, height]
}
pub fn circle(index: usize, w: f32, h: f32, density: f32) -> Circle {
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
    Circle {
        x: x * w,
        y: top + y * b,
        radius: if index < 2 {
            (b * 0.18).min(w * 0.12)
        } else {
            (w * 0.052).min(b * 0.085)
        },
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
