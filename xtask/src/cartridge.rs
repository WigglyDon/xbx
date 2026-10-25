// SPDX-License-Identifier: MIT
//! Admission stays with the existing workspace; the Rust host owns packaging.
use crate::ramht_workspace::Workspace;
use std::{path::Path, process::Command};
pub fn command(repo: &Path, action: &str, attempt: &str) -> Result<(), Box<dyn std::error::Error>> {
    let w = Workspace::acquire_cartridge(repo, action)?;
    let builder = std::env::var_os("XBX_CARTRIDGE_BUILDER")
        .ok_or("set XBX_CARTRIDGE_BUILDER to the isolated Rust build owner")?;
    if !w
        .run(
            Command::new(builder).arg(repo).arg(&w.root).arg(action),
            &w.root.join(format!("{action}-{attempt}.log")),
            600,
        )?
        .success()
    {
        return Err("cartridge build failed; source/log preserved".into());
    }
    w.check()?;
    w.completed();
    Ok(())
}
