// SPDX-License-Identifier: MIT
//! Bounded integration/packaging for XBX-CARTRIDGE-1, retained inputs only.
use super::{self as c, resources};
use serde_json::{Value, json};
use std::io::Write;
use std::{fs, path::Path, process::Command};
use std::{
    io::{BufRead, BufReader},
    process::{Child, ChildStdin, ChildStdout, Stdio},
    time::{Duration, Instant},
};
struct BuildPaths {
    root: std::path::PathBuf,
}
type Error = Box<dyn std::error::Error>;
fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
fn fixture(repo: &Path, w: &BuildPaths) -> Result<(), Error> {
    let dir = w.root.join("fixtures-v2");
    c::mkdir(&dir)?;
    let llvm = std::path::PathBuf::from(std::env::var("XBX_NDK")?)
        .join("toolchains/llvm/prebuilt/linux-x86_64/bin");
    for (label, tag) in [("a", 65), ("b", 66)] {
        let obj = dir.join(format!("{label}.o"));
        let bin = dir.join(format!("{label}.bin"));
        c::store::checked(
            Command::new(llvm.join("clang"))
                .args(["--target=i386-none-elf", "-march=pentium3", "-c"])
                .arg(format!("-DCARTRIDGE_TAG={tag}"))
                .arg(repo.join("native/fixtures/cartridge.S"))
                .arg("-o")
                .arg(&obj),
        )?;
        c::store::checked(
            Command::new(llvm.join("ld.lld"))
                .args([
                    "-m",
                    "elf_i386",
                    "-Ttext=0x11000",
                    "--oformat=binary",
                    "--entry=_start",
                ])
                .arg(&obj)
                .arg("-o")
                .arg(&bin),
        )?;
        let code = fs::read(bin)?;
        if code.is_empty() || code.len() > 4096 {
            return Err("fixture extent".into());
        }
        let mut xbe = vec![0; 4096];
        xbe[..4].copy_from_slice(b"XBEH");
        for (at, v) in [
            (0x104, 0x10000),
            (0x108, 0x1000),
            (0x10c, 0x2000),
            (0x110, 0x178),
            (0x118, 0x10178),
            (0x11c, 1),
            (0x120, 0x10348),
            (0x128, 0x11000 ^ 0xa8fc57ab),
            (0x130, 0x10000),
            (0x134, 0x100000),
            (0x138, 0x1000),
            (0x158, 0x10300 ^ 0x5b6d40b6),
            (0x178, 0x1d0),
            (0x180, 0x58424300 + tag as u32),
            (0x214, 0xffffff),
            (0x218, 0x7fffffff),
        ] {
            put(&mut xbe, at, v)
        }
        let name = format!("xbx cartridge {label}");
        for (i, ch) in name.encode_utf16().enumerate() {
            xbe[0x184 + i * 2..0x186 + i * 2].copy_from_slice(&ch.to_le_bytes())
        }
        xbe[0x400..0x406].copy_from_slice(b".text\0");
        for (at, v) in [
            (0, 7),
            (4, 0x11000),
            (8, 4096),
            (12, 4096),
            (16, code.len() as u32),
            (20, 0x10400),
            (28, 0x10410),
            (32, 0x10412),
        ] {
            put(&mut xbe, 0x348 + at, v)
        }
        xbe[0x348 + 36..0x348 + 56].copy_from_slice(&section_digest(&code)?);
        xbe.extend_from_slice(&code);
        c::write_new(&dir.join(format!("{label}.xbe")), &xbe)?;
        let mut iso = vec![0; 64 * 2048];
        let at = 32 * 2048;
        let magic = b"MICROSOFT*XBOX*MEDIA";
        iso[at..at + 20].copy_from_slice(magic);
        put(&mut iso, at + 20, 33);
        put(&mut iso, at + 24, 2048);
        iso[at + 2028..at + 2048].copy_from_slice(magic);
        let root = 33 * 2048;
        put(&mut iso, root + 4, 34);
        put(&mut iso, root + 8, xbe.len() as u32);
        iso[root + 12] = 0x20;
        iso[root + 13] = 11;
        iso[root + 14..root + 25].copy_from_slice(b"default.xbe");
        iso[34 * 2048..34 * 2048 + xbe.len()].copy_from_slice(&xbe);
        c::write_new(&dir.join(format!("{label}.iso")), &iso)?;
    }
    Ok(())
}
fn package(repo: &Path, w: &BuildPaths) -> Result<(), Error> {
    let project = w.root.parent().ok_or("workspace parent")?;
    let dir = w.root.join("package-v3");
    c::mkdir(&dir)?;
    c::mkdir(&dir.join("native"))?;
    fs::create_dir_all(dir.join("native/subprojects/slirp"))?;
    let core = w.root.join("native-v3/core");
    let open = project.join("open-startup-v1/build-release-v3/bios.bin");
    if c::file_digest(&core)? != resources::CORE || c::file_digest(&open)? != resources::OPEN {
        return Err("accepted input identity differs".into());
    }
    fs::copy(core, dir.join("native/core"))?;
    fs::copy(open, dir.join("open.bin"))?;
    fs::copy("/usr/bin/qemu-img", dir.join("native/qemu-img"))?;
    fs::copy(
        project.join("ramht-v1/build-v2-retry2/subprojects/slirp/libslirp.so.0"),
        dir.join("native/subprojects/slirp/libslirp.so.0"),
    )?;
    let manifest = resources::Manifest {
        schema: 1,
        core: resources::CORE.into(),
        open_system: resources::OPEN.into(),
        disk_tool: c::file_digest(&dir.join("native/qemu-img"))?,
        slirp: c::file_digest(&dir.join("native/subprojects/slirp/libslirp.so.0"))?,
        state_abi: 1,
    };
    c::write_new(&dir.join("runtime.json"), &c::json(&manifest)?)?;
    fs::copy(repo.join("THIRD_PARTY.md"), dir.join("THIRD_PARTY.md"))?;
    resources::Resources::open(&dir)?;
    Ok(())
}
pub fn command(repo: &Path, root: &Path, action: &str) -> Result<(), Error> {
    let w = BuildPaths { root: root.into() };
    match action {
        "cartridge-package" => package(repo, &w)?,
        "cartridge-fixture" => fixture(repo, &w)?,
        "cartridge-proof" => proof(&w)?,
        "cartridge-native" => native(repo, &w)?,
        "cartridge-partial-load" => partial(&w)?,
        "cartridge-policy-proof" => policy_proof(&w)?,
        "cartridge-paused-contract" => paused_contract(repo, &w)?,
        "cartridge-native-paused" => native_paused(repo, &w)?,
        _ => return Err("unknown cartridge build action".into()),
    };
    Ok(())
}
fn section_digest(bytes: &[u8]) -> Result<[u8; 20], Error> {
    use std::process::Stdio;
    let mut child = Command::new("sha1sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut input = child.stdin.take().ok_or("digest stdin")?;
    input.write_all(&(bytes.len() as u32).to_le_bytes())?;
    input.write_all(bytes)?;
    drop(input);
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err("digest failed".into());
    }
    let s = String::from_utf8(out.stdout)?;
    let mut result = [0; 20];
    for (i, b) in result.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)?
    }
    Ok(result)
}

struct Transport {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}
impl Transport {
    fn spawn(app: &Path, package: &Path) -> Result<Self, Error> {
        let binary = std::env::current_exe()?.with_file_name("xbx-player");
        let mut child = Command::new(binary)
            .arg(app)
            .arg(package)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        Ok(Self {
            input: child.stdin.take().ok_or("stdin")?,
            output: BufReader::new(child.stdout.take().ok_or("stdout")?),
            child,
        })
    }
    fn send(&mut self, v: Value) -> Result<Value, Error> {
        let started = Instant::now();
        writeln!(self.input, "{v}")?;
        self.input.flush()?;
        let mut line = String::new();
        self.output.read_line(&mut line)?;
        let response: Value = serde_json::from_str(&line)?;
        println!(
            "{}",
            json!({"request":v,"response":response,"elapsed_ms":started.elapsed().as_secs_f64()*1000.0,"host_pid":self.child.id(),"memory":memory(self.child.id())})
        );
        if response.get("error").is_some() {
            return Err(format!("operation refused: {response}").into());
        }
        Ok(response["ok"].clone())
    }
    fn refusal(&mut self, v: Value) -> Result<String, Error> {
        writeln!(self.input, "{v}")?;
        self.input.flush()?;
        let mut line = String::new();
        self.output.read_line(&mut line)?;
        let response: Value = serde_json::from_str(&line)?;
        println!(
            "{}",
            json!({"request":v,"expected_refusal":response,"host_pid":self.child.id(),"memory":memory(self.child.id())})
        );
        response["error"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "expected operation was not refused".into())
    }
    fn wait(&mut self, expected: &str) -> Result<String, Error> {
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut output = String::new();
        while Instant::now() < deadline {
            let v = self.send(json!({"op":"serial"}))?;
            output.push_str(v["text"].as_str().unwrap_or(""));
            if output.contains(expected) {
                return Ok(output);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Err(format!("guest completion missing: {expected}; got {output}").into())
    }
    fn quit(mut self) -> Result<(), Error> {
        self.send(json!({"op":"quit"}))?;
        if !self.child.wait()?.success() {
            return Err("player did not retire".into());
        }
        Ok(())
    }
}
impl Drop for Transport {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn proof(w: &BuildPaths) -> Result<(), Error> {
    let app = w.root.join(format!("proof-{}", c::new_id()?));
    let package = w.root.join("package-v3");
    println!(
        "{}",
        json!({"fresh_app":app,"packaged_runtime":package,"original_program":true,"phone":false})
    );
    let mut first = Transport::spawn(&app, &package)?;
    let a = first.send(json!({"op":"add","media":w.root.join("fixtures-v2/a.iso")}))?["game"]
        .as_str()
        .ok_or("game")?
        .to_owned();
    let b = first.send(json!({"op":"add","media":w.root.join("fixtures-v2/b.iso")}))?["game"]
        .as_str()
        .ok_or("game")?
        .to_owned();
    first.send(json!({"op":"open","game":a}))?;
    first.wait("CART A state=00000000 save=00000000")?;
    first.send(json!({"op":"fixture-input","text":"as a"}))?;
    first.wait("CART A state=00000002 save=00000001 cookie=57F090E4")?;
    first.send(json!({"op":"commit"}))?;
    first.quit()?;
    let mut second = Transport::spawn(&app, &package)?;
    if second.send(json!({"op":"open","game":a}))?["restored"] != true {
        return Err("fresh process did not restore".into());
    }
    second.send(json!({"op":"fixture-input","text":"?"}))?;
    second.wait("CART A state=00000002 save=00000001 cookie=57F090E4")?;
    second.send(json!({"op":"fixture-input","text":"a"}))?;
    second.wait("CART A state=00000003 save=00000001 cookie=6B482CC3")?;
    second.send(json!({"op":"commit"}))?;
    second.quit()?;
    let mut third = Transport::spawn(&app, &package)?;
    third.send(json!({"op":"open","game":a}))?;
    third.send(json!({"op":"fixture-input","text":"?"}))?;
    third.wait("CART A state=00000003 save=00000001 cookie=6B482CC3")?;
    third.send(json!({"op":"open","game":b}))?;
    third.wait("CART B state=00000000 save=00000000")?;
    third.send(json!({"op":"fixture-input","text":"a"}))?;
    third.wait("CART B state=00000001 save=00000000")?;
    third.send(json!({"op":"open","game":a}))?;
    third.send(json!({"op":"fixture-input","text":"?"}))?;
    third.wait("CART A state=00000003 save=00000001 cookie=6B482CC3")?;
    third.send(json!({"op":"fixture-input","text":"f"}))?;
    third.wait("CART COMPLETE")?;
    third.quit()?;
    c::write_new(
        &app.join("runtime.qualified"),
        resources::Resources::open(&package)?.identity.as_bytes(),
    )?;
    c::write_new(&app.join("continuity.qualified"),b"original A unsaved2/save1 fresh-process restore; action3; second fresh restore3; A-B-A; real native snapshots; original UART input only\n")?;
    Ok(())
}

fn native(repo: &Path, w: &BuildPaths) -> Result<(), Error> {
    use serde_json::{Value, json};
    let project = w.root.parent().ok_or("project")?;
    let old = project.join("ramht-v1/build-v2-retry2");
    let src = project.join("ramht-v1/source-v2");
    let new = w.root.join("native-v2");
    c::mkdir(&new)?;
    c::mkdir(&new.join("ui"))?;
    if c::file_digest(&old.join("qemu-system-i386"))? != resources::REFERENCE_CORE {
        return Err("retained core differs".into());
    }
    fs::copy(src.join("ui/xemu.c"), new.join("ui/xemu.c"))?;
    let patch = repo.join("patches/xemu-headless-session.patch");
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&new)
            .args(["apply", "--unsafe-paths"])
            .arg(&patch),
    )?;
    let commands: Vec<Value> =
        serde_json::from_slice(&fs::read(old.join("compile_commands.json"))?)?;
    let cc = commands
        .iter()
        .find(|v| v["output"] == "libsystem.a.p/ui_xemu.c.o")
        .ok_or("compile command missing")?["command"]
        .as_str()
        .ok_or("command missing")?;
    let object = new.join("ui.o");
    let dep = new.join("ui.o.d");
    let cc = cc
        .replace("libsystem.a.p/ui_xemu.c.o.d", dep.to_str().unwrap())
        .replace("libsystem.a.p/ui_xemu.c.o", object.to_str().unwrap())
        .replace(
            "../source-v2/ui/xemu.c",
            new.join("ui/xemu.c").to_str().unwrap(),
        );
    let cc = format!("{cc} -I{}", src.join("ui").display());
    let ninja = fs::read_to_string(old.join("build.ninja"))?;
    let edge = ninja
        .split("\n\n")
        .find(|s| s.starts_with("build qemu-system-i386:"))
        .ok_or("link edge missing")?;
    let input = edge
        .lines()
        .next()
        .ok_or("link inputs missing")?
        .strip_prefix("build qemu-system-i386: cpp_LINKER_RSP ")
        .ok_or("unexpected link rule")?
        .split(" | ")
        .next()
        .ok_or("inputs")?;
    let input = input.replace("libsystem.a.p/ui_xemu.c.o", object.to_str().unwrap());
    let link = edge
        .lines()
        .find_map(|s| s.strip_prefix(" LINK_ARGS = "))
        .ok_or("link args missing")?;
    let link = link.replace(
        &format!(":{}", repo.join(".local/host-sysroot/usr/lib64").display()),
        "",
    );
    let build = format!(
        "builddir = {}\nrule compile_ui\n command = {cc}\nrule link_core\n command = c++ -m64 @$out.rsp\n rspfile = $out.rsp\n rspfile_content = -o $out $in {link}\nbuild {}: compile_ui {}\nbuild {}: link_core {input}\n",
        new.display(),
        object.display(),
        new.join("ui/xemu.c").display(),
        new.join("core").display()
    );
    c::write_new(&new.join("build.ninja"), build.as_bytes())?;
    c::store::checked(
        Command::new("ninja")
            .current_dir(&old)
            .arg("-f")
            .arg(new.join("build.ninja"))
            .args(["-j", "1"])
            .arg(new.join("core")),
    )?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(
            &json!({"base_core_sha256":resources::REFERENCE_CORE,"base_source_pin":"03b988c9d76553b33ed712ba00822ffdf49f0e7d","prior_base_and_RAMHT_patches_retained":true,"patch_sha256":c::file_digest(&patch)?,"retained_compile_commands_sha256":c::file_digest(&old.join("compile_commands.json"))?,"retained_ninja_sha256":c::file_digest(&old.join("build.ninja"))?,"modified_source_sha256":c::file_digest(&new.join("ui/xemu.c"))?,"core_sha256":c::file_digest(&new.join("core"))?,"recompiled_units":["ui/xemu.c"],"reused_other_objects":true,"old_build_mutated":false,"jobs":1}),
        )?,
    )?;
    Ok(())
}

fn partial(w: &BuildPaths) -> Result<(), Error> {
    let r = resources::Resources::open(&w.root.join("package-v3"))?;
    let app = qualified(w, &r)?;
    let library = c::library::Library::read(&app)?;
    let entry = library.entries.first().ok_or("original entry")?;
    let receipt = c::desktop::original_partial_load_test(&app, &r, entry)?;
    c::write_new(&app.join("partial-native-load.json"), &c::json(&receipt)?)?;
    println!("{}", serde_json::to_string(&receipt)?);
    Ok(())
}

fn memory(parent: u32) -> Vec<Value> {
    let mut pids = vec![parent];
    if let Ok(children) = fs::read_to_string(format!("/proc/{parent}/task/{parent}/children")) {
        pids.extend(
            children
                .split_whitespace()
                .filter_map(|s| s.parse::<u32>().ok()),
        );
    }
    pids.into_iter()
        .filter_map(|pid| {
            let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
            let fields: serde_json::Map<String, Value> = status
                .lines()
                .filter(|s| {
                    s.starts_with("VmHWM:") || s.starts_with("VmRSS:") || s.starts_with("VmPeak:")
                })
                .filter_map(|s| {
                    let (k, v) = s.split_once(':')?;
                    Some((
                        format!("{k}_kib"),
                        json!(v.split_whitespace().next()?.parse::<u64>().ok()?),
                    ))
                })
                .collect();
            Some(json!({"ephemeral_pid":pid,"readings":fields}))
        })
        .collect()
}
fn qualified(w: &BuildPaths, r: &resources::Resources) -> Result<std::path::PathBuf, Error> {
    fs::read_dir(&w.root)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.join("continuity.qualified").is_file()
                && fs::read_to_string(p.join("runtime.qualified")).is_ok_and(|id| id == r.identity)
        })
        .ok_or_else(|| "no completed original continuity proof for this exact runtime".into())
}

fn clone_original(w: &BuildPaths, r: &resources::Resources) -> Result<std::path::PathBuf, Error> {
    let source = qualified(w, r)?;
    let _app = crate::storage::ProfileLease::acquire(&source)?;
    let library = c::library::Library::read(&source)?;
    let dest = w.root.join(format!("policy-{}", c::new_id()?));
    c::mkdir(&dest)?;
    c::mkdir(&dest.join("profiles"))?;
    fs::copy(source.join("library.json"), dest.join("library.json"))?;
    for entry in &library.entries {
        if !resources::ORIGINAL_FIXTURES.contains(&entry.media_sha256.as_str()) {
            return Err("original fixtures only".into());
        }
        let p = c::store::Profile::open(&source, entry, r)?;
        let v = c::checkpoint::load(
            &p,
            r,
            &c::Binding {
                game: entry.id.clone(),
                profile: entry.profile.clone(),
                media: entry.media_sha256.clone(),
                partition: entry.partition,
                runtime: r.identity.clone(),
                state_abi: 1,
            },
        )?
        .ok_or("original checkpoint")?;
        let target = dest.join("profiles").join(&entry.profile);
        c::mkdir(&target)?;
        c::mkdir(&target.join("storage"))?;
        c::mkdir(&target.join("checkpoints"))?;
        for name in ["storage.json", "session.json", "generation"] {
            fs::copy(p.root.join(name), target.join(name))?;
        }
        let current = p.current()?;
        let disk = target
            .join("storage")
            .join(current.file_name().ok_or("storage")?);
        c::mkdir(&disk)?;
        c::store::copy_machine(&current, &disk)?;
        let checkpoint = target.join("checkpoints").join(&v.id);
        c::mkdir(&checkpoint)?;
        c::store::copy_machine(&v.directory, &checkpoint)?;
        fs::copy(
            v.directory.join("record.json"),
            checkpoint.join("record.json"),
        )?;
    }
    Ok(dest)
}
fn policy_proof(w: &BuildPaths) -> Result<(), Error> {
    let package = w.root.join("package-v3");
    let r = resources::Resources::open(&package)?;
    let app = clone_original(w, &r)?;
    let library = c::library::Library::read(&app)?;
    let a = library.entries.first().ok_or("game")?.id.clone();
    let profile = app.join("profiles").join(&library.entries[0].profile);
    println!(
        "{}",
        json!({"isolated_synthetic_app":app,"source_preserved":qualified(w,&r)?,"shared_policy":true})
    );
    let mut first = Transport::spawn(&app, &package)?;
    first.send(json!({"op":"open","game":a}))?;
    first.send(json!({"op":"fixture-input","text":"?"}))?;
    first.wait("CART A state=00000003 save=00000001 cookie=6B482CC3")?;
    first.send(json!({"op":"pause"}))?;
    first.quit()?;
    let mut second = Transport::spawn(&app, &package)?;
    second.send(json!({"op":"return"}))?;
    second.send(json!({"op":"fixture-input","text":"?"}))?;
    if second.send(json!({"op":"serial"}))?["text"]
        .as_str()
        .is_some_and(|s| !s.is_empty())
    {
        return Err("explicit Pause did not remain paused".into());
    }
    second.send(json!({"op":"resume"}))?;
    second.wait("CART A state=00000003 save=00000001 cookie=6B482CC3")?;
    second.send(json!({"op":"fixture-input","text":"a"}))?;
    second.wait("CART A state=00000004 save=00000001 cookie=7E9FC8A2")?;
    second.send(json!({"op":"commit"}))?;
    let old = fs::read(profile.join("session.json"))?;
    let session: c::checkpoint::Session = serde_json::from_slice(&old)?;
    let state = profile
        .join("checkpoints")
        .join(&session.checkpoint)
        .join("hdd.qcow2");
    let state_hash = c::file_digest(&state)?;
    second.send(json!({"op":"resume"}))?;
    second.send(json!({"op":"fixture-input","text":"s"}))?;
    second.wait("CART A state=00000004 save=00000004 cookie=7E9FC8A2")?;
    second.send(json!({"op":"stop"}))?;
    second.quit()?;
    let (_global, lock) = (
        crate::storage::ProfileLease::acquire(&app)?,
        crate::storage::ProfileLease::acquire(&profile)?,
    );
    c::atomic(&profile.join("session.json"), &old)?;
    let current: c::store::Current = c::read_json(&profile.join("storage.json"))?;
    let current = profile.join("storage").join(current.directory);
    let before = (
        c::file_digest(&current.join("hdd.qcow2"))?,
        c::file_digest(&current.join("eeprom.bin"))?,
        lock.generation()?,
        c::file_digest(&state)?,
        fs::read(profile.join("session.json"))?,
    );
    drop(lock);
    drop(_global);
    let mut third = Transport::spawn(&app, &package)?;
    let refusal = third.refusal(json!({"op":"open","game":a}))?;
    if !refusal.contains("generation") {
        return Err(format!("wrong refusal: {refusal}").into());
    }
    third.quit()?;
    let lock = crate::storage::ProfileLease::acquire(&profile)?;
    let after = (
        c::file_digest(&current.join("hdd.qcow2"))?,
        c::file_digest(&current.join("eeprom.bin"))?,
        lock.generation()?,
        c::file_digest(&state)?,
        fs::read(profile.join("session.json"))?,
    );
    drop(lock);
    if before != after || c::file_digest(&state)? != state_hash {
        return Err("newer ordinary save or rejected checkpoint was changed".into());
    }
    let mut fourth = Transport::spawn(&app, &package)?;
    fourth.send(json!({"op":"restart","game":a}))?;
    fourth.send(json!({"op":"open","game":a}))?;
    fourth.wait("CART A state=00000000 save=00000004")?;
    fourth.send(json!({"op":"fixture-input","text":"a"}))?;
    fourth.wait("CART A state=00000001 save=00000004")?;
    fourth.quit()?;
    if c::file_digest(&state)? != state_hash {
        return Err("explicit restart changed rejected checkpoint".into());
    }
    c::write_new(&app.join("policy.qualified"),b"shared policy actual fresh-process explicit Pause; normal Resume; newer guest disk save4 rejects old checkpoint with both unchanged; explicit restart cold-loads current save4; native returns clean\n")?;
    Ok(())
}

fn paused_contract(repo: &Path, w: &BuildPaths) -> Result<(), Error> {
    let root = w.root.join("paused-capture-contract-v2");
    c::mkdir(&root)?;
    let source = w
        .root
        .parent()
        .ok_or("project")?
        .join("ramht-v1/source-v2/migration/migration-hmp-cmds.c");
    let text = fs::read_to_string(&source)?;
    for (label, corrected) in [("before", false), ("after", true)] {
        let dir = root.join(label);
        c::mkdir(&dir)?;
        c::mkdir(&dir.join("migration"))?;
        fs::copy(&source, dir.join("migration/migration-hmp-cmds.c"))?;
        if corrected {
            c::store::checked(
                Command::new("git")
                    .arg("-C")
                    .arg(&dir)
                    .args(["apply", "--unsafe-paths"])
                    .arg(repo.join("patches/xemu-paused-savevm.patch")),
            )?;
        }
        let actual = fs::read_to_string(dir.join("migration/migration-hmp-cmds.c"))?;
        let start = actual
            .find("void hmp_savevm(")
            .ok_or("capture function missing")?;
        let end = start
            + actual[start..]
                .find("\nvoid hmp_delvm")
                .ok_or("capture function extent")?;
        c::write_new(&dir.join("subject.inc"), &actual.as_bytes()[start..end])?;
        fs::copy(
            repo.join("fixtures/paused_capture_shim.c"),
            dir.join("shim.c"),
        )?;
        fs::copy(
            repo.join("fixtures/paused_capture_contract.rs"),
            dir.join("contract.rs"),
        )?;
        c::store::checked(
            Command::new("cc")
                .args([
                    "-std=c11", "-O2", "-fPIC", "-Wall", "-Wextra", "-Werror", "-c",
                ])
                .arg(dir.join("shim.c"))
                .arg("-o")
                .arg(dir.join("subject.o")),
        )?;
        c::store::checked(
            Command::new("rustc")
                .args(["--edition=2024", "--test", "-Dwarnings"])
                .arg(dir.join("contract.rs"))
                .arg("-C")
                .arg(format!("link-arg={}", dir.join("subject.o").display()))
                .arg("-o")
                .arg(dir.join("test")),
        )?;
        let output = Command::new(dir.join("test"))
            .arg("--test-threads=1")
            .output()?;
        c::write_new(
            &dir.join("test.log"),
            &[output.stdout.as_slice(), output.stderr.as_slice()].concat(),
        )?;
        if output.status.success() != corrected {
            return Err(
                format!("unexpected capture contract {label} result; evidence retained").into(),
            );
        }
    }
    c::write_new(
        &root.join("result.json"),
        &c::json(
            &json!({"public_source_sha256":c::file_digest(&source)?,"public_source_file_bytes":text.len(),"patch_sha256":c::file_digest(&repo.join("patches/xemu-paused-savevm.patch"))?,"before_expected_failure":true,"after_pass":true,"native_or_game_input":false,"scope":"actual HMP wrapper with original controlled runstate/device-preparation/error dependencies, not native serialization or vCPU concurrency"}),
        )?,
    )?;
    Ok(())
}

fn native_paused(repo: &Path, w: &BuildPaths) -> Result<(), Error> {
    let project = w.root.parent().ok_or("project")?;
    let old = project.join("ramht-v1/build-v2-retry2");
    let src = project.join("ramht-v1/source-v2");
    let new = w.root.join("native-v3");
    let contract = w.root.join("paused-capture-contract-v2/result.json");
    let result: Value = c::read_json(&contract)?;
    if result["after_pass"] != true
        || result["patch_sha256"] != c::file_digest(&repo.join("patches/xemu-paused-savevm.patch"))?
    {
        return Err("original paused-capture regression gate missing".into());
    }
    c::mkdir(&new)?;
    c::mkdir(&new.join("migration"))?;
    fs::copy(
        src.join("migration/migration-hmp-cmds.c"),
        new.join("migration/migration-hmp-cmds.c"),
    )?;
    c::store::checked(
        Command::new("git")
            .arg("-C")
            .arg(&new)
            .args(["apply", "--unsafe-paths"])
            .arg(repo.join("patches/xemu-paused-savevm.patch")),
    )?;
    let commands: Vec<Value> =
        serde_json::from_slice(&fs::read(old.join("compile_commands.json"))?)?;
    let subject = "libsystem.a.p/migration_migration-hmp-cmds.c.o";
    let command = commands
        .iter()
        .find(|v| v["output"] == subject)
        .ok_or("HMP compile command")?["command"]
        .as_str()
        .ok_or("compile command")?;
    let object = new.join("hmp.o");
    let dep = new.join("hmp.o.d");
    let command = command
        .replace(&format!("{subject}.d"), dep.to_str().unwrap())
        .replace(subject, object.to_str().unwrap())
        .replace(
            "../source-v2/migration/migration-hmp-cmds.c",
            new.join("migration/migration-hmp-cmds.c").to_str().unwrap(),
        );
    let command = format!("{command} -I{}", src.join("migration").display());
    let ninja = fs::read_to_string(old.join("build.ninja"))?;
    let edge = ninja
        .split("\n\n")
        .find(|s| s.starts_with("build qemu-system-i386:"))
        .ok_or("native link edge")?;
    let input = edge
        .lines()
        .next()
        .ok_or("inputs")?
        .strip_prefix("build qemu-system-i386: cpp_LINKER_RSP ")
        .ok_or("native rule")?
        .split(" | ")
        .next()
        .ok_or("inputs")?;
    let ui = w.root.join("native-v2/ui.o");
    let identity: Value = c::read_json(&w.root.join("native-v2/build.identity.json"))?;
    if identity["core_sha256"] != resources::HEADLESS_CORE
        || identity["patch_sha256"]
            != c::file_digest(&repo.join("patches/xemu-headless-session.patch"))?
    {
        return Err("retained headless repair binding differs".into());
    }
    let input = input
        .replace(subject, object.to_str().unwrap())
        .replace("libsystem.a.p/ui_xemu.c.o", ui.to_str().unwrap());
    let link = edge
        .lines()
        .find_map(|s| s.strip_prefix(" LINK_ARGS = "))
        .ok_or("link args")?
        .replace(
            &format!(":{}", repo.join(".local/host-sysroot/usr/lib64").display()),
            "",
        );
    let build = format!(
        "builddir = {}\nrule compile_hmp\n command = {command}\nrule link_core\n command = c++ -m64 @$out.rsp\n rspfile = $out.rsp\n rspfile_content = -o $out $in {link}\nbuild {}: compile_hmp {}\nbuild {}: link_core {input}\n",
        new.display(),
        object.display(),
        new.join("migration/migration-hmp-cmds.c").display(),
        new.join("core").display()
    );
    c::write_new(&new.join("build.ninja"), build.as_bytes())?;
    c::store::checked(
        Command::new("ninja")
            .current_dir(&old)
            .arg("-f")
            .arg(new.join("build.ninja"))
            .args(["-j", "1"])
            .arg(new.join("core")),
    )?;
    c::write_new(
        &new.join("build.identity.json"),
        &c::json(
            &json!({"base_pin":"03b988c9d76553b33ed712ba00822ffdf49f0e7d","prior_core_sha256":resources::HEADLESS_CORE,"prior_headless_object_sha256":c::file_digest(&ui)?,"paused_capture_patch_sha256":c::file_digest(&repo.join("patches/xemu-paused-savevm.patch"))?,"regression_result_sha256":c::file_digest(&contract)?,"core_sha256":c::file_digest(&new.join("core"))?,"recompiled_units":["migration/migration-hmp-cmds.c"],"reused_previous_headless_repair":true,"accepted_base_RAMHT_retained":true,"prior_builds_untouched":true}),
        )?,
    )?;
    Ok(())
}
