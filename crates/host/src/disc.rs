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
    pub title: Title,
}

/// Certificate metadata is untrusted display information, not a permanent GameId.
#[derive(Debug, PartialEq, Eq)]
pub struct Title {
    pub id: u32,
    pub name: String,
    pub region: u32,
    pub version: u32,
}

pub fn inspect(path: &Path) -> io::Result<Disc> {
    let mut file = File::open(path)?;
    inspect_open(&mut file)
}

/// Duplicate a validated read-only capability for QEMU's -add-fd parser.
/// QEMU requires CLOEXEC clear on this incoming FD and sets it on its own copy.
/// The original grant keeps its flags and remains owned by LiveRuntime.
pub fn qemu_media_fd(file: &File) -> io::Result<std::os::fd::OwnedFd> {
    use std::os::fd::AsRawFd;
    unsafe extern "C" {
        fn fcntl(fd: i32, command: i32, ...) -> i32;
    }
    let access = unsafe { fcntl(file.as_raw_fd(), 3) }; // F_GETFL
    if access < 0 {
        return Err(io::Error::last_os_error());
    }
    if access & 3 != 0 {
        return Err(io::Error::other(
            "QEMU disc handoff requires a read-only file",
        ));
    }
    let copy: std::os::fd::OwnedFd = file.try_clone()?.into();
    let flags = unsafe { fcntl(copy.as_raw_fd(), 1) }; // F_GETFD
    if flags < 0 || unsafe { fcntl(copy.as_raw_fd(), 2, flags & !1) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(copy)
}

/// Inspect the retained grant object, never reopen a provider display path.
pub fn inspect_open(file: &mut File) -> io::Result<Disc> {
    if !file.metadata()?.is_file() {
        return Err(io::Error::other("disc must be a regular seekable file"));
    }
    let bytes = file.seek(SeekFrom::End(0))?;
    if bytes == 0 || bytes != file.metadata()?.len() {
        return Err(io::Error::other(
            "provider does not expose a stable seekable image size",
        ));
    }
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
            find_executable(file, partition_offset, root_sector, root_bytes, bytes)?;
        let title = read_title(
            file,
            partition_offset + u64::from(executable_sector) * SECTOR,
            executable_bytes,
        )?;
        return Ok(Disc {
            partition_offset,
            root_sector,
            root_bytes,
            bytes,
            executable_sector,
            executable_bytes,
            title,
        });
    }
    Err(io::Error::other(
        "no complete XDVDFS volume header; keep the original image unchanged",
    ))
}

fn word(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_title(file: &mut File, start: u64, executable_bytes: u32) -> io::Result<Title> {
    // Read only metadata before the certificate's LAN/signature key fields.
    let mut header = [0; 0x178];
    file.seek(SeekFrom::Start(start))?;
    file.read_exact(&mut header)?;
    let headers = u64::from(word(&header, 0x108));
    let certificate = word(&header, 0x118)
        .checked_sub(word(&header, 0x104))
        .ok_or_else(|| io::Error::other("XBE certificate precedes image base"))?;
    if headers > u64::from(executable_bytes)
        || certificate < 0x178
        || u64::from(certificate) + 0xb0 > headers
    {
        return Err(io::Error::other(
            "XBE certificate outside executable headers",
        ));
    }
    let mut metadata = [0; 0xb0];
    file.seek(SeekFrom::Start(start + u64::from(certificate)))?;
    file.read_exact(&mut metadata)?;
    let size = u64::from(word(&metadata, 0));
    if size < 0xb0 || u64::from(certificate) + size > headers {
        return Err(io::Error::other("invalid XBE certificate size"));
    }
    let utf16: Vec<_> = metadata[0x0c..0x5c]
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .take_while(|value| *value != 0)
        .collect();
    let name =
        String::from_utf16(&utf16).map_err(|_| io::Error::other("invalid XBE title UTF-16"))?;
    if name.chars().any(char::is_control) {
        return Err(io::Error::other("control character in XBE title"));
    }
    Ok(Title {
        id: word(&metadata, 8),
        name,
        region: word(&metadata, 0xa0),
        version: word(&metadata, 0xac),
    })
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
