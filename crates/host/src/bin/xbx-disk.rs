// SPDX-License-Identifier: MIT
//! Packaged Android executable, separate from the live machine. QEMU decodes
//! its own disk format; Rust hashes the same canonical logical blocks as host.
#[cfg(target_os = "android")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};
    use std::{
        ffi::{CString, c_char, c_void},
        os::unix::ffi::OsStrExt,
    };
    #[link(name = "xemu")]
    unsafe extern "C" {
        fn xbx_image_open(path: *const c_char) -> *mut c_void;
        fn xbx_image_length(image: *mut c_void) -> i64;
        fn xbx_image_read(image: *mut c_void, offset: u64, bytes: *mut u8, size: usize) -> i32;
        fn xbx_image_close(image: *mut c_void);
    }
    struct Image(*mut c_void);
    impl Drop for Image {
        fn drop(&mut self) {
            unsafe { xbx_image_close(self.0) }
        }
    }
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 || args[1] != "digest" {
        return Err("usage: xbx-disk digest PRIVATE_QUIESCED_DISK".into());
    }
    let path = CString::new(args[2].as_bytes())?;
    let image = Image(unsafe { xbx_image_open(path.as_ptr()) });
    if image.0.is_null() {
        std::mem::forget(image);
        return Err("native read-only image open failed".into());
    }
    let len = unsafe { xbx_image_length(image.0) };
    if len != xbx::disk::DISK_BYTES as i64 {
        return Err("generated storage logical size differs; preserve it".into());
    }
    let mut hash = Sha256::new();
    hash.update((len as u64).to_le_bytes());
    let mut bytes = vec![0; 65536];
    let started = std::time::Instant::now();
    for offset in (0..len as u64).step_by(bytes.len()) {
        if started.elapsed().as_secs() > 90 {
            return Err("read-only logical identity deadline".into());
        }
        let size = ((len as u64 - offset) as usize).min(bytes.len());
        if unsafe { xbx_image_read(image.0, offset, bytes.as_mut_ptr(), size) } < 0 {
            return Err("native logical read failed".into());
        }
        if bytes[..size].iter().any(|b| *b != 0) {
            hash.update(offset.to_le_bytes());
            hash.update((size as u64).to_le_bytes());
            hash.update(&bytes[..size]);
        }
    }
    println!("{hash:x}", hash = hash.finalize());
    Ok(())
}
#[cfg(not(target_os = "android"))]
fn main() {
    eprintln!("xbx-disk is the Android native-library executable; host uses its retained qemu-img");
}
