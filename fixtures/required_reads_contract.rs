// SPDX-License-Identifier: MIT
// Original synthetic read-completion contract against the actual public function.
use std::sync::{
    Mutex,
    atomic::{AtomicI32, AtomicU32, Ordering},
};
static LOCK: Mutex<()> = Mutex::new(());
static TRANSFER: AtomicU32 = AtomicU32::new(0);
static STATUS: AtomicI32 = AtomicI32::new(0);
#[repr(C)]
struct ReadResult {
    status: i32,
    head: u32,
    tail: u32,
    references: u32,
}
#[link(name = "required_reads", kind = "static")]
unsafe extern "C" {
    fn fixture_section() -> ReadResult;
}
#[unsafe(no_mangle)]
unsafe extern "C" fn fixture_read(dst: *mut u8, len: u32, got: *mut u32) -> i32 {
    let n = TRANSFER.load(Ordering::SeqCst);
    unsafe {
        got.write(n);
        std::ptr::write_bytes(dst, 0x5a, n.min(len) as usize);
    }
    STATUS.load(Ordering::SeqCst)
}
fn run(n: u32, status: i32) -> ReadResult {
    TRANSFER.store(n, Ordering::SeqCst);
    STATUS.store(status, Ordering::SeqCst);
    unsafe { fixture_section() }
}
#[test]
fn successful_short_read_is_not_a_complete_section() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let r = run(4, 0);
    assert_eq!(
        r.status, 0xc0000011u32 as i32,
        "short successful section read was accepted"
    );
    assert_eq!((r.head, r.tail, r.references), (3, 5, 0));
}
#[test]
fn full_read_publishes_section_and_references() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let r = run(8, 0);
    assert_eq!((r.status, r.head, r.tail, r.references), (0, 4, 6, 1));
}
#[test]
fn underlying_io_failure_is_preserved_and_unpublishes_section() {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let status = 0xc0000022u32 as i32;
    let r = run(0, status);
    assert_eq!((r.status, r.head, r.tail, r.references), (status, 3, 5, 0));
}
