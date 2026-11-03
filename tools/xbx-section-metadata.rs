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
// At most 28 KiB of descriptor table, of which only 10 KiB of structural
// fields are read. Names, digests and section payloads remain unread.
const MAX_SECTION_RECORDS: u32 = 512;
fn describe(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
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
        || count > MAX_SECTION_RECORDS
        || u64::from(table) + u64::from(count) * 56 > u64::from(headers)
        || headers > info.executable_bytes
        || image == 0
    {
        return Err(io::Error::other(format!(
            "invalid bounded section table: count={count} table={table} headers={headers} executable={} image={image} maximum={MAX_SECTION_RECORDS}",
            info.executable_bytes
        )));
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
    Ok(format!(
        "{{\"source_bytes\":{},\"executable_file_offset\":{exe},\"base\":{base},\"image_bytes\":{image},\"sections\":[{}]}}",
        info.bytes,
        rows.join(",")
    ))
}
fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err(io::Error::other("expected one read-only image input"));
    }
    println!("{}", describe(Path::new(&args[1]))?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Write, process::Command};
    fn fixture(name: &str, count: u32, headers: u32, table: u32) -> std::path::PathBuf {
        let original =
            std::env::var_os("XBX_ORIGINAL_METADATA_FIXTURE").expect("original fixture required");
        let mut data = fs::read(original).unwrap();
        let directory = 33 * 2048;
        let exe = 34 * 2048;
        assert_eq!(&data[exe..exe + 4], b"XBEH");
        let put = |data: &mut [u8], at: usize, value: u32| {
            data[at..at + 4].copy_from_slice(&value.to_le_bytes())
        };
        put(&mut data, directory + 8, 0xA000);
        put(&mut data, exe + 0x108, headers);
        put(&mut data, exe + 0x10C, 0x10000);
        put(&mut data, exe + 0x11C, count);
        put(&mut data, exe + 0x120, 0x10000 + table);
        for i in 0..count {
            let at = exe + table as usize + i as usize * 56;
            data[at..at + 56].fill(0);
            put(&mut data, at + 4, 0x10000 + headers);
        }
        let at = exe + table as usize;
        put(&mut data, at, 4);
        put(&mut data, at + 4, 0x11000);
        put(&mut data, at + 8, 4096);
        put(&mut data, at + 12, 0x1000);
        let root = std::path::PathBuf::from(
            std::env::var_os("XBX_METADATA_TEST_OUTPUT").expect("test output required"),
        );
        fs::create_dir_all(&root).unwrap();
        let path = root.join(name);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        file.write_all(&data).unwrap();
        file.sync_all().unwrap();
        path
    }
    #[test]
    fn original_65_sections_legacy_refuses_current_describes() {
        let path = fixture("valid-65.iso", 65, 0x8000, 0x1000);
        let legacy =
            std::env::var_os("XBX_OLD_METADATA_TOOL").expect("retained legacy tool required");
        let result = Command::new(legacy).arg(&path).output().unwrap();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("invalid bounded section table"));
        let result = describe(&path).unwrap();
        assert_eq!(result.matches("\"index\":").count(), 65);
        assert!(result.contains("\"address\":69632,\"size\":4096,\"raw\":4096,\"bytes\":0"));
        assert!(result.len() < 128 * 1024);
    }
    #[test]
    fn refuses_table_outside_declared_headers() {
        let path = fixture("outside-header.iso", 2, 0x8000, 0x7ff0);
        assert!(
            describe(&path)
                .unwrap_err()
                .to_string()
                .contains("invalid bounded section table")
        );
    }
    #[test]
    fn refuses_above_declared_metadata_budget() {
        let path = fixture("over-budget.iso", 513, 0x9000, 0x1000);
        assert!(
            describe(&path)
                .unwrap_err()
                .to_string()
                .contains("maximum=512")
        );
    }
}
