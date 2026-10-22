//! Reads only the separately built kernel's fixed-size public-API result journal.
//! Never reads game code, arguments, buffers, paths or call-site addresses.
use crate::probe::{hmp_words, map_symbol, response};
use std::io::{BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

/// Acceptance for the original three-call synthetic guest only, never a game.
pub fn validate_fixture(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path)?;
    let entries: Vec<_> = text
        .lines()
        .filter(|line| line.as_bytes().first().is_some_and(u8::is_ascii_digit))
        .map(|line| line.split_ascii_whitespace().collect::<Vec<_>>())
        .collect();
    if !text.lines().any(|line| line == "count=3")
        || entries.len() != 3
        || entries[0].get(..3) != Some(&["1", "285", "1"])
        || entries[1] != ["2", "269", "1", "00000040"]
        || entries[2] != ["3", "307", "1", "04030201"]
    {
        return Err("original API journal fixture did not reach all checked calls".into());
    }
    println!(
        "Original guest fill/read/canaries/ABI and three recorded returns passed. Native termination is separate."
    );
    Ok(())
}

pub fn verify_binding(map: &Path, flash: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let receipt = std::fs::read_to_string(map.with_file_name("build.identity"))?;
    let lock = include_str!("../../../kernel-journal.lock.json");
    let field = |key: &str| -> Result<&str, Box<dyn std::error::Error>> {
        let prefix = format!("{key}=");
        let mut values = receipt
            .lines()
            .filter_map(|line| line.strip_prefix(&prefix));
        let value = values.next().ok_or("missing journal build binding")?;
        if values.next().is_some() {
            return Err("duplicate journal build binding".into());
        }
        Ok(value)
    };
    for (key, expected) in [
        ("mode", "journal"),
        ("image", "open"),
        ("selftest", "0"),
        ("trace_imports", "1"),
        ("launch_trace", "1"),
        ("ram_mib", "64"),
    ] {
        if field(key)? != expected {
            return Err("wrong API journal build mode".into());
        }
    }
    for (key, lock_key) in [
        ("source", "commit"),
        ("cromwell", "cromwell_commit"),
        ("patch_sha256", "patch_sha256"),
    ] {
        if !lock.contains(&format!("\"{lock_key}\": \"{}\"", field(key)?)) {
            return Err("unbound journal source/patch".into());
        }
    }
    for (path, key) in [(map, "map_sha256"), (flash, "firmware_sha256")] {
        let output = std::process::Command::new("sha256sum").arg(path).output()?;
        if !output.status.success()
            || String::from_utf8(output.stdout)?.split_whitespace().next() != Some(field(key)?)
        {
            return Err("journal map/firmware identity mismatch".into());
        }
    }
    Ok(())
}

fn kernel_range(address: u64, bytes: u64) -> Result<(), Box<dyn std::error::Error>> {
    if address & 3 != 0
        || address < 0x80010000
        || address
            .checked_add(bytes)
            .is_none_or(|end| end > 0xc0000000)
    {
        return Err("journal address outside bounded kernel range".into());
    }
    Ok(())
}
fn read(
    socket: &mut UnixStream,
    reader: &mut BufReader<UnixStream>,
    address: u64,
    words: usize,
) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    if words == 0 || words > 128 {
        return Err("journal read bound".into());
    }
    kernel_range(address, words as u64 * 4)?;
    writeln!(
        socket,
        "{{\"execute\":\"human-monitor-command\",\"arguments\":{{\"command-line\":\"x/{words}wx 0x{address:x}\"}}}}"
    )?;
    let result = hmp_words(&response(reader)?);
    if result.len() != words {
        return Err("incomplete kernel journal read".into());
    }
    Ok(result)
}

pub fn capture(
    socket: &mut UnixStream,
    reader: &mut BufReader<UnixStream>,
    map: &Path,
    output: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let map = std::fs::read_to_string(map)?;
    let count_at = map_symbol(&map, "_XkTraceCount")?;
    let ring_at = map_symbol(&map, "_XkTraceRing")?;
    let count = read(socket, reader, count_at, 1)?[0];
    let ring = read(socket, reader, ring_at, 1)?[0] as u64;
    if count == 0 || ring == 0 {
        return Err("kernel API journal unavailable".into());
    }
    kernel_range(ring, 1024 * 16)?;
    let mut entries = Vec::new();
    for block in 0..32 {
        let words = read(socket, reader, ring + block * 128 * 4, 128)?;
        for e in words.chunks_exact(4) {
            if e[0] > count.saturating_sub(1024)
                && e[0] <= count
                && (1..=366).contains(&e[1])
                && e[2] <= 1
            {
                entries.push([e[0], e[1], e[2], e[3]]);
            }
        }
    }
    entries.sort_unstable_by_key(|e| e[0]);
    let mut text = format!(
        "schema=xbx-kernel-api-journal-v1\ncount={count}\nretained={}\n# sequence ordinal returned eax_hex (private numeric results, no arguments)\n",
        entries.len()
    );
    for e in entries {
        text.push_str(&format!("{} {} {} {:08x}\n", e[0], e[1], e[2], e[3]));
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_never_follows_a_title_pointer_or_overflows_kernel_range() {
        for (address, bytes) in [
            (0, 4),
            (0x10000, 16384),
            (0x80010001, 4),
            (0xbffffffc, 8),
            (u64::MAX - 3, 16),
        ] {
            assert!(kernel_range(address, bytes).is_err());
        }
        assert!(kernel_range(0x80010000, 4).is_ok());
        assert!(kernel_range(0xbfffc000, 16384).is_ok());
    }
}
