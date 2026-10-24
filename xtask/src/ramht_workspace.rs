//! Bounded admission for the single RAMHT compatibility candidate.
//! Uses the existing host binding/lease; never changes internal-host guards.
use std::cell::Cell;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::os::unix::{
    fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    process::CommandExt,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

type Error = Box<dyn std::error::Error>;
const GIB: u64 = 1024 * 1024 * 1024;
const GROWTH: u64 = 6 * GIB;

pub struct Workspace {
    pub root: PathBuf,
    project: PathBuf,
    mount: PathBuf,
    uuid: String,
    maximum: u64,
    shared: File,
    project_lock: File,
    phase_record: Option<PathBuf>,
    phase: String,
    successful: Cell<bool>,
}

fn output(command: &mut Command) -> Result<String, Error> {
    let out = command.output()?;
    if !out.status.success() {
        return Err("shared-workspace accounting failed".into());
    }
    Ok(String::from_utf8(out.stdout)?)
}

fn required(key: &str) -> Result<String, Error> {
    std::env::var(key).map_err(|_| format!("missing private workspace binding: {key}").into())
}

impl Workspace {
    pub fn acquire(repo: &Path, phase: &str) -> Result<Self, Error> {
        let shared = OpenOptions::new()
            .read(true)
            .write(true)
            .open(required("XBX_SHARED_CAPACITY_LEASE")?)?;
        shared
            .try_lock()
            .map_err(|_| "shared capacity lease busy; no phase started")?;
        let project_lock = OpenOptions::new()
            .read(true)
            .write(true)
            .open(repo.join(".local/build.lock"))?;
        project_lock
            .try_lock()
            .map_err(|_| "another xbx writer is active")?;
        let project = PathBuf::from(required("XBX_SHARED_ROOT")?).join("projects/xbx");
        let mut this = Self {
            root: project.join("ramht-v1"),
            project,
            mount: PathBuf::from(required("XBX_SHARED_MOUNT")?),
            uuid: required("XBX_SHARED_UUID")?,
            maximum: 0,
            shared,
            project_lock,
            phase_record: None,
            phase: phase.into(),
            successful: Cell::new(false),
        };
        this.presence()?;
        let baseline = this.allocated()?;
        if this.available()? < 12 * GIB + GROWTH {
            return Err("preserve shared 12 GiB reserve plus the 6 GiB candidate envelope".into());
        }
        if !this.root.exists() {
            fs::DirBuilder::new().mode(0o700).create(&this.root)?;
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(this.root.join("allocation-base.bytes"))?;
            writeln!(f, "{baseline}")?;
            f.sync_all()?;
            File::open(&this.root)?.sync_all()?;
        }
        this.maximum = fs::read_to_string(this.root.join("allocation-base.bytes"))?
            .trim()
            .parse::<u64>()?
            .checked_add(GROWTH)
            .ok_or("allocation bound overflow")?;
        this.check()?;
        this.phase_record = std::env::var_os("XBX_SHARED_PHASE_RECORD").map(PathBuf::from);
        this.record("active", true)?;
        Ok(this)
    }

    pub fn completed(&self) {
        self.successful.set(true);
    }

    fn record(&self, status: &str, held: bool) -> Result<(), Error> {
        if let Some(path) = &self.phase_record {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();
            let growth = if held { GROWTH } else { 0 };
            fs::write(
                path,
                format!(
                    "{{\"owner\":\"xbx\",\"phase\":\"{}\",\"pid\":{},\"utc_unix_seconds\":{timestamp},\"status\":\"{status}\",\"lease_held\":{held},\"data_growth_bound\":{GROWTH},\"shared_floor\":{},\"outstanding_growth_commitment\":{growth}}}\n",
                    self.phase,
                    std::process::id(),
                    12 * GIB
                ),
            )?;
        }
        Ok(())
    }

    pub fn presence(&self) -> Result<(), Error> {
        let result = output(
            Command::new("findmnt")
                .args(["-n", "-o", "UUID,FSTYPE,TARGET", "-T"])
                .arg(&self.project),
        )?;
        let fields: Vec<_> = result.split_whitespace().collect();
        if fields
            != [
                self.uuid.as_str(),
                "ext4",
                self.mount.to_str().ok_or("non-UTF8 mount")?,
            ]
        {
            return Err("missing or wrong workspace mount; no internal fallback".into());
        }
        unsafe extern "C" {
            fn getuid() -> u32;
        }
        let uid = unsafe { getuid() };
        let mounted = fs::metadata(&self.mount)?;
        for p in [
            &self.project,
            self.project
                .parent()
                .ok_or("missing projects parent")?
                .parent()
                .ok_or("missing workspace")?,
        ] {
            let m = fs::symlink_metadata(p)?;
            if !m.is_dir()
                || m.dev() != mounted.dev()
                || m.uid() != uid
                || m.mode() & 0o777 != 0o700
            {
                return Err("workspace ownership, permission or device mismatch".into());
            }
        }
        if self.root.exists() {
            let m = fs::symlink_metadata(&self.root)?;
            if !m.is_dir()
                || m.dev() != mounted.dev()
                || m.uid() != uid
                || m.mode() & 0o777 != 0o700
            {
                return Err("candidate directory is not the admitted private directory".into());
            }
        }
        Ok(())
    }

    fn available(&self) -> Result<u64, Error> {
        let out = output(
            Command::new("stat")
                .args(["-f", "-c", "%a %S %d"])
                .arg(&self.project),
        )?;
        let n: Vec<u64> = out
            .split_whitespace()
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        if n.len() != 3 || n[2] < 4096 {
            return Err("ext4 inode headroom refused".into());
        }
        n[0].checked_mul(n[1])
            .ok_or_else(|| "available-byte overflow".into())
    }

    fn allocated(&self) -> Result<u64, Error> {
        for _ in 0..3 {
            let out = Command::new("du")
                .args(["-s", "-B1"])
                .arg(&self.project)
                .env("LC_ALL", "C")
                .output()?;
            if out.status.success() {
                return Ok(String::from_utf8(out.stdout)?
                    .split_whitespace()
                    .next()
                    .ok_or("missing allocation")?
                    .parse()?);
            }
            let error = String::from_utf8(out.stderr)?;
            if out.status.code() != Some(1)
                || error.is_empty()
                || !error.lines().all(|l| {
                    l.starts_with("du: cannot access ")
                        && l.contains(self.project.to_str().unwrap_or("\0"))
                        && l.ends_with("No such file or directory")
                })
            {
                return Err("allocation inventory refused; outputs preserved".into());
            }
            self.presence()?;
        }
        Err("allocation inventory did not stabilize after three reads".into())
    }

    pub fn check(&self) -> Result<(), Error> {
        self.presence()?;
        let allocated = self.allocated()?;
        if allocated > self.maximum
            || self.available()? < 12 * GIB + self.maximum.saturating_sub(allocated)
        {
            return Err("RAMHT candidate allocation or shared reserve bound refused".into());
        }
        Ok(())
    }

    pub fn run(
        &self,
        command: &mut Command,
        log: &Path,
        seconds: u64,
    ) -> Result<std::process::ExitStatus, Error> {
        self.check()?;
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(log)?;
        command
            .stdout(Stdio::from(file.try_clone()?))
            .stderr(Stdio::from(file));
        for (key, relative) in [
            ("TMPDIR", "tmp"),
            ("PIP_CACHE_DIR", "pip-cache"),
            ("PYTHONPYCACHEPREFIX", "pycache"),
            ("XDG_CACHE_HOME", "cache"),
        ] {
            let path = self.root.join(relative);
            fs::create_dir_all(&path)?;
            command.env(key, path);
        }
        command
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .env("PYTHONUNBUFFERED", "1")
            .env("PIP_NO_INDEX", "1")
            .env("PIP_DISABLE_PIP_VERSION_CHECK", "1")
            .process_group(0);
        let fds = [self.shared.as_raw_fd(), self.project_lock.as_raw_fd()];
        unsafe extern "C" {
            fn fcntl(fd: i32, cmd: i32, ...) -> i32;
            fn setrlimit(resource: i32, limits: *const [u64; 2]) -> i32;
            fn kill(pid: i32, signal: i32) -> i32;
        }
        unsafe {
            command.pre_exec(move || {
                for fd in fds {
                    if fcntl(fd, 2, 0) < 0 {
                        return Err(io::Error::last_os_error());
                    }
                }
                if setrlimit(4, &[0, 0]) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut child = command.spawn()?;
        let start = Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                if group_alive(child.id())? {
                    unsafe {
                        kill(-(child.id() as i32), 15);
                    }
                    std::thread::sleep(Duration::from_millis(500));
                    unsafe {
                        kill(-(child.id() as i32), 9);
                    }
                    while group_alive(child.id())? {
                        std::thread::sleep(Duration::from_secs(1));
                    }
                    return Err("unexpected owned descendants retired; output unqualified".into());
                }
                self.check()?;
                return Ok(status);
            }
            let check = self.check();
            if check.is_err() || start.elapsed() > Duration::from_secs(seconds) {
                // A fresh process group contains only this operation's descendants.
                unsafe {
                    kill(-(child.id() as i32), 15);
                }
                std::thread::sleep(Duration::from_millis(500));
                unsafe {
                    kill(-(child.id() as i32), 9);
                }
                child.wait()?; // Retain custody if an owned writer cannot retire.
                while group_alive(child.id())? {
                    eprintln!("Waiting for owned operation writers; shared lease retained.");
                    std::thread::sleep(Duration::from_secs(1));
                }
                return Err(check
                    .err()
                    .unwrap_or_else(|| "finite RAMHT operation deadline".into()));
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = self.record(
            if self.successful.get() {
                "complete"
            } else {
                "stopped"
            },
            false,
        );
    }
}

fn group_alive(group: u32) -> Result<bool, Error> {
    for item in fs::read_dir("/proc")? {
        let path = item?.path();
        if !path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().bytes().all(|b| b.is_ascii_digit()))
        {
            continue;
        }
        let Ok(stat) = fs::read_to_string(path.join("stat")) else {
            continue;
        };
        let Some((_, fields)) = stat.rsplit_once(')') else {
            continue;
        };
        let values: Vec<_> = fields.split_whitespace().collect();
        if values.get(2).and_then(|x| x.parse::<u32>().ok()) == Some(group)
            && values.first() != Some(&"Z")
        {
            return Ok(true);
        }
    }
    Ok(false)
}
