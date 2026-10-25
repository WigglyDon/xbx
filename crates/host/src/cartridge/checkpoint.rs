// SPDX-License-Identifier: MIT
//! GCX-style validation/publication, adapted to native Xbox qcow2 machine state.
//! A restore mutates only a private attempt until native completion and a second
//! current-save check. Failed teardown can never flush into authoritative saves.
use super::*;
use std::path::PathBuf;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub schema: u32,
    pub binding: Binding,
    pub generation: u64,
    pub saves: store::SaveSignature,
    pub paused: bool,
    pub tag: String,
    pub disk_bytes: String,
    pub eeprom_bytes: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub checkpoint: String,
}
pub struct Validated {
    pub id: String,
    pub directory: PathBuf,
    pub record: Record,
}
impl Validated {
    pub fn recheck(&self, p: &store::Profile, r: &resources::Resources) -> io::Result<()> {
        p.lease.require_generation(self.record.generation)?;
        if store::signature(&p.current()?, r)? != self.record.saves {
            return Err(io::Error::other(
                "newer ordinary save bytes; checkpoint retained; restart explicitly with current saves",
            ));
        }
        Ok(())
    }
}
pub fn load(
    p: &store::Profile,
    r: &resources::Resources,
    binding: &Binding,
) -> io::Result<Option<Validated>> {
    let session: Session = match read_json(&p.root.join("session.json")) {
        Ok(v) => v,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    if !valid_id(&session.checkpoint) {
        return Err(io::Error::other("invalid checkpoint identity"));
    }
    let directory = p.root.join("checkpoints").join(&session.checkpoint);
    let record: Record = read_json(&directory.join("record.json"))?;
    if record.schema != 1 || record.binding != *binding || record.tag != "xbx-session" {
        return Err(io::Error::other(
            "session incompatible; checkpoint/current saves retained; explicit restart available",
        ));
    }
    if file_digest(&directory.join("hdd.qcow2"))? != record.disk_bytes
        || file_digest(&directory.join("eeprom.bin"))? != record.eeprom_bytes
    {
        return Err(io::Error::other(
            "checkpoint damaged; current saves retained",
        ));
    }
    let v = Validated {
        id: session.checkpoint,
        directory,
        record,
    };
    v.recheck(p, r)?;
    Ok(Some(v))
}
/// Native savevm completion and quiescence are prerequisites, not inferred here.
pub fn publish(
    p: &store::Profile,
    r: &resources::Resources,
    binding: &Binding,
    paused: bool,
    tag: &str,
) -> io::Result<String> {
    let current = p.current()?;
    store::sync_files(&current)?;
    let generation = p.lease.generation()?;
    let saves = store::signature(&current, r)?;
    let id = new_id()?;
    let dir = p.root.join("checkpoints").join(&id);
    mkdir(&dir)?;
    store::copy_machine(&current, &dir)?;
    let record = Record {
        schema: 1,
        binding: binding.clone(),
        generation,
        saves,
        paused,
        tag: tag.into(),
        disk_bytes: file_digest(&dir.join("hdd.qcow2"))?,
        eeprom_bytes: file_digest(&dir.join("eeprom.bin"))?,
    };
    write_new(&dir.join("record.json"), &json(&record)?)?;
    File::open(&dir)?.sync_all()?;
    File::open(dir.parent().unwrap())?.sync_all()?;
    // Byte readback plus logical-generation validation precedes publication.
    if file_digest(&dir.join("hdd.qcow2"))? != record.disk_bytes
        || file_digest(&dir.join("eeprom.bin"))? != record.eeprom_bytes
    {
        return Err(io::Error::other("checkpoint readback failed"));
    }
    let v = Validated {
        id: id.clone(),
        directory: dir,
        record,
    };
    v.recheck(p, r)?;
    let old = read_json::<Session>(&p.root.join("session.json"));
    atomic(
        &p.root.join("session.json"),
        &json(&Session {
            checkpoint: id.clone(),
        })?,
    )?;
    p.retire_previous()?;
    // Only retire an earlier accepted successful generation, after the new
    // reference is durable. Rejected/damaged/interrupted candidates are retained.
    if let Ok(old) = old
        && valid_id(&old.checkpoint)
        && old.checkpoint != id
    {
        let previous = p.root.join("checkpoints").join(old.checkpoint);
        if let Ok(prior) = read_json::<Record>(&previous.join("record.json"))
            && prior.schema == 1
            && prior.binding == *binding
            && file_digest(&previous.join("hdd.qcow2"))? == prior.disk_bytes
            && file_digest(&previous.join("eeprom.bin"))? == prior.eeprom_bytes
        {
            fs::remove_dir_all(previous)?;
            File::open(p.root.join("checkpoints"))?.sync_all()?;
        }
    }
    Ok(id)
}
/// Exact pointer preservation is useful for explicit restart and interrupted
/// publication recovery. It never rolls ordinary storage back.
pub fn restart(p: &store::Profile) -> io::Result<()> {
    let path = p.root.join("session.json");
    if path.exists() {
        let retained = p.root.join(format!("rejected-session-{}.json", new_id()?));
        fs::rename(path, retained)?;
        File::open(&p.root)?.sync_all()?;
    }
    atomic(
        &p.root.join("restart.intent.json"),
        b"{\"explicit_restart\":true}\n",
    )
}
pub fn restore_copy(v: &Validated, p: &store::Profile) -> io::Result<PathBuf> {
    let dir = p.attempt()?;
    store::copy_machine(&v.directory, &dir)?;
    Ok(dir)
}
