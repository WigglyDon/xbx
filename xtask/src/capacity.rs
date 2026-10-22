use std::fs::{self, File, OpenOptions};
use std::path::Path;
use std::process::Command;

const GIB: u64 = 1024 * 1024 * 1024;

fn output(command: &mut Command) -> Result<String, Box<dyn std::error::Error>> {
    let result = command.output()?;
    if !result.status.success() {
        return Err("capacity accounting command failed".into());
    }
    Ok(String::from_utf8(result.stdout)?)
}

pub fn check(root: &Path, growth_gib: u64) -> Result<(), Box<dyn std::error::Error>> {
    let df = output(Command::new("df").args(["-B1", "--output=avail"]).arg(root))?;
    let available: u64 = df
        .lines()
        .last()
        .ok_or("missing free bytes")?
        .trim()
        .parse()?;
    if available < (12 + growth_gib) * GIB {
        return Err("growth refused: preserve 12 GiB plus the operation's estimated peak".into());
    }
    let fs_type = output(
        Command::new("findmnt")
            .args(["-n", "-o", "FSTYPE", "-T"])
            .arg(root),
    )?;
    if fs_type.trim() == "btrfs" {
        let usage = output(
            Command::new("btrfs")
                .args(["filesystem", "usage", "-b"])
                .arg(root),
        )?;
        let unallocated: u64 = usage
            .lines()
            .find_map(|line| line.trim().strip_prefix("Device unallocated:"))
            .ok_or("missing device-unallocated accounting")?
            .trim()
            .parse()?;
        let reserve = usage
            .lines()
            .find_map(|line| line.trim().strip_prefix("Global reserve:"))
            .ok_or("missing global-reserve accounting")?;
        let reserve_bytes: u64 = reserve
            .split_whitespace()
            .next()
            .ok_or("missing reserve size")?
            .parse()?;
        let used: u64 = reserve
            .split("used:")
            .nth(1)
            .ok_or("missing reserve use")?
            .trim()
            .trim_end_matches(')')
            .parse()?;
        if used != 0 {
            return Err("growth refused: Btrfs emergency reserve is in use".into());
        }
        let metadata = usage
            .lines()
            .find(|line| line.starts_with("Metadata,"))
            .ok_or("missing metadata accounting")?;
        let number = |key: &str| -> Result<u64, Box<dyn std::error::Error>> {
            let field = metadata.split(key).nth(1).ok_or("missing metadata field")?;
            Ok(field
                .split([',', ' ', '\t'])
                .find(|s| !s.is_empty())
                .ok_or("missing metadata value")?
                .parse()?)
        };
        let usable = number("Size:")?
            .saturating_sub(number("Used:")?)
            .saturating_sub(reserve_bytes);
        let floor = if unallocated < GIB { GIB } else { GIB / 2 };
        if usable < floor {
            return Err(
                "growth refused: insufficient usable Btrfs metadata outside emergency reserve"
                    .into(),
            );
        }
        println!(
            "Capacity: {} MiB available, {} MiB usable metadata; emergency reserve unused.",
            available / 1048576,
            usable / 1048576
        );
    }
    Ok(())
}

pub fn writer(root: &Path, growth_gib: u64) -> Result<File, Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join(".local"))?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join(".local/build.lock"))?;
    lock.try_lock()
        .map_err(|_| "another xbx build/source writer is active")?;
    check(root, growth_gib)?;
    Ok(lock)
}
