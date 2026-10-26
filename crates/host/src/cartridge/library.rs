// SPDX-License-Identifier: MIT
use super::*;
use std::path::{Path, PathBuf};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    pub profile: String,
    pub name: String,
    pub media: String,
    pub media_sha256: String,
    pub partition: u64,
}
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Library {
    pub entries: Vec<Entry>,
}
impl Library {
    pub fn read(app: &Path) -> io::Result<Self> {
        match read_json(&app.join("library.json")) {
            Ok(v) => Ok(v),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e),
        }
    }
    pub fn resolve(&self, id: &str) -> io::Result<Entry> {
        if !valid_id(id) {
            return Err(io::Error::other("invalid game identity"));
        }
        self.entries
            .iter()
            .find(|e| e.id == id)
            .cloned()
            .ok_or_else(|| io::Error::other("game not in collection"))
    }
    pub fn add(&mut self, app: &Path, media: &Path) -> io::Result<String> {
        let path = media.canonicalize()?;
        let file = File::open(&path)?;
        self.add_open(
            app,
            path.to_str()
                .ok_or_else(|| io::Error::other("non UTF8 media"))?,
            file,
        )
    }
    /// A scoped document grant is a retained read-only capability. Android
    /// supplies its URI and descriptor; neither import nor launch reopens it as
    /// a filesystem pathname or copies the image into app storage.
    pub fn add_open(&mut self, app: &Path, location: &str, mut file: File) -> io::Result<String> {
        if self.entries.len() >= 128 {
            return Err(io::Error::other("collection limit"));
        }
        if location.is_empty() || location.len() > 16384 || location.chars().any(char::is_control) {
            return Err(io::Error::other("invalid media location"));
        }
        let (d, hash) = inspect_capability(&mut file)?;
        if let Some(e) = self.entries.iter_mut().find(|e| e.media_sha256 == hash) {
            let id = e.id.clone();
            e.media = location.into();
            atomic(&app.join("library.json"), &json(self)?)?;
            return Ok(id);
        }
        let id = new_id()?;
        self.entries.push(Entry {
            id: id.clone(),
            profile: new_id()?,
            name: d.title.name,
            media: location.into(),
            media_sha256: hash,
            partition: d.partition_offset,
        });
        atomic(&app.join("library.json"), &json(self)?)?;
        Ok(id)
    }
    pub fn relink(&mut self, app: &Path, id: &str, media: &Path) -> io::Result<()> {
        let path = media.canonicalize()?;
        self.relink_open(
            app,
            id,
            path.to_str()
                .ok_or_else(|| io::Error::other("non UTF8 path"))?,
            File::open(&path)?,
        )
    }
    pub fn relink_open(
        &mut self,
        app: &Path,
        id: &str,
        location: &str,
        mut file: File,
    ) -> io::Result<()> {
        if location.is_empty() || location.len() > 16384 || location.chars().any(char::is_control) {
            return Err(io::Error::other("invalid media location"));
        }
        let e = self
            .entries
            .iter_mut()
            .find(|e| e.id == id && valid_id(id))
            .ok_or_else(|| io::Error::other("unknown game"))?;
        let (d, hash) = inspect_capability(&mut file)?;
        if hash != e.media_sha256 || d.partition_offset != e.partition {
            return Err(io::Error::other(
                "relink requires the same game image; identity/progress retained",
            ));
        }
        e.media = location.into();
        atomic(&app.join("library.json"), &json(self)?)
    }
    pub fn media(e: &Entry) -> io::Result<(File, PathBuf)> {
        let path = PathBuf::from(&e.media);
        let f = File::open(&path).map_err(|e| {
            io::Error::new(
                e.kind(),
                "game image unavailable; relink this collection entry",
            )
        })?;
        Ok((Self::verify_open(e, f)?, path))
    }
    pub fn verify_open(e: &Entry, mut file: File) -> io::Result<File> {
        let (d, hash) = inspect_capability(&mut file)?;
        if d.partition_offset != e.partition || hash != e.media_sha256 {
            return Err(io::Error::other(
                "game image changed; original session retained",
            ));
        }
        Ok(file)
    }
}

fn inspect_capability(file: &mut File) -> io::Result<(crate::disc::Disc, String)> {
    use std::io::{Seek, SeekFrom};
    use std::os::fd::AsRawFd;
    unsafe extern "C" {
        fn fcntl(fd: i32, cmd: i32, ...) -> i32;
    }
    let flags = unsafe { fcntl(file.as_raw_fd(), 3) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    if flags & 3 != 0 {
        return Err(io::Error::other("game capability must be read-only"));
    }
    let before = file.metadata()?;
    let disc = crate::disc::inspect_open(file)?;
    file.seek(SeekFrom::Start(0))?;
    let hash = reader_digest(&mut *file)?;
    let after = file.metadata()?;
    if before.len() != after.len() || before.modified()? != after.modified()? {
        return Err(io::Error::other(
            "game changed during capability validation",
        ));
    }
    file.seek(SeekFrom::Start(0))?;
    Ok((disc, hash))
}
