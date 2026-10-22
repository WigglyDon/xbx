//! Bounded, read-only XDVDFS volume qualification (not a game compatibility claim).
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

const SECTOR: u64 = 2048;
const MAGIC: &[u8; 20] = b"MICROSOFT*XBOX*MEDIA";

#[derive(Debug, PartialEq, Eq)]
pub struct Disc {
    pub partition_offset: u64,
    pub root_sector: u32,
    pub root_bytes: u32,
    pub bytes: u64,
    pub executable_sector: u32,
    pub executable_bytes: u32,
}

pub fn inspect(path: &Path) -> io::Result<Disc> {
    let mut file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(io::Error::other("disc must be a regular seekable file"));
    }
    let bytes = file.metadata()?.len();
    // XISO, original-Xbox full-disc partitions, and XGD3/XGD2 game partitions.
    // A header match alone never establishes that a title is original-Xbox software.
    for partition_offset in [0, 0x18300000, 0xfd90000, 0x2080000] {
        let offset = partition_offset + 32 * SECTOR;
        if offset + SECTOR > bytes {
            continue;
        }
        file.seek(SeekFrom::Start(offset))?;
        let mut header = [0u8; SECTOR as usize];
        file.read_exact(&mut header)?;
        if &header[..20] != MAGIC || &header[2028..] != MAGIC {
            continue;
        }
        let root_sector = u32::from_le_bytes(header[20..24].try_into().unwrap());
        let root_bytes = u32::from_le_bytes(header[24..28].try_into().unwrap());
        let root_end = partition_offset + u64::from(root_sector) * SECTOR + u64::from(root_bytes);
        if root_sector < 33 || root_bytes == 0 || root_end > bytes {
            return Err(io::Error::other("XDVDFS root lies outside the image"));
        }
        let (executable_sector, executable_bytes) =
            find_executable(&mut file, partition_offset, root_sector, root_bytes, bytes)?;
        return Ok(Disc {
            partition_offset,
            root_sector,
            root_bytes,
            bytes,
            executable_sector,
            executable_bytes,
        });
    }
    Err(io::Error::other(
        "no complete XDVDFS volume header; keep the original image unchanged",
    ))
}

fn find_executable(
    file: &mut File,
    partition: u64,
    root_sector: u32,
    root_bytes: u32,
    image_bytes: u64,
) -> io::Result<(u32, u32)> {
    let root = partition + u64::from(root_sector) * SECTOR;
    let mut pending = vec![0u64];
    let mut visited = BTreeSet::new();
    while let Some(offset) = pending.pop() {
        if offset + 14 > u64::from(root_bytes) || !visited.insert(offset) || visited.len() > 65536 {
            return Err(io::Error::other("invalid or cyclic XDVDFS root directory"));
        }
        file.seek(SeekFrom::Start(root + offset))?;
        let mut entry = [0u8; 14];
        file.read_exact(&mut entry)?;
        let length = usize::from(entry[13]);
        if offset + 14 + length as u64 > u64::from(root_bytes) {
            return Err(io::Error::other("XDVDFS filename outside directory"));
        }
        let mut name = vec![0; length];
        file.read_exact(&mut name)?;
        if name.eq_ignore_ascii_case(b"default.xbe") && entry[12] & 0x10 == 0 {
            let sector = u32::from_le_bytes(entry[4..8].try_into().unwrap());
            let bytes = u32::from_le_bytes(entry[8..12].try_into().unwrap());
            let start = partition + u64::from(sector) * SECTOR;
            if bytes < 0x178 || start + u64::from(bytes) > image_bytes {
                return Err(io::Error::other("incomplete original-Xbox executable"));
            }
            file.seek(SeekFrom::Start(start))?;
            let mut magic = [0; 4];
            file.read_exact(&mut magic)?;
            if &magic != b"XBEH" {
                return Err(io::Error::other(
                    "default.xbe has no Xbox executable header",
                ));
            }
            return Ok((sector, bytes));
        }
        for edge in [&entry[..2], &entry[2..4]] {
            let child = u64::from(u16::from_le_bytes(edge.try_into().unwrap())) * 4;
            if child != 0 {
                pending.push(child);
            }
        }
    }
    Err(io::Error::other(
        "no original-Xbox default.xbe in the game partition",
    ))
}
