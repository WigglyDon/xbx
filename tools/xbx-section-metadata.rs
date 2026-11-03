// SPDX-License-Identifier: MIT
// Read-only qualification adapter. Only structural image fields are emitted;
// no instructions, section payloads, names, certificate strings or key fields.
#[allow(dead_code)]
#[path = "../crates/host/src/disc.rs"]
mod disc;
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
};

fn word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err(io::Error::other("expected one read-only image input"));
    }
    let mut file = File::open(Path::new(&args[1]))?;
    let info = disc::inspect_open(&mut file)?;
    let exe = info.partition_offset + u64::from(info.executable_sector) * 2048;
    let mut header = [0u8; 0x124];
    file.seek(SeekFrom::Start(exe))?;
    file.read_exact(&mut header)?;
    if &header[..4] != b"XBEH" {
        return Err(io::Error::other("invalid executable header"));
    }
    let base = word(&header, 0x104);
    let headers = word(&header, 0x108);
    let image = word(&header, 0x10c);
    let count = word(&header, 0x11c);
    let table = word(&header, 0x120)
        .checked_sub(base)
        .ok_or_else(|| io::Error::other("section table precedes image"))?;
    if count == 0
        || count > 64
        || u64::from(table) + u64::from(count) * 56 > u64::from(headers)
        || headers > info.executable_bytes
        || image == 0
    {
        return Err(io::Error::other("invalid bounded section table"));
    }
    let mut rows = Vec::new();
    for index in 0..count {
        let mut entry = [0u8; 20];
        file.seek(SeekFrom::Start(
            exe + u64::from(table) + u64::from(index) * 56,
        ))?;
        file.read_exact(&mut entry)?;
        let flags = word(&entry, 0);
        let address = word(&entry, 4);
        let size = word(&entry, 8);
        let raw = word(&entry, 12);
        let bytes = word(&entry, 16);
        if address < base
            || u64::from(address) + u64::from(size) > u64::from(base) + u64::from(image)
            || u64::from(raw) + u64::from(bytes) > u64::from(info.executable_bytes)
        {
            return Err(io::Error::other("section outside declared image"));
        }
        rows.push(format!("{{\"index\":{index},\"flags\":{flags},\"address\":{address},\"size\":{size},\"raw\":{raw},\"bytes\":{bytes}}}"));
    }
    println!(
        "{{\"source_bytes\":{},\"executable_file_offset\":{exe},\"base\":{base},\"image_bytes\":{image},\"sections\":[{}]}}",
        info.bytes,
        rows.join(",")
    );
    Ok(())
}
