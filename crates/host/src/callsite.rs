//! Private, bounded executable observation for an already observed title caller.
//! Never exports certificates, complete executables, disc images or live RAM.
use crate::disc::Disc;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

fn invalid() -> io::Error {
    io::Error::other("invalid or unbounded title callsite probe")
}
fn word(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}
fn read(file: &mut File, start: u64, limit: u32, offset: u32, size: u32) -> io::Result<Vec<u8>> {
    if offset.checked_add(size).is_none_or(|end| end > limit) || size > 65536 {
        return Err(invalid());
    }
    let mut bytes = vec![0; size as usize];
    file.seek(SeekFrom::Start(
        start.checked_add(u64::from(offset)).ok_or_else(invalid)?,
    ))?;
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}
struct Section {
    flags: u32,
    address: u32,
    virtual_bytes: u32,
    offset: u32,
    raw_bytes: u32,
}
fn mapped(sections: &[Section], address: u32, size: u32, executable: bool) -> io::Result<u32> {
    let mut matches = sections.iter().filter_map(|s| {
        let relative = address.checked_sub(s.address)?;
        let end = relative.checked_add(size)?;
        if end > s.raw_bytes || end > s.virtual_bytes || (executable && s.flags & 4 == 0) {
            return None;
        }
        s.offset.checked_add(relative)
    });
    let offset = matches.next().ok_or_else(invalid)?;
    if matches.next().is_some() {
        return Err(invalid());
    }
    Ok(offset)
}

/// Two bounded hex fields: virtual start and byte count. All raw evidence stays
/// private. Numeric import slots help inspect indirect calls without copying keys.
pub fn observe(file: &mut File, disc: &Disc, request: &str) -> io::Result<(Vec<u8>, String)> {
    if request.len() > 128 {
        return Err(invalid());
    }
    let fields = request.split_whitespace().collect::<Vec<_>>();
    if fields.len() != 3 || fields[0] != "xbx-callsite-v1" {
        return Err(invalid());
    }
    let address = u32::from_str_radix(fields[1], 16).map_err(|_| invalid())?;
    let count = u32::from_str_radix(fields[2], 16).map_err(|_| invalid())?;
    if count == 0 || count > 32768 || file.metadata()?.len() != disc.bytes {
        return Err(invalid());
    }
    let start = disc
        .partition_offset
        .checked_add(u64::from(disc.executable_sector) * 2048)
        .ok_or_else(invalid)?;
    let limit = disc.executable_bytes;
    if start
        .checked_add(u64::from(limit))
        .is_none_or(|end| end > disc.bytes)
    {
        return Err(invalid());
    }
    let header = read(file, start, limit, 0, 0x178)?;
    if &header[..4] != b"XBEH" {
        return Err(invalid());
    }
    let base = word(&header, 0x104);
    let headers = word(&header, 0x108);
    let image_end = base.checked_add(word(&header, 0x10c)).ok_or_else(invalid)?;
    let sections_at = word(&header, 0x120).checked_sub(base).ok_or_else(invalid)?;
    let sections_count = word(&header, 0x11c);
    if headers > limit || sections_at < 0x178 || sections_count == 0 || sections_count > 256 {
        return Err(invalid());
    }
    let table = read(file, start, headers, sections_at, sections_count * 0x38)?;
    let sections = table
        .chunks_exact(0x38)
        .map(|s| Section {
            flags: word(s, 0),
            address: word(s, 4),
            virtual_bytes: word(s, 8),
            offset: word(s, 12),
            raw_bytes: word(s, 16),
        })
        .collect::<Vec<_>>();
    // Header/certificate ranges can never be requested, even if a malformed
    // section claims to overlap them.
    let offset = mapped(&sections, address, count, true)?;
    if offset < headers {
        return Err(invalid());
    }
    let code = read(file, start, limit, offset, count)?;
    let inside = |va| va >= base && va < image_end;
    let entry = word(&header, 0x128);
    let thunk = word(&header, 0x158);
    let (entry, thunk) = if inside(entry ^ 0xa8fc57ab) && inside(thunk ^ 0x5b6d40b6) {
        (entry ^ 0xa8fc57ab, thunk ^ 0x5b6d40b6)
    } else {
        (entry ^ 0x94859d4b, thunk ^ 0xefb1f152)
    };
    if !inside(entry) || !inside(thunk) {
        return Err(invalid());
    }
    let mut metadata = format!(
        "xbx-callsite-evidence-v1\naddress={address:08x}\nbytes={count}\nentry={entry:08x}\n"
    );
    for index in 0..4096u32 {
        let slot = thunk.checked_add(index * 4).ok_or_else(invalid)?;
        let offset = mapped(&sections, slot, 4, false)?;
        let value = word(&read(file, start, limit, offset, 4)?, 0);
        if value == 0 {
            return Ok((code, metadata));
        }
        if value & 0xffff0000 != 0x80000000 || value & 0xffff == 0 {
            return Err(invalid());
        }
        metadata.push_str(&format!("import={slot:08x}:{}\n", value & 0xffff));
    }
    Err(invalid())
}

/// Explicit diagnostic-root opt-in, before native initialization or writes.
/// Existing results are preserved; partial output is refused on the next run.
pub fn requested(root: &Path, file: &mut File, disc: &Disc) -> io::Result<bool> {
    let path = root.join("callsite.request");
    if !path.try_exists()? {
        return Ok(false);
    }
    if !root.join("branch.identity").is_file()
        || !fs::symlink_metadata(&path)?.is_file()
        || fs::metadata(&path)?.len() > 128
    {
        return Err(invalid());
    }
    let (code, metadata) = observe(file, disc, &fs::read_to_string(path)?)?;
    let output = root.join("callsite-evidence");
    fs::create_dir(&output)?;
    fs::set_permissions(&output, fs::Permissions::from_mode(0o700))?;
    for (name, bytes) in [("code", code.as_slice()), ("map", metadata.as_bytes())] {
        let mut target = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(output.join(name))?;
        target.write_all(bytes)?;
        target.sync_all()?;
    }
    File::open(output)?.sync_all()?;
    File::open(root)?.sync_all()?;
    Ok(true)
}
