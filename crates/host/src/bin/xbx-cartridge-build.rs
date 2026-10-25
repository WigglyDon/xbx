// SPDX-License-Identifier: MIT
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() != 4 {
        return Err("usage: xbx-cartridge-build REPO WORKSPACE ACTION".into());
    }
    xbx::cartridge::build::command(
        std::path::Path::new(&a[1]),
        std::path::Path::new(&a[2]),
        &a[3],
    )
}
