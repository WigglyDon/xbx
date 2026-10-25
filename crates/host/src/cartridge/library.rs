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
        if self.entries.len() >= 128 {
            return Err(io::Error::other("collection limit"));
        }
        let media = media.canonicalize()?;
        let d = crate::disc::inspect(&media)?;
        let hash = file_digest(&media)?;
        if let Some(e) = self.entries.iter().find(|e| e.media_sha256 == hash) {
            return Ok(e.id.clone());
        }
        let id = new_id()?;
        self.entries.push(Entry {
            id: id.clone(),
            profile: new_id()?,
            name: d.title.name,
            media: media
                .to_str()
                .ok_or_else(|| io::Error::other("non UTF8 media"))?
                .into(),
            media_sha256: hash,
            partition: d.partition_offset,
        });
        atomic(&app.join("library.json"), &json(self)?)?;
        Ok(id)
    }
    pub fn relink(&mut self, app: &Path, id: &str, media: &Path) -> io::Result<()> {
        let e = self
            .entries
            .iter_mut()
            .find(|e| e.id == id && valid_id(id))
            .ok_or_else(|| io::Error::other("unknown game"))?;
        let media = media.canonicalize()?;
        let d = crate::disc::inspect(&media)?;
        if file_digest(&media)? != e.media_sha256 || d.partition_offset != e.partition {
            return Err(io::Error::other(
                "relink requires the same game image; identity/progress retained",
            ));
        }
        e.media = media
            .to_str()
            .ok_or_else(|| io::Error::other("non UTF8 path"))?
            .into();
        atomic(&app.join("library.json"), &json(self)?)
    }
    pub fn media(e: &Entry) -> io::Result<(File, PathBuf)> {
        use std::io::{Seek, SeekFrom};
        let path = PathBuf::from(&e.media);
        let mut f = File::open(&path).map_err(|e| {
            io::Error::new(
                e.kind(),
                "game image unavailable; relink this collection entry",
            )
        })?;
        let d = crate::disc::inspect_open(&mut f)?;
        f.seek(SeekFrom::Start(0))?;
        if d.partition_offset != e.partition || reader_digest(&mut f)? != e.media_sha256 {
            return Err(io::Error::other(
                "game image changed; original session retained",
            ));
        }
        f.seek(SeekFrom::Start(0))?;
        Ok((f, path))
    }
}
