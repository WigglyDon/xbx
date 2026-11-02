//! Successful machine initialization and display readiness are separate facts.
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

/// Recognizes the guest boot-failure marker across arbitrary serial reads.
/// The caller supplies the dedicated guest stream, before host log mixing.
#[derive(Default)]
pub struct GuestBootFailureDetector {
    suffix: Vec<u8>,
}
impl GuestBootFailureDetector {
    pub fn observe(&mut self, data: &[u8]) -> bool {
        const MARKER: &[u8] = b"*** fatal error ";
        let keep = MARKER.len() - 1;
        self.suffix.extend_from_slice(&data[..data.len().min(keep)]);
        let found = self.suffix.windows(MARKER.len()).any(|w| w == MARKER)
            || data.windows(MARKER.len()).any(|w| w == MARKER);
        if data.len() >= keep {
            self.suffix.clear();
            self.suffix.extend_from_slice(&data[data.len() - keep..]);
        } else {
            let discard = self.suffix.len().saturating_sub(keep);
            self.suffix.drain(..discard);
        }
        found
    }
}

pub struct Startup {
    machine: AtomicI32,
    display: AtomicBool,
    foreground: AtomicBool,
    finished: AtomicBool,
    guest_failed: AtomicBool,
}

impl Default for Startup {
    fn default() -> Self {
        Self {
            machine: AtomicI32::new(0),
            display: AtomicBool::new(false),
            foreground: AtomicBool::new(true),
            finished: AtomicBool::new(false),
            guest_failed: AtomicBool::new(false),
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
            guest_failed: AtomicBool::new(false),
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
    pub fn report_guest_boot_failure(&self) -> bool {
        !self.guest_failed.swap(true, Ordering::SeqCst)
    }
    pub fn guest_boot_failed(&self) -> bool {
        self.guest_failed.load(Ordering::SeqCst)
    }
    pub fn ready(&self) -> bool {
        self.machine.load(Ordering::SeqCst) == 1
            && self.display.load(Ordering::SeqCst)
            && !self.finished.load(Ordering::SeqCst)
            && !self.guest_boot_failed()
    }
    pub fn may_resume(&self) -> bool {
        self.ready() && self.foreground.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guest_marker_survives_every_serial_split() {
        let marker = b"*** fatal error ";
        for split in 1..marker.len() {
            let mut detector = GuestBootFailureDetector::default();
            assert!(!detector.observe(&marker[..split]));
            assert!(detector.observe(&marker[split..]));
        }
        let mut detector = GuestBootFailureDetector::default();
        for (i, byte) in marker.iter().enumerate() {
            assert_eq!(detector.observe(&[*byte]), i == marker.len() - 1);
        }
    }
    #[test]
    fn unrelated_text_and_overlapping_prefix_do_not_lose_the_marker() {
        let mut detector = GuestBootFailureDetector::default();
        assert!(!detector.observe(b"native initialization failed\n"));
        assert!(!detector.observe(b"ordinary guest output***"));
        assert!(detector.observe(b"* fatal error 99\n"));
    }
    #[test]
    fn guest_failure_is_distinct_from_native_status_and_stays_unready() {
        let native = Startup::new();
        native.report(-4);
        assert!(!native.guest_boot_failed());
        let guest = Startup::new();
        guest.report(1);
        guest.display_ready();
        assert!(guest.ready());
        assert!(guest.report_guest_boot_failure());
        assert!(!guest.report_guest_boot_failure());
        guest.report(1);
        assert!(guest.guest_boot_failed());
        assert!(!guest.ready());
        assert!(!guest.may_resume());
    }
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
