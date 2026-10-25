// SPDX-License-Identifier: MIT
//! Admission stays with the existing workspace; the Rust host owns packaging.
use crate::ramht_workspace::Workspace;
use std::{path::Path, process::Command};
pub fn command(repo: &Path, action: &str, attempt: &str) -> Result<(), Box<dyn std::error::Error>> {
    let w = Workspace::acquire_cartridge(repo, action)?;
    if action == "cartridge-checks" {
        if attempt.is_empty()
            || !attempt
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("invalid isolated source label".into());
        }
        let source = w.root.join(format!("tools-{attempt}/source"));
        let checks: Vec<Vec<&str>> = vec![
            vec![
                "test",
                "--locked",
                "--offline",
                "-j",
                "1",
                "-p",
                "xbx-host",
                "--lib",
                "cartridge::tests",
                "--",
                "--test-threads=1",
            ],
            vec![
                "clippy",
                "--locked",
                "--offline",
                "-j",
                "1",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
            vec!["fmt", "--all", "--", "--check"],
        ];
        for (i, args) in checks.iter().enumerate() {
            if !w
                .run(
                    Command::new("cargo")
                        .args(args)
                        .current_dir(&source)
                        .env("CARGO_TARGET_DIR", w.root.join("rust-target"))
                        .env("CARGO_INCREMENTAL", "0")
                        .env("CARGO_PROFILE_DEV_DEBUG", "0")
                        .env("CARGO_PROFILE_TEST_DEBUG", "0"),
                    &w.root.join(format!("checks-{attempt}-rust-{i}.log")),
                    420,
                )?
                .success()
            {
                return Err("focused checks failed; exact source/log retained".into());
            }
        }
        w.check()?;
        w.completed();
        return Ok(());
    }
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
