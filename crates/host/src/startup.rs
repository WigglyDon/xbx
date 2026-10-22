//! Successful machine initialization and display readiness are separate facts.
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

pub struct Startup {
    machine: AtomicI32,
    display: AtomicBool,
    foreground: AtomicBool,
    finished: AtomicBool,
}

impl Default for Startup {
    fn default() -> Self {
        Self {
            machine: AtomicI32::new(0),
            display: AtomicBool::new(false),
            foreground: AtomicBool::new(true),
            finished: AtomicBool::new(false),
        }
    }
}

impl Startup {
    pub const fn new() -> Self {
        Self {
            machine: AtomicI32::new(0),
            display: AtomicBool::new(false),
            foreground: AtomicBool::new(true),
            finished: AtomicBool::new(false),
        }
    }
    pub fn report(&self, status: i32) {
        // A rejected profile cannot be revived by a later display notification.
        let _ = self
            .machine
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |old| {
                if old < 0 { None } else { Some(status) }
            });
    }
    pub fn display_ready(&self) {
        self.display.store(true, Ordering::SeqCst);
    }
    pub fn foreground(&self, value: bool) {
        self.foreground.store(value, Ordering::SeqCst);
    }
    pub fn finish(&self) {
        self.finished.store(true, Ordering::SeqCst);
    }
    pub fn ready(&self) -> bool {
        self.machine.load(Ordering::SeqCst) == 1
            && self.display.load(Ordering::SeqCst)
            && !self.finished.load(Ordering::SeqCst)
    }
    pub fn may_resume(&self) -> bool {
        self.ready() && self.foreground.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejected_machine_stays_rejected_across_display_and_resume() {
        let state = Startup::new();
        state.foreground(false);
        state.report(-1);
        state.display_ready();
        state.foreground(true);
        state.report(1);
        assert!(!state.ready());
        assert!(!state.may_resume());
    }
    #[test]
    fn accepted_display_and_foreground_are_all_required() {
        let state = Startup::new();
        state.report(1);
        assert!(!state.may_resume());
        state.foreground(false);
        state.display_ready();
        assert!(!state.may_resume());
        state.foreground(true);
        assert!(state.may_resume());
        state.finish();
        assert!(!state.may_resume());
    }
}
