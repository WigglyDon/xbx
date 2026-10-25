// SPDX-License-Identifier: MIT
//! Shared product session policy. Desktop and Android transports serialize native
//! work through this owner; they do not own competing save/resume decisions.
use super::*;
use serde_json::json;
use std::path::{Path, PathBuf};
pub trait Engine: Sized {
    fn start(
        r: &resources::Resources,
        p: &store::Profile,
        dir: &Path,
        media: &File,
        partition: u64,
        app_lease: &crate::storage::ProfileLease,
    ) -> io::Result<Self>;
    fn pause(&mut self) -> io::Result<()>;
    fn resume(&mut self) -> io::Result<()>;
    fn capture(&mut self) -> io::Result<()>;
    fn load(&mut self) -> io::Result<()>;
    fn retire(&mut self) -> io::Result<bool>;
}
pub struct Live<E: Engine> {
    pub native: E,
    pub profile: store::Profile,
    pub binding: Binding,
    pub paused: bool,
    pub restored: bool,
    running: bool,
    committed: bool,
}
pub struct Player<E: Engine> {
    pub resources: resources::Resources,
    pub app: PathBuf,
    pub live: Option<Live<E>>,
    _lease: crate::storage::ProfileLease,
}
impl<E: Engine> Player<E> {
    pub fn open(app: &Path, resources: &Path) -> io::Result<Self> {
        if !app.exists() {
            mkdir(app)?
        }
        let app = app.canonicalize()?;
        let lease = crate::storage::ProfileLease::acquire(&app)?;
        Ok(Self {
            resources: resources::Resources::open(resources)?,
            app,
            live: None,
            _lease: lease,
        })
    }
    pub fn add(&mut self, media: &Path) -> io::Result<String> {
        let mut library = library::Library::read(&self.app)?;
        library.add(&self.app, media)
    }
    pub fn launch(&mut self, id: &str) -> io::Result<bool> {
        let entry = library::Library::read(&self.app)?.resolve(id)?;
        // Resolve media before disturbing another game's only live state.
        let (media, _) = library::Library::media(&entry)?;
        if self.live.as_ref().is_some_and(|l| l.binding.game == id) {
            return Ok(self.live.as_ref().unwrap().restored);
        }
        self.depart()?;
        let p = store::Profile::open(&self.app, &entry, &self.resources)?;
        let binding = Binding {
            game: entry.id,
            profile: entry.profile,
            media: entry.media_sha256,
            partition: entry.partition,
            runtime: self.resources.identity.clone(),
            state_abi: 1,
        };
        let checkpoint = checkpoint::load(&p, &self.resources, &binding)?;
        if checkpoint.is_none() && p.lease.generation()? > 0 {
            if !p.root.join("restart.intent.json").is_file() {
                return Err(io::Error::other(
                    "interrupted session has no healthy checkpoint; retained data requires explicit restart using current saves",
                ));
            }
            if !p.current()?.join("eeprom.bin").is_file() {
                return Err(io::Error::other(
                    "existing console identity is missing; preserve storage and recover its EEPROM",
                ));
            }
            fs::rename(
                p.root.join("restart.intent.json"),
                p.root.join(format!("restart-used-{}.json", new_id()?)),
            )?;
            File::open(&p.root)?.sync_all()?;
        }
        let dir = match &checkpoint {
            Some(v) => checkpoint::restore_copy(v, &p)?,
            None => p.current()?,
        };
        if checkpoint.is_none() {
            p.lease.begin_writes()?;
        }
        let mut native = E::start(
            &self.resources,
            &p,
            &dir,
            &media,
            entry.partition,
            &self._lease,
        )?;
        let paused = checkpoint.as_ref().is_some_and(|v| v.record.paused);
        if let Some(v) = &checkpoint {
            // Immediately before native mutation, and again before this attempted
            // machine/storage can become authoritative. Never attach current saves
            // to a load attempt or let partial failure publish through teardown.
            v.recheck(&p, &self.resources)?;
            native.load()?;
            v.recheck(&p, &self.resources)?;
            p.select(&dir)?;
        }
        self.live = Some(Live {
            native,
            profile: p,
            binding,
            paused,
            restored: checkpoint.is_some(),
            running: false,
            committed: false,
        });
        if !paused {
            self.resume()?
        }
        Ok(checkpoint.is_some())
    }
    pub fn resume(&mut self) -> io::Result<()> {
        let l = self
            .live
            .as_mut()
            .ok_or_else(|| io::Error::other("no game"))?;
        if !l.running {
            l.profile.lease.begin_writes()?;
            l.native.resume()?;
            l.running = true;
            l.committed = false;
        }
        l.paused = false;
        Ok(())
    }
    pub fn pause(&mut self, user: bool) -> io::Result<()> {
        if let Some(l) = self.live.as_mut() {
            l.native.pause()?;
            l.running = false;
            if user {
                l.paused = true;
                l.committed = false
            }
        }
        if user && self.live.is_some() {
            self.commit()?;
        }
        Ok(())
    }
    pub fn commit(&mut self) -> io::Result<String> {
        let l = self
            .live
            .as_mut()
            .ok_or_else(|| io::Error::other("no game"))?;
        l.native.capture()?;
        l.running = false;
        let id = checkpoint::publish(
            &l.profile,
            &self.resources,
            &l.binding,
            l.paused,
            "xbx-session",
        )?;
        l.committed = true;
        Ok(id)
    }
    pub fn depart(&mut self) -> io::Result<()> {
        if self.live.is_some() {
            self.pause(false)?;
            if !self.live.as_ref().unwrap().committed {
                self.commit()?;
            }
            let mut l = self.live.take().unwrap();
            let clean = l.native.retire()?;
            write_new(
                &l.profile.root.join(format!("return-{}.json", new_id()?)),
                &json(&json!({"native_reaped":true,"storage_synced":true,"clean":clean}))?,
            )?;
            if !clean {
                return Err(io::Error::other(
                    "native shutdown not clean; checkpoint/profile retained",
                ));
            }
        }
        Ok(())
    }
    pub fn restart(&mut self, id: &str) -> io::Result<()> {
        if self.live.is_some() {
            return Err(io::Error::other(
                "leave the current game before explicit restart",
            ));
        }
        let e = library::Library::read(&self.app)?.resolve(id)?;
        let p = store::Profile::open(&self.app, &e, &self.resources)?;
        checkpoint::restart(&p)
    }
}
