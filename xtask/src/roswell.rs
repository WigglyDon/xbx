//! One pinned, serial, capacity-bounded Roswell release build.
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const PIN: &str = "1569e2e89fb47cc72b9c704a8884f98200432bd4";
const GIB: u64 = 1 << 30;

fn output(command: &mut Command) -> Result<String, Box<dyn std::error::Error>> {
    let out = command.output()?;
    if !out.status.success() {
        return Err(format!("command failed: {command:?}").into());
    }
    Ok(String::from_utf8(out.stdout)?)
}

fn hash(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    Ok(output(Command::new("sha256sum").arg(path))?
        .split_whitespace()
        .next()
        .ok_or("missing SHA-256")?
        .to_owned())
}

fn within_limits(free: u64, allocated: u64) -> Result<(), &'static str> {
    // Stop with 1 GiB of headroom inside both hard limits. The existing local
    // policy's 12 GiB reserve is stricter than the assignment's 10 GiB floor.
    if free < 13 * GIB || allocated >= 7 * GIB {
        return Err("Roswell capacity guard: preserve 12 GiB free and stop below 8 GiB growth");
    }
    Ok(())
}

fn capacity(root: &Path) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let free = output(Command::new("df").args(["-B1", "--output=avail"]).arg(root))?
        .lines()
        .last()
        .ok_or("missing free-space accounting")?
        .trim()
        .parse()?;
    let usage = output(
        Command::new("du")
            .args(["-s", "-B1"])
            .arg(root.join(".local/roswell-v1"))
            .arg(root.join(".local/evidence/roswell-v1")),
    )?;
    let mut allocated = 0u64;
    for row in usage.lines() {
        allocated = allocated
            .checked_add(
                row.split_whitespace()
                    .next()
                    .ok_or("missing allocation")?
                    .parse()?,
            )
            .ok_or("allocation accounting overflow")?;
    }
    within_limits(free, allocated)?;
    crate::capacity::check(root, 1)?;
    Ok((free, allocated))
}

struct Group {
    child: Child,
    reaped: bool,
}
impl Drop for Group {
    fn drop(&mut self) {
        if self.reaped {
            return;
        }
        // Only the process group created below. Kill all owned build descendants
        // on an error/guard refusal, rather than leaving a compiler behind.
        unsafe extern "C" {
            fn kill(pid: i32, signal: i32) -> i32;
        }
        unsafe {
            kill(-(self.child.id() as i32), 9);
        }
        let _ = self.child.wait();
    }
}

fn bounded(
    root: &Path,
    build: &Path,
    phase: &str,
    lease: &File,
    command: &mut Command,
) -> Result<(), Box<dyn std::error::Error>> {
    capacity(root)?;
    let log = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(build.join(format!("{phase}.log")))?;
    let mut samples = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(build.join(format!("{phase}.capacity")))?;
    fs::write(
        build.join(format!("{phase}.command")),
        format!("{command:?}\n"),
    )?;
    let descriptor = lease.as_raw_fd();
    // Descendants retain the same writer lease even if the Rust parent dies.
    unsafe {
        command.pre_exec(move || {
            unsafe extern "C" {
                fn fcntl(fd: i32, command: i32, ...) -> i32;
            }
            if fcntl(descriptor, 2, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    command
        .process_group(0)
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log));
    let mut child = Group {
        child: command.spawn()?,
        reaped: false,
    };
    let start = Instant::now();
    loop {
        if let Some(status) = child.child.try_wait()? {
            child.reaped = true;
            fs::write(
                build.join(format!("{phase}.termination")),
                format!(
                    "status={status}\nelapsed_seconds={:.3}\n",
                    start.elapsed().as_secs_f64()
                ),
            )?;
            if !status.success() {
                return Err(format!("Roswell {phase} failed; attempt and logs retained").into());
            }
            capacity(root)?;
            return Ok(());
        }
        match capacity(root) {
            Ok((free, allocated)) => writeln!(
                samples,
                "elapsed={:.3} free={free} allocated={allocated}",
                start.elapsed().as_secs_f64()
            )?,
            Err(error) => {
                fs::write(build.join(format!("{phase}.refusal")), error.to_string())?;
                return Err(error);
            }
        }
        if start.elapsed() > Duration::from_secs(7200) {
            return Err("Roswell build phase exceeded two-hour bound; attempt retained".into());
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

pub fn command(root: &Path, action: &str, attempt: &str) -> Result<(), Box<dyn std::error::Error>> {
    if attempt.is_empty()
        || attempt.len() > 16
        || !attempt
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("Roswell attempt must be a short simple label".into());
    }
    let root = root.canonicalize()?;
    let lease = crate::capacity::writer(&root, 4)?;
    capacity(&root)?;
    let base = root.join(".local/roswell-v1");
    let source = base.join(format!("source-{attempt}"));
    let build = base.join(format!("build-{attempt}"));
    let patch_path = root.join("patches/roswell-build.patch");
    let patch = if patch_path.exists() {
        fs::read(&patch_path)?
    } else {
        Vec::new()
    };
    crate::kernel::verify(&source, PIN, Some(&patch))?;
    let dep_hash = hash(&base.join("dependencies.identity"))?;
    let lock = fs::read_to_string(root.join("roswell.lock.json"))?;
    if !lock.contains(&format!("\"commit\": \"{PIN}\""))
        || !lock.contains(&format!("\"dependencies_sha256\": \"{dep_hash}\""))
    {
        return Err("Roswell source/dependency lock differs".into());
    }
    output(
        Command::new("sha256sum")
            .args(["-c", "dependencies.identity"])
            .current_dir(&base),
    )?;
    let source_patch = output(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .args(["diff", "--binary", "HEAD"]),
    )?;
    let inputs = format!(
        "source={PIN}\nmode=Release\ndbg=0\narch=i386\nsarch=xbox\ntarget=flash\ncompiler_jobs=1\ndependencies_sha256={dep_hash}\nsource_patch_bytes={}\n",
        source_patch.len()
    );
    let tools = base.join("sysroot/usr");
    let path = std::env::join_paths(std::iter::once(tools.join("bin")).chain(
        std::env::split_paths(&std::env::var_os("PATH").ok_or("PATH unavailable")?),
    ))?;
    let mut cmake = Command::new("cmake");
    cmake
        .env("PATH", path)
        .env("BISON_PKGDATADIR", tools.join("share/bison"))
        .env("M4", tools.join("bin/m4"))
        .env("CMAKE_BUILD_PARALLEL_LEVEL", "1");
    if action == "roswell-configure" {
        fs::create_dir(&build)?;
        fs::write(build.join("build.inputs"), &inputs)?;
        fs::write(build.join("source.patch"), &source_patch)?;
        cmake
            .arg("-S")
            .arg(&source)
            .arg("-B")
            .arg(&build)
            .args(["-G", "Ninja"])
            .arg(format!(
                "-DCMAKE_TOOLCHAIN_FILE={}",
                source.join("toolchain-gcc.cmake").display()
            ))
            .args([
                "-DCMAKE_BUILD_TYPE=Release",
                "-DDBG=0",
                "-DARCH=i386",
                "-DSARCH=xbox",
                "-DHOST_BUILD_TYPE=Release",
                "-DENABLE_CCACHE=OFF",
            ]);
        bounded(&root, &build, "configure", &lease, &mut cmake)?;
        println!("Configured pinned Roswell release: {}", build.display());
    } else if action == "roswell-build" {
        if fs::read_to_string(build.join("build.inputs"))? != inputs
            || fs::read_to_string(build.join("source.patch"))? != source_patch
            || build.join("build.identity").exists()
        {
            return Err(
                "Roswell attempt changed or already complete; preserve and select another attempt"
                    .into(),
            );
        }
        cmake
            .arg("--build")
            .arg(&build)
            .args(["--target", "flash", "--parallel", "1"]);
        bounded(&root, &build, "build", &lease, &mut cmake)?;
        fs::write(
            build.join("build.identity"),
            format!("{inputs}flash_sha256={}\n", hash(&build.join("flash.bin"))?),
        )?;
        println!("Built pinned Roswell release: {}", build.display());
    } else {
        return Err("unknown Roswell action".into());
    }
    Ok(())
}

/// Carry the same slice limits across an ordinary bounded native observation.
pub fn observe(
    root: &Path,
    attempt: &str,
    label: &str,
    memory: u16,
    media: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    for value in [attempt, label] {
        if value.is_empty()
            || value.len() > 16
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("Roswell observation labels must be short and simple".into());
        }
    }
    if !matches!(memory, 64 | 128) {
        return Err("Roswell memory must be 64 or 128 MiB".into());
    }
    let root = root.canonicalize()?;
    let lease = crate::capacity::writer(&root, 4)?;
    capacity(&root)?;
    let base = root.join(".local/roswell-v1");
    let build = base.join(format!("build-{attempt}"));
    let flash = build.join("flash.bin");
    let flash_hash = hash(&flash)?;
    let identity = fs::read_to_string(build.join("build.identity"))?;
    if ![
        format!("source={PIN}\n"),
        "mode=Release\n".into(),
        "dbg=0\n".into(),
        "target=flash\n".into(),
        format!("flash_sha256={flash_hash}\n"),
    ]
    .iter()
    .all(|value| identity.contains(value))
    {
        return Err("Roswell flash does not match completed release build".into());
    }
    let core = root.join(".local/build-reference-03b988c9d76553b33ed712ba00822ffdf49f0e7d-4e317283ddb368c1c80243804aa7fcfdc1e5228ac6d7ddec9ca84cb5cdd138e1-v3/qemu-system-i386");
    if hash(&core)? != "31e1c0f5f3c165d25e9c6315ecaf3191b972f57ae22616d03de6ee8d6243be3d" {
        return Err("retained upstream reference core identity differs".into());
    }
    let disk = root.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2");
    if hash(&disk)? != "00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd" {
        return Err("pristine dashboard seed identity differs".into());
    }
    let profile = base.join(format!("profile-{label}"));
    if profile.exists() {
        return Err("Roswell profile already exists; preserve it".into());
    }
    let host = base.join("host-target/debug/xbx-host");
    let observation = base.join(format!("observe-{label}"));
    fs::create_dir(&observation)?;
    let inputs = format!(
        "source={PIN}\nmode=Release\ndbg=0\nflash_sha256={flash_hash}\ncore_sha256={}\nhost_sha256={}\nmemory_mib={memory}\ndisc_present={}\nnetwork={}\nobservation_seconds=300\n",
        hash(&core)?,
        hash(&host)?,
        media.is_some(),
        if media.is_some() {
            "core_default"
        } else {
            "none"
        }
    );
    fs::write(observation.join("qualification.inputs"), &inputs)?;
    let mut command = Command::new(host);
    command
        .env("XBX_DESKTOP_MEMORY_MIB", memory.to_string())
        .env("XBX_DESKTOP_VIDEO_DRIVER", "x11")
        .env("G_DEBUG", "fatal-criticals")
        .arg(if media.is_some() {
            "run-title"
        } else {
            "run-dashboard"
        })
        .arg(core)
        .arg(&profile)
        .arg(flash)
        .arg(disk)
        .arg(root.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin"));
    if let Some(media) = media {
        command.arg(if media.is_absolute() {
            media.to_path_buf()
        } else {
            root.join(media)
        });
    }
    command.arg("300");
    bounded(&root, &observation, "run", &lease, &mut command)?;
    let termination = fs::read_to_string(profile.join("termination.txt"))?;
    if ![
        "status=exit status: 0\n",
        "forced=false\n",
        "qmp=Ok(())\n",
        "storage_synced=true\n",
    ]
    .iter()
    .all(|s| termination.contains(s))
    {
        return Err("ordinary native termination not verified; profile retained".into());
    }
    let profile_lease = OpenOptions::new()
        .read(true)
        .write(true)
        .open(profile.join("writer.lock"))?;
    profile_lease.try_lock()?;
    fs::write(profile.join("qualification.inputs"), inputs)?;
    File::open(profile.join("qualification.inputs"))?.sync_all()?;
    File::open(&profile)?.sync_all()?;
    println!(
        "Roswell observation retained at {}; visible acceptance requires owned-window review.",
        profile.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capacity_guard_stops_with_headroom_before_either_hard_limit() {
        assert!(within_limits(14 * GIB, 6 * GIB).is_ok());
        assert!(within_limits(13 * GIB - 1, 6 * GIB).is_err());
        assert!(within_limits(20 * GIB, 7 * GIB).is_err());
        assert!(within_limits(20 * GIB, 8 * GIB).is_err());
    }
}
