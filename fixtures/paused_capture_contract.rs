// SPDX-License-Identifier: MIT
//! Original HMP ownership contract. Actual extracted public wrapper is linked.
//! Native integration remains the device-state proof; this fixture controls its
//! stop/start/notification dependency and error channel under a serialized owner.
use std::{
    ffi::{c_char, c_void},
    sync::Mutex,
};
const RUNNING: i32 = 1;
const PAUSED: i32 = 2;
const SAVE: i32 = 3;
#[derive(Default)]
struct State {
    current: i32,
    prepared: u32,
    failed: bool,
    error_observed: bool,
    captured_prepared: u32,
}
static STATE: Mutex<State> = Mutex::new(State {
    current: PAUSED,
    prepared: 0,
    failed: false,
    error_observed: false,
    captured_prepared: 0,
});
unsafe extern "C" {
    fn original_capture();
}
#[unsafe(no_mangle)]
extern "C" fn runstate_get() -> i32 {
    STATE.lock().unwrap().current
}
#[unsafe(no_mangle)]
extern "C" fn runstate_is_running() -> bool {
    runstate_get() == RUNNING
}
#[unsafe(no_mangle)]
extern "C" fn vm_start() {
    STATE.lock().unwrap().current = RUNNING;
}
#[unsafe(no_mangle)]
extern "C" fn vm_stop(state: i32) -> i32 {
    let mut s = STATE.lock().unwrap();
    if s.current == RUNNING {
        s.current = state;
        if state == SAVE {
            s.prepared += 1;
        }
    }
    0
}
#[unsafe(no_mangle)]
unsafe extern "C" fn save_snapshot(
    _name: *const c_char,
    _overwrite: bool,
    _vm: *const c_char,
    _has: bool,
    _devices: *mut c_void,
    error: *mut *mut c_void,
) -> bool {
    let prior = runstate_get();
    vm_stop(SAVE);
    let mut s = STATE.lock().unwrap();
    s.captured_prepared = s.prepared;
    s.current = prior;
    if s.failed {
        unsafe { *error = std::ptr::dangling_mut::<u8>().cast() };
        false
    } else {
        true
    }
}
#[unsafe(no_mangle)]
extern "C" fn hmp_handle_error(_mon: *mut c_void, error: *mut c_void) {
    STATE.lock().unwrap().error_observed = !error.is_null();
}
fn run(current: i32, failed: bool) {
    *STATE.lock().unwrap() = State {
        current,
        failed,
        ..State::default()
    };
    unsafe { original_capture() };
    let (actual, prepared, error) = {
        let s = STATE.lock().unwrap();
        (s.current, s.captured_prepared, s.error_observed)
    };
    assert_eq!(actual, current, "capture must preserve run/pause intent");
    assert_eq!(
        prepared, 1,
        "device SAVE_VM preparation must precede serialization"
    );
    assert_eq!(error, failed, "native failure must remain visible");
}
#[test]
fn running_control_prepares_devices() {
    run(RUNNING, false)
}
#[test]
fn already_paused_capture_prepares_devices_and_stays_paused() {
    run(PAUSED, false)
}
#[test]
fn already_paused_failure_preserves_pause_and_error() {
    run(PAUSED, true)
}
