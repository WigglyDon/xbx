//! Original sparse, empty retail-layout FATX disk fixture. Never format an existing file.
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub const DISK_BYTES: u64 = 8 * 1024 * 1024 * 1024;
pub const PARTITIONS: [(&str, u64, u64); 5] = [
    ("C", 0x465400, 0x1f400000),
    ("E", 0x55f400, 0x1312d6000),
    ("X", 0x400, 0x2ee00000),
    ("Y", 0x177400, 0x2ee00000),
    ("Z", 0x2ee400, 0x2ee00000),
];

pub fn create_empty_raw(path: &Path) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.set_len(DISK_BYTES)?;
    let mut random = File::open("/dev/urandom")?;
    for (name, lba, bytes) in PARTITIONS {
        let start = lba * 512;
        let cluster = 32 * 512;
        let entries = bytes / cluster + 1;
        let width = if entries < 0xfff0 { 2 } else { 4 };
        let fat_bytes = (entries * width).div_ceil(4096) * 4096;
        if start + bytes > DISK_BYTES || 4096 + fat_bytes + cluster > bytes {
            return Err(io::Error::other("invalid empty disk layout"));
        }
        let mut header = [0u8; 4096];
        header[..4].copy_from_slice(b"FATX");
        random.read_exact(&mut header[4..8])?; // volume serial, never console identity/key material
        header[8..12].copy_from_slice(&32u32.to_le_bytes());
        header[12..16].copy_from_slice(&1u32.to_le_bytes());
        file.seek(SeekFrom::Start(start))?;
        file.write_all(&header)?;
        // Reserved FAT slot and the allocated end-of-chain root cluster.
        if width == 2 {
            file.write_all(&[0xf8, 0xff, 0xff, 0xff])?;
        } else {
            file.write_all(&[0xf8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff])?;
        }
        if name == "E" {
            file.write_all(&[0xff; 8])?;
        }
        file.seek(SeekFrom::Start(start + 4096 + fat_bytes))?;
        let mut root = vec![0xff; cluster as usize];
        if name == "E" {
            for (i, dir) in [b"TDATA", b"UDATA"].iter().enumerate() {
                let entry = &mut root[i * 64..(i + 1) * 64];
                entry[0] = 5;
                entry[1] = 0x10;
                entry[2..7].copy_from_slice(*dir);
                entry[44..48].copy_from_slice(&((i + 2) as u32).to_le_bytes());
                entry[48..52].copy_from_slice(&0u32.to_le_bytes());
                for offset in [52, 56, 60] {
                    entry[offset..offset + 4].copy_from_slice(&0x00210000u32.to_le_bytes());
                }
            }
        }
        file.write_all(&root)?;
        if name == "E" {
            file.write_all(&vec![0xff; 2 * cluster as usize])?;
        }
    }
    file.sync_all()?;
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}
