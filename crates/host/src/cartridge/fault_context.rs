// SPDX-License-Identifier: MIT
//! Explicit owner-authorized, private development observation. This is not an
//! ordinary launch option, healthy checkpoint or commercial acceptance test.
//! Reuses the product's media/storage/session owners and retained native GDB stub.
use super::{self as c, resources};
use serde_json::json;
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadPlan {
    stop_pc: u32,
    commands: Vec<String>,
    #[serde(default = "default_deadline")]
    deadline_seconds: u64,
    #[serde(default)]
    stops: Vec<u32>,
    #[serde(default)]
    follow_word: Option<String>,
}
fn default_deadline() -> u64 {
    90
}
pub(super) fn hex_words(text: &str) -> Vec<u32> {
    text.split_whitespace()
        .filter_map(|s| {
            let word = s.trim_end_matches([':', ',']);
            if word == "0" {
                Some(0) // HMP p /x uses C alternate formatting: zero has no 0x prefix.
            } else {
                u32::from_str_radix(word.strip_prefix("0x")?, 16).ok()
            }
        })
        .collect()
}
fn memory_words(text: &str) -> Vec<u32> {
    text.lines()
        .filter_map(|line| line.split_once(':'))
        .flat_map(|(_, data)| hex_words(data))
        .collect()
}
fn hmp(native: &mut c::desktop::Native, command: &str) -> Result<String, Error> {
    native
        .call("human-monitor-command", json!({"command-line":command}))?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "invalid HMP response".into())
}

struct Debugger(std::process::Child);
impl std::ops::Deref for Debugger {
    type Target = std::process::Child;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for Debugger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Drop for Debugger {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

pub(super) fn observe(repo: &Path, root: &Path, original: bool, rsp: bool) -> Result<(), Error> {
    let dir = root.join("fault-context-v2");
    if !dir.exists() {
        c::mkdir(&dir)?;
    }
    let name = if original {
        if rsp { "original-rsp-v1" } else { "original" }.to_owned()
    } else {
        match std::env::var("XBX_PRIVATE_OBSERVATION_NAME") {
            Ok(name)
                if !name.is_empty()
                    && name.len() <= 48
                    && name != "original"
                    && name
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') =>
            {
                name
            }
            Ok(_) => return Err("invalid private observation name".into()),
            Err(std::env::VarError::NotPresent) => "fable".to_owned(),
            Err(error) => return Err(error.into()),
        }
    };
    let result_path = dir.join(format!("{name}.result.json"));
    let log_path = dir.join(format!("{name}.{}.log", if rsp { "rsp" } else { "gdb" }));
    if result_path.exists() || log_path.exists() {
        return Err("private observation already attempted; preserve and reconcile it".into());
    }
    let qualified: serde_json::Value =
        c::read_json(&root.join("x87-boundary-v1/relaunch-product-v2.result.json"))?;
    if qualified["native_clean"] != true || qualified["core"] != resources::X87_RESTART_CORE {
        return Err("exact core relaunch qualification required".into());
    }
    if !original {
        let control: serde_json::Value = c::read_json(&dir.join(if rsp {
            "original-rsp-v1.result.json"
        } else {
            "original.result.json"
        }))?;
        if control["debugger_completed"] != true
            || control["native_clean"] != true
            || control["memory_reads_completed"] != true
            || control["stop_pc_matched"] != true
        {
            return Err("actual original debugger transport/control required".into());
        }
    }
    let commands = std::path::PathBuf::from(std::env::var("XBX_PRIVATE_GDB_COMMANDS")?);
    if !fs::symlink_metadata(&commands)?.is_file() || fs::metadata(&commands)?.len() > 16384 {
        return Err("invalid bounded private debugger commands".into());
    }
    let fragment = fs::read_to_string(&commands)?;
    let plan_path = std::path::PathBuf::from(std::env::var("XBX_PRIVATE_HMP_PLAN")?);
    let plan: ReadPlan = c::read_json(&plan_path)?;
    if !(30..=300).contains(&plan.deadline_seconds)
        || !(0x10000..0x80000000).contains(&plan.stop_pc)
        || plan.commands.is_empty()
        || plan.commands.len() > 8
        || plan.commands.iter().any(|s| {
            s.len() > 128
                || s.contains([';', '\n', '\r'])
                || !(s.starts_with("x /1wx ") || s.starts_with("x /4wx "))
        })
    {
        return Err("invalid bounded private word-read plan".into());
    }

    if rsp
        && (plan.stops.is_empty()
            || plan.stops.len() > 2
            || plan.stops.last() != Some(&plan.stop_pc)
            || plan
                .stops
                .iter()
                .any(|p| !(0x10000..0x80000000).contains(p)))
        || !rsp && (!plan.stops.is_empty() || plan.follow_word.is_some())
        || plan.follow_word.as_ref().is_some_and(|s| {
            s.len() > 128 || s.contains([';', '\n', '\r']) || !s.starts_with("x /1wx ")
        })
    {
        return Err("invalid bounded stop/follow plan".into());
    }
    let r = resources::Resources::x87_logging(&root.join("package-x87-npx-v1"))?;
    let app = root.join(format!("fault-{name}-{}", c::new_id()?));
    let mut player = c::desktop::Player::with_resources(&app, r)?;
    let media = if original {
        root.join("npx-owner-port-v1/layout-v1/original.iso")
    } else {
        std::path::PathBuf::from(std::env::var("XBX_REFERENCE_MEDIA")?)
    };
    if original
        && c::file_digest(&media)?
            != "372b08bfa9b9022b74985a088ecae10eeea7e25cf61823db6b0242f23b19123e"
    {
        return Err("original control changed".into());
    }
    let game = player.add(&media)?;
    let entry = c::library::Library::read(&app)?.resolve(&game)?;
    if !original && entry.media_sha256 != std::env::var("XBX_REFERENCE_MEDIA_SHA256")? {
        return Err("matched game media changed; no guest start".into());
    }
    let disk = repo.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2");
    let eeprom = repo.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin");
    let seeds = (c::file_digest(&disk)?, c::file_digest(&eeprom)?);
    if seeds.0 != "00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd" {
        return Err("matched seed changed".into());
    }
    {
        let _p = c::store::Profile::open_seeded(&app, &entry, &player.resources, &disk, &eeprom)?;
    }
    // Dirty/write epoch is durable before native entry. GDB controls this one
    // developer execution; no healthy snapshot is captured from a debug stop.
    player.launch_quiesced(&game)?;
    let run = player.live.as_ref().ok_or("live")?.native.run.clone();
    let socket = std::path::PathBuf::from(format!("/tmp/xbx-gdb-{}.sock", c::new_id()?));
    let script = dir.join(format!("{name}.gdb"));
    c::write_new(&script, format!("set pagination off\nset confirm off\nset auto-load off\nset debuginfod enabled off\nset disassemble-next-line off\nset remotetimeout 10\nset architecture i386\ntarget remote | /usr/bin/socat - UNIX-CONNECT:{}\n{}\nprintf \"XBX-DEBUG-CAPTURE-COMPLETE\\n\"\ndisconnect\nquit\n", socket.display(),fragment).as_bytes())?;
    let started = Instant::now();
    let mut memory_reads = Vec::new();
    let mut memory_reads_completed = false;
    let mut stop_pc_matched = false;
    let observed = (|| -> Result<bool, Error> {
        let native = &mut player.live.as_mut().ok_or("live")?.native;
        let response = native.call("human-monitor-command", json!({"command-line":format!("gdbserver unix:{},server=on,wait=off",socket.display())}))?;
        c::write_new(
            &dir.join(format!("{name}.server.json")),
            &c::json(&response)?,
        )?;
        let log = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&log_path)?;
        let mut serial = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(app.join("kernel.log"))?;
        if rsp {
            super::debug_stop::run(
                native,
                &socket,
                &plan.stops,
                plan.deadline_seconds,
                log,
                &mut serial,
            )?;
        } else {
            let mut debugger = Debugger(
                Command::new("/usr/bin/gdb")
                    .args(["--batch", "--nx", "-x"])
                    .arg(&script)
                    .stdin(Stdio::null())
                    .stdout(log.try_clone()?)
                    .stderr(log)
                    .spawn()?,
            );
            let wait = (|| -> Result<bool, Error> {
                loop {
                    let text = native.serial()?;
                    serial.write_all(text.as_bytes())?;
                    if serial.metadata()?.len() > 1024 * 1024 {
                        return Err("ordinary log bound".into());
                    }
                    if let Some(status) = debugger.try_wait()? {
                        return Ok(status.success()
                            && fs::read_to_string(&log_path)?
                                .contains("XBX-DEBUG-CAPTURE-COMPLETE"));
                    }
                    if started.elapsed() >= Duration::from_secs(plan.deadline_seconds) {
                        return Err("finite debugger deadline".into());
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            })();
            // Always reap the debugger, including its pipe transport, before return.
            if debugger.try_wait()?.is_none() {
                debugger.kill()?;
            }
            debugger.wait()?;
            serial.sync_all()?;
            if !wait? {
                return Ok(false);
            }
        }
        // GDB disconnect closes its pipe without detaching/resuming the target.
        // Reconfirm the paused instruction through the independent native owner
        // before reading any words; a moved PC refuses this observation.
        native.pause()?;
        let pc = hmp(native, "p /x $eip")?;
        stop_pc_matched = hex_words(&pc).last() == Some(&plan.stop_pc);
        memory_reads.push(json!({"command":"p /x $eip","response":pc}));
        if !stop_pc_matched {
            return Err("debug stop moved before memory observation".into());
        }
        for command in &plan.commands {
            let response = hmp(native, command)?;
            let valid =
                memory_words(&response).len() == if command.starts_with("x /4wx ") { 4 } else { 1 };
            memory_reads.push(json!({"command":command,"response":response}));
            if !valid {
                return Err("bounded native word read did not produce data".into());
            }
        }
        if rsp {
            let status = hmp(native, "p /x $eax")?;
            memory_reads.push(json!({"command":"p /x $eax","response":status}));
            if hex_words(&status).len() != 1 {
                return Err("public call return register unavailable".into());
            }
        }
        if let Some(command) = &plan.follow_word {
            let response = hmp(native, command)?;
            let words = memory_words(&response);
            memory_reads.push(json!({"command":command,"response":response}));
            if words.len() != 1 || !(0x10000..=0x7fffffec).contains(&words[0]) {
                return Err("follow pointer must name a bounded user buffer".into());
            }
            let command = format!("x /4wx 0x{:x}", words[0]);
            let response = hmp(native, &command)?;
            let words = memory_words(&response);
            memory_reads.push(json!({"command":command,"response":response}));
            if words.len() != 4 || original && words != [0x48454258, 0, 0, 0] {
                return Err("bounded buffer read/original header control mismatch".into());
            }
        }
        if original {
            let mut file = std::fs::File::open(&media)?;
            let disc = crate::disc::inspect_open(&mut file)?;
            let (code, _) = crate::callsite::observe(&mut file, &disc, "xbx-callsite-v1 11000 4")?;
            let expected = u32::from_le_bytes(code.try_into().map_err(|_| "original word extent")?);
            let actual = memory_reads
                .get(1)
                .and_then(|v| v["response"].as_str())
                .and_then(|v| memory_words(v).last().copied());
            if actual != Some(expected) {
                return Err("native memory-read original control mismatch".into());
            }
        }
        memory_reads_completed = true;
        Ok(true)
    })();
    c::write_new(
        &dir.join(format!("{name}.memory-reads.json")),
        &c::json(&memory_reads)?,
    )?;
    let returned = player.stop();
    let native: serde_json::Value = c::read_json(&run.join("native-return.json"))?;
    let unchanged = seeds == (c::file_digest(&disk)?, c::file_digest(&eeprom)?);
    let completed = observed.as_ref().is_ok_and(|v| *v);
    let clean = returned.is_ok() && native["clean"] == true && unchanged;
    c::write_new(
        &result_path,
        &c::json(
            &json!({"original":original,"observation":name,"core":resources::X87_RESTART_CORE,"firmware":resources::NPX_LOGGING_OPEN,"app":app,"native_run":run,"debugger_completed":completed,"memory_reads_completed":memory_reads_completed,"stop_pc_matched":stop_pc_matched,"observation_error":observed.err().map(|e|e.to_string()),"native_clean":clean,"native_return":native,"return_error":returned.err().map(|e|e.to_string()),"seeds_unchanged":unchanged,"elapsed_ms":started.elapsed().as_millis(),"debugger_deadline_seconds":plan.deadline_seconds,"transport":if rsp { "bounded-rsp-stops-hmp-reads" } else { "gdb-hmp-reads" },"healthy_checkpoint_published":false,"commercial_acceptance":false,"private_commands_sha256":c::file_digest(&commands)?,"private_read_plan_sha256":c::file_digest(&plan_path)?}),
        )?,
    )?;
    let _ = fs::remove_file(socket);
    if !completed || !clean {
        return Err("private observation/return incomplete; preserve actual evidence".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_hmp_zero_and_memory_columns() {
        assert_eq!(hex_words("0\r\n"), vec![0]);
        assert_eq!(hex_words("0x4000\r\n"), vec![0x4000]);
        assert_eq!(
            memory_words("00011000: 0x00000000 0x00004000\r\n"),
            vec![0, 0x4000]
        );
        assert!(hex_words("invalid expression").is_empty());
    }
}
