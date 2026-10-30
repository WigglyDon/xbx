// SPDX-License-Identifier: MIT
//! Original, bounded CPU workload motivated by actual native host profiling.
//! It measures no commercial gameplay and changes no ordinary runtime default.
use super::{self as c, resources};
use serde_json::json;
use std::{
    fs,
    io::Write,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};
type Error = Box<dyn std::error::Error>;
fn completed_workload(root: &Path, name: &str, core: &str) -> Result<serde_json::Value, Error> {
    let v: serde_json::Value = c::read_json(
        &root
            .join("cpu-performance-v1")
            .join(name)
            .join("result.json"),
    )?;
    if v["core"] != core
        || v["guest_passed"] != true
        || v["clean"] != true
        || v["seeds_unchanged"] != true
    {
        return Err("actual original workload and clean return required".into());
    }
    Ok(v)
}
pub(super) fn candidate_io(repo: &Path, root: &Path) -> Result<(), Error> {
    completed_workload(root, "candidate", resources::CPU_FLOAT_CORE)?;
    let dir = root.join("cpu-float-v1/original-io");
    c::mkdir(&dir)?;
    let media = root.join("file-io-errors-v1/original.iso");
    if c::file_digest(&media)? != "f1e46094955b06096eada46bbc5a740f36e5ae1d50f91deb9d7bbfd6265945c6"
    {
        return Err("retained original I/O fixture differs".into());
    }
    let r = resources::Resources::cpu_float(&root.join("cpu-float-v1/package"))?;
    super::file_read::observe_with(repo, root, &dir, &media, false, r)
}
fn phase_ms(v: &serde_json::Value, phase: &str) -> Result<u64, Error> {
    phase_delta(v, phase, "host_received_ms")
}
fn phase_delta(v: &serde_json::Value, phase: &str, field: &str) -> Result<u64, Error> {
    let rows = v["events"].as_array().ok_or("workload events")?;
    let time = |suffix: &str| -> Result<u64, Error> {
        let marker = format!("{phase} {suffix}");
        rows.iter()
            .find(|r| r["marker"] == marker)
            .and_then(|r| r[field].as_u64())
            .ok_or_else(|| "workload phase timestamp".into())
    };
    time("DONE")?
        .checked_sub(time("BEGIN")?)
        .filter(|v| *v > 0)
        .ok_or_else(|| "workload timing resolution insufficient".into())
}
/// One finite ABBA cohort on original code. This does not reserve the host or
/// establish a formal host benchmark: thread runtime excludes descheduling but
/// still varies with clock frequency, interrupts and other host work.
pub(super) fn paired(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("cpu-performance-v1/paired-v1");
    c::mkdir(&dir)?;
    let mut rows = Vec::new();
    for (label, candidate) in [
        ("baseline-first", false),
        ("candidate-first", true),
        ("candidate-second", true),
        ("baseline-last", false),
    ] {
        let out = dir.join(label);
        observe_into(repo, root, candidate, &out, true)?;
        let v: serde_json::Value = c::read_json(&out.join("result.json"))?;
        let mut phases = serde_json::Map::new();
        for phase in ["DISPATCH", "SSE EXACT", "SSE STICKY"] {
            phases.insert(phase.into(), json!({"vcpu_runtime_ns":phase_delta(&v,phase,"vcpu_runtime_ns")?,"host_ms":phase_ms(&v,phase)?}));
        }
        rows.push(json!({"label":label,"core":v["core"],"fixture":v["fixture"],"clean":v["clean"],"guest_passed":v["guest_passed"],"seeds_unchanged":v["seeds_unchanged"],"phases":phases}));
    }
    let runtime = |row: usize, phase: &str| {
        rows[row]["phases"][phase]["vcpu_runtime_ns"]
            .as_u64()
            .unwrap()
    };
    let benefit = [1, 2].into_iter().all(|i| {
        runtime(i, "SSE EXACT") * 100 < runtime(0, "SSE EXACT").min(runtime(3, "SSE EXACT")) * 90
    });
    let controls = ["DISPATCH", "SSE STICKY"].into_iter().all(|phase| {
        [1, 2].into_iter().all(|i| {
            runtime(i, phase) <= runtime(0, phase).max(runtime(3, phase)) * 5 / 4 + 20_000_000
        })
    });
    c::write_new(
        &dir.join("result.json"),
        &c::json(
            &json!({"rows":rows,"clear_exact_benefit":benefit,"controls_within_bound":controls,"commercial_acceptance":false,"limit":"Finite original workload, host not reserved. Linux per-vCPU schedstat runtime at UART markers; excludes runqueue delay, not host frequency/interrupt effects. No scheduler, governor or peer changes."}),
        )?,
    )?;
    Ok(())
}
pub(super) fn fable(
    repo: &Path,
    root: &Path,
    returning: bool,
    vsync_off: bool,
) -> Result<(), Error> {
    // The earlier sub-second host-only samples were inconclusive. Preserve them;
    // admission uses the separately retained finite per-vCPU-runtime cohort.
    let paired: serde_json::Value =
        c::read_json(&root.join("cpu-performance-v1/paired-v1/result.json"))?;
    let expected = completed_workload(root, "candidate", resources::CPU_FLOAT_CORE)?;
    let rows = paired["rows"]
        .as_array()
        .ok_or("paired original workload")?;
    if rows.len() != 4
        || paired["clear_exact_benefit"] != true
        || paired["controls_within_bound"] != true
    {
        return Err("no supported original-workload benefit; preserve without game replay".into());
    }
    for (i, row) in rows.iter().enumerate() {
        let core = if i == 0 || i == 3 {
            resources::APU_QUIESCENT_CORE
        } else {
            resources::CPU_FLOAT_CORE
        };
        if row["core"] != core
            || row["fixture"] != expected["fixture"]
            || row["clean"] != true
            || row["guest_passed"] != true
            || row["seeds_unchanged"] != true
        {
            return Err("paired original identities or return differ".into());
        }
    }
    let io: serde_json::Value =
        c::read_json(&root.join("cpu-float-v1/original-io/original.result.json"))?;
    if io["core"] != resources::CPU_FLOAT_CORE
        || io["guest_passed"] != true
        || io["data_verified"] != true
        || io["native_clean"] != true
        || io["seeds_unchanged"] != true
    {
        return Err("changed-core original I/O gate required".into());
    }
    let mut r = resources::Resources::cpu_float(&root.join("cpu-float-v1/package"))?;
    if vsync_off {
        if !returning {
            return Err("presentation comparison requires retained gameplay checkpoint".into());
        }
        r.presentation_vsync = false;
    }
    if returning {
        let prior: serde_json::Value =
            c::read_json(&root.join("fable-cpu-float.native-started.json"))?;
        let app = prior["app"]
            .as_str()
            .ok_or("existing arithmetic-candidate app")?;
        super::build::observe_local_play(
            repo,
            root,
            r,
            &format!(
                "fable-cpu-float-{}-{}",
                if vsync_off { "vsync-control" } else { "return" },
                c::new_id()?
            ),
            Some(app.into()),
            true,
        )
    } else {
        if root.join("fable-cpu-float.native-started.json").exists() {
            return Err("game observation already started; reconcile existing session".into());
        }
        super::build::observe_local_play(repo, root, r, "fable-cpu-float", None, false)
    }
}
pub(super) fn float_core(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("cpu-float-v1");
    let tested: serde_json::Value = c::read_json(&dir.join("candidate-reference.json"))?;
    if tested["passed"] != true
        || tested["retained_core_and_source_unchanged"] != true
        || tested["compiler_hardening_unchanged"] != true
    {
        return Err("complete independent arithmetic checks required".into());
    }
    let patch = repo.join("patches/xemu-f32-exact-flags.patch");
    if tested["patch"] != c::file_digest(&patch)?
        || tested["subject_source"] != c::file_digest(&dir.join("source/fpu/softfloat.c"))?
    {
        return Err("tested arithmetic candidate differs".into());
    }
    if dir.join("native.identity.json").exists() {
        return Err("native arithmetic candidate already built; reuse actual identity".into());
    }
    let build = dir.join("build");
    let dry = Command::new("ninja")
        .current_dir(&build)
        .args(["-n", "qemu-system-i386"])
        .env("NINJA_STATUS", "[%f/%t] ")
        .output()?;
    let plan = String::from_utf8(dry.stdout)?;
    let steps = plan.lines().filter(|v| v.starts_with('[')).count();
    c::write_new(
        &dir.join(format!("native-plan-{}.log", c::new_id()?)),
        plan.as_bytes(),
    )?;
    if !dry.status.success()
        || steps > 32
        || !plan.contains("fpu_softfloat.c.o")
        || plan.to_ascii_lowercase().contains("regenerating build")
    {
        return Err("arithmetic native build exceeds targeted objects".into());
    }
    let log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(format!("native-build-{}.log", c::new_id()?)))?;
    c::store::checked(
        Command::new("ninja")
            .current_dir(&build)
            .args(["-j1", "qemu-system-i386"])
            .stdout(log.try_clone()?)
            .stderr(log),
    )?;
    let core = c::file_digest(&build.join("qemu-system-i386"))?;
    let old = root.join("apu-quiescence-v1/corrected-runtime-v1");
    let original: serde_json::Value = c::read_json(&dir.join("baseline-reference.json"))?;
    if core == resources::APU_QUIESCENT_CORE
        || c::file_digest(&old.join("build/qemu-system-i386"))? != resources::APU_QUIESCENT_CORE
        || original["subject_source"] != c::file_digest(&old.join("source/fpu/softfloat.c"))?
    {
        return Err("new native identity or preservation differs".into());
    }
    let package = dir.join("package");
    c::mkdir(&package)?;
    fs::create_dir_all(package.join("native/subprojects/slirp"))?;
    let base = root.join("apu-quiescence-v1/package");
    let baseline = resources::Resources::apu_quiescent(&base)?;
    for (from, to) in [
        (build.join("qemu-system-i386"), package.join("native/core")),
        (base.join("open.bin"), package.join("open.bin")),
        (
            base.join("native/qemu-img"),
            package.join("native/qemu-img"),
        ),
        (
            base.join("native/subprojects/slirp/libslirp.so.0"),
            package.join("native/subprojects/slirp/libslirp.so.0"),
        ),
    ] {
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(from)
                .arg(to),
        )?;
    }
    let mut manifest = baseline.manifest;
    manifest.core = core.clone();
    c::write_new(&package.join("runtime.json"), &c::json(&manifest)?)?;
    c::write_new(
        &dir.join("native.identity.json"),
        &c::json(
            &json!({"core":core,"open_system":resources::LOCAL_BASE_OPEN,"patch":tested["patch"],"subject_source":tested["subject_source"],"reference_program":tested["reference_program"],"planned_native_steps":steps,"compiler_jobs":1,"compiler_hardening_unchanged":true,"retained_core_source_and_package_unchanged":true,"separate_developer_package":true,"native_or_Fable_qualified":false,"ordinary_default_changed":false}),
        )?,
    )?;
    Ok(())
}
pub(super) fn float_candidate_test(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("cpu-float-v1");
    let original: serde_json::Value = c::read_json(&dir.join("baseline-reference.json"))?;
    if original["passed"] != true || original["original_source_and_core_unchanged"] != true {
        return Err("independent baseline arithmetic reference required".into());
    }
    let patch = repo.join("patches/xemu-f32-exact-flags.patch");
    let patch_hash = c::file_digest(&patch)?;
    if patch_hash != "937eed133bd896b5b4a3949c269027dd4357b4b031825a08697e5d0678af7c27" {
        return Err("bounded exact-flags patch differs".into());
    }
    let source = dir.join("source");
    let subject = source.join("fpu/softfloat.c");
    let build = dir.join("build");
    let prepared = dir.join("candidate-prepared.json");
    if !prepared.exists() {
        if original["subject_source"] != c::file_digest(&subject)?
            || original["reference_program"] != c::file_digest(&build.join("tests/fp/fp-test"))?
        {
            return Err("baseline reference changed before candidate preparation".into());
        }
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(build.join("tests/fp/fp-test"))
                .arg(dir.join("fp-test-baseline")),
        )?;
        c::store::checked(
            Command::new("git")
                .arg("-C")
                .arg(&source)
                .env("GIT_CEILING_DIRECTORIES", dir.canonicalize()?)
                .args(["apply", "--check"])
                .arg(&patch),
        )?;
        c::store::checked(
            Command::new("git")
                .arg("-C")
                .arg(&source)
                .env("GIT_CEILING_DIRECTORIES", dir.canonicalize()?)
                .arg("apply")
                .arg(&patch),
        )?;
        c::write_new(
            &prepared,
            &c::json(
                &json!({"patch":patch_hash,"before":original["subject_source"],"after":c::file_digest(&subject)?,"baseline_reference_program":original["reference_program"],"compiler_hardening_unchanged":true}),
            )?,
        )?;
    }
    let p: serde_json::Value = c::read_json(&prepared)?;
    if p["patch"] != patch_hash
        || p["after"] != c::file_digest(&subject)?
        || p["baseline_reference_program"] != c::file_digest(&dir.join("fp-test-baseline"))?
    {
        return Err("preserve and reconcile partial exact-flags candidate".into());
    }
    if dir.join("candidate-reference.json").exists() {
        return Err("candidate reference already attempted; consume actual receipt".into());
    }
    let dry = Command::new("ninja")
        .current_dir(&build)
        .args(["-n", "tests/fp/fp-test"])
        .env("NINJA_STATUS", "[%f/%t] ")
        .output()?;
    let plan = String::from_utf8(dry.stdout)?;
    let steps = plan.lines().filter(|v| v.starts_with('[')).count();
    c::write_new(
        &dir.join(format!("candidate-reference-plan-{}.log", c::new_id()?)),
        plan.as_bytes(),
    )?;
    if !dry.status.success()
        || steps > 16
        || plan.contains("qemu-system-i386")
        || plan.to_ascii_lowercase().contains("regenerating build")
    {
        return Err("candidate reference exceeds targeted arithmetic units".into());
    }
    let log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(format!("candidate-reference-build-{}.log", c::new_id()?)))?;
    c::store::checked(
        Command::new("ninja")
            .current_dir(&build)
            .args(["-j1", "tests/fp/fp-test"])
            .stdout(log.try_clone()?)
            .stderr(log),
    )?;
    let mut results = Vec::new();
    for (name, args) in [
        (
            "all-rounding",
            vec![
                "-q", "-s", "-l", "1", "-r", "all", "f32_add", "f32_sub", "f32_mul",
            ],
        ),
        (
            "preserved-flags",
            vec![
                "-q", "-s", "-l", "1", "-r", "all", "-f", "viou", "f32_add", "f32_sub", "f32_mul",
            ],
        ),
        (
            "nearest-level2",
            vec![
                "-q", "-s", "-l", "2", "-r", "even", "f32_add", "f32_sub", "f32_mul",
            ],
        ),
    ] {
        let log = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join(format!("candidate-{name}.log")))?;
        let started = Instant::now();
        let status = Command::new("timeout")
            .args(["--signal=TERM", "--kill-after=5s", "120s"])
            .arg(build.join("tests/fp/fp-test"))
            .args(args)
            .stdout(log.try_clone()?)
            .stderr(log)
            .status()?;
        results.push(json!({"case":name,"test_exit":status.code(),"passed":status.success(),"elapsed_ms":started.elapsed().as_millis()}));
        if !status.success() {
            break;
        }
    }
    let old = root.join("apu-quiescence-v1/corrected-runtime-v1");
    let preserved = original["subject_source"]
        == c::file_digest(&old.join("source/fpu/softfloat.c"))?
        && c::file_digest(&old.join("build/qemu-system-i386"))? == resources::APU_QUIESCENT_CORE;
    let pass = results.len() == 3 && results.iter().all(|v| v["passed"] == true) && preserved;
    c::write_new(
        &dir.join("candidate-reference.json"),
        &c::json(
            &json!({"patch":patch_hash,"subject_source":p["after"],"reference_program":c::file_digest(&build.join("tests/fp/fp-test"))?,"cases":results,"passed":pass,"retained_core_and_source_unchanged":preserved,"compiler_hardening_unchanged":true,"native_candidate_built":false,"performance_or_game_acceptance":false}),
        )?,
    )?;
    if !pass {
        return Err(
            "candidate arithmetic reference incomplete/failed; preserve all evidence".into(),
        );
    }
    Ok(())
}
/// Build only the retained independent arithmetic reference in a copied graph.
/// The accepted native core/source and its hardening options remain immutable.
pub(super) fn float_reference(repo: &Path, root: &Path) -> Result<(), Error> {
    let benchmark: serde_json::Value =
        c::read_json(&root.join("cpu-performance-v1/baseline/result.json"))?;
    if benchmark["guest_passed"] != true
        || benchmark["clean"] != true
        || benchmark["core"] != resources::APU_QUIESCENT_CORE
    {
        return Err("original CPU workload and clean return required".into());
    }
    let base = root.join("apu-quiescence-v1/corrected-runtime-v1");
    if c::file_digest(&base.join("build/qemu-system-i386"))? != resources::APU_QUIESCENT_CORE {
        return Err("retained qualified native differs".into());
    }
    let subject = "fpu/softfloat.c";
    let old = c::file_digest(&base.join("source").join(subject))?;
    let dir = root.join("cpu-float-v1");
    if !dir.exists() {
        c::mkdir(&dir)?;
        c::store::checked(
            Command::new("cp")
                .args(["-a", "--reflink=always"])
                .arg(base.join("source"))
                .arg(base.join("build"))
                .arg(&dir),
        )?;
        let graph = fs::read_to_string(dir.join("build/build.ninja"))?;
        if graph.contains("build build.ninja: REGENERATE_BUILD ")
            || graph
                .matches("build xbx-preserved-regeneration-edge: REGENERATE_BUILD ")
                .count()
                != 1
        {
            return Err("copied graph regeneration containment differs".into());
        }
        c::write_new(
            &dir.join("inputs.json"),
            &c::json(
                &json!({"base_core":resources::APU_QUIESCENT_CORE,"subject_source":old,"graph":c::file_digest(&dir.join("build/build.ninja"))?,"source_and_objects_copied_with_reflinks":true,"compiler_jobs":1,"new_dependencies_or_downloads":false,"peak_additional_allocation_bound":2147483648u64}),
            )?,
        )?;
    }
    let inputs: serde_json::Value = c::read_json(&dir.join("inputs.json"))?;
    if inputs["base_core"] != resources::APU_QUIESCENT_CORE
        || inputs["subject_source"] != old
        || c::file_digest(&dir.join("source").join(subject))? != old
        || inputs["graph"] != c::file_digest(&dir.join("build/build.ninja"))?
    {
        return Err("preserve/reconcile partial arithmetic reference preparation".into());
    }
    if dir.join("baseline-reference.json").exists() {
        return Err("original arithmetic reference already complete; reuse its receipt".into());
    }
    let build = dir.join("build");
    let dry = Command::new("ninja")
        .current_dir(&build)
        .args(["-n", "tests/fp/fp-test"])
        .env("NINJA_STATUS", "[%f/%t] ")
        .output()?;
    let mut planned = dry.stdout.clone();
    planned.extend_from_slice(&dry.stderr);
    c::write_new(
        &dir.join(format!("reference-plan-{}.log", c::new_id()?)),
        &planned,
    )?;
    let text = String::from_utf8(dry.stdout)?;
    let count = text.lines().filter(|line| line.starts_with('[')).count();
    if !dry.status.success()
        || count > 2048
        || text.contains("qemu-system-i386")
        || text.to_ascii_lowercase().contains("regenerating build")
    {
        return Err("arithmetic reference exceeds bounded retained target".into());
    }
    let log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(format!("reference-build-{}.log", c::new_id()?)))?;
    c::store::checked(
        Command::new("ninja")
            .current_dir(&build)
            .args(["-j1", "tests/fp/fp-test"])
            .stdout(log.try_clone()?)
            .stderr(log),
    )?;
    let log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join("baseline-reference.log"))?;
    let started = Instant::now();
    let status = Command::new("timeout")
        .args(["--signal=TERM", "--kill-after=5s", "60s"])
        .arg(build.join("tests/fp/fp-test"))
        .args([
            "-q", "-s", "-l", "1", "-r", "all", "f32_add", "f32_sub", "f32_mul",
        ])
        .stdout(log.try_clone()?)
        .stderr(log)
        .status()?;
    let preserved = old == c::file_digest(&base.join("source").join(subject))?
        && c::file_digest(&base.join("build/qemu-system-i386"))? == resources::APU_QUIESCENT_CORE;
    c::write_new(
        &dir.join("baseline-reference.json"),
        &c::json(
            &json!({"reference":"retained QEMU fp-test against independent Berkeley TestFloat/SlowFloat source","operations":["f32_add","f32_sub","f32_mul"],"level":1,"rounding":"all","subject_source":old,"subject_core":resources::APU_QUIESCENT_CORE,"reference_program":c::file_digest(&build.join("tests/fp/fp-test"))?,"test_exit":status.code(),"passed":status.success(),"elapsed_ms":started.elapsed().as_millis(),"planned_build_steps":count,"original_source_and_core_unchanged":preserved,"compiler_hardening_options_unchanged":true,"repo":repo,"performance_or_commercial_acceptance":false}),
        )?,
    )?;
    if !status.success() || !preserved {
        return Err("original arithmetic reference or preservation failed".into());
    }
    Ok(())
}
fn put(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
pub(super) fn build(repo: &Path, root: &Path) -> Result<(), Error> {
    let dir = root.join("cpu-performance-v1");
    if dir.exists() {
        return Err("preserve existing CPU fixture preparation; reconcile before reuse".into());
    }
    c::mkdir(&dir)?;
    let template = root.join("fixtures-v2/a.iso");
    if c::file_digest(&template)? != resources::ORIGINAL_FIXTURES[0] {
        return Err("original fixture template differs".into());
    }
    let source = repo.join("native/fixtures/cpu-performance.S");
    let uart_source = repo.join("native/fixtures/startup-lifetime.S");
    let uart = fs::read_to_string(&uart_source)?;
    let uart = uart
        .split_once("serial_init:\n")
        .ok_or("original UART start")?
        .1
        .split_once(".balign 4\n")
        .ok_or("original UART end")?
        .0;
    let mut expected = 0u32;
    for i in 0..4_000_000 {
        expected = if i & 1 == 0 {
            expected.wrapping_add(3)
        } else {
            expected ^ 0x5a5a5a5a
        };
    }
    let generated = format!(
        ".set DISPATCH_EXPECTED, {expected}\n{}\n.text\nserial_init:\n{uart}",
        fs::read_to_string(&source)?
    );
    c::write_new(&dir.join("original.S"), generated.as_bytes())?;
    c::store::checked(
        Command::new("/usr/bin/clang")
            .args(["--target=i386-none-elf", "-march=pentium3", "-c"])
            .arg(dir.join("original.S"))
            .arg("-o")
            .arg(dir.join("original.o")),
    )?;
    c::store::checked(
        Command::new("/usr/bin/ld")
            .args([
                "-m",
                "elf_i386",
                "-Ttext=0x11000",
                "-Tdata=0x12000",
                "--entry=_start",
            ])
            .arg(dir.join("original.o"))
            .arg("-o")
            .arg(dir.join("original.elf")),
    )?;
    let symbols = Command::new("/usr/bin/nm")
        .args(["-n", "--defined-only"])
        .arg(dir.join("original.elf"))
        .output()?;
    if !symbols.status.success() {
        return Err("original symbol inventory failed".into());
    }
    let symbols = String::from_utf8(symbols.stdout)?;
    let thunk = symbols
        .lines()
        .find(|v| v.ends_with(" D thunks"))
        .and_then(|v| v.split_whitespace().next())
        .ok_or("original import terminator")?;
    let thunk = u32::from_str_radix(thunk, 16)?;
    if !(0x12000..0x13000).contains(&thunk) {
        return Err("original data page differs".into());
    }
    c::store::checked(
        Command::new("/usr/bin/objcopy")
            .args(["-O", "binary"])
            .arg(dir.join("original.elf"))
            .arg(dir.join("original.bin")),
    )?;
    let bytes = fs::read(dir.join("original.bin"))?;
    if bytes.len() <= 4096 || bytes.len() > 8192 {
        return Err("bounded original code/data extent".into());
    }
    let original = fs::read(template)?;
    let mut xbe = original[34 * 2048..34 * 2048 + 4096].to_vec();
    put(&mut xbe, 0x10c, 0x3000);
    put(&mut xbe, 0x11c, 2);
    put(&mut xbe, 0x158, thunk ^ 0x5b6d40b6);
    put(&mut xbe, 0x180, 0x58424350);
    xbe[0x184..0x1d4].fill(0);
    for (i, ch) in "xbx original CPU workload".encode_utf16().enumerate() {
        xbe[0x184 + i * 2..0x186 + i * 2].copy_from_slice(&ch.to_le_bytes());
    }
    xbe[0x408..0x40e].copy_from_slice(b".data\0");
    for (i, flags, va, raw, part) in [
        (0usize, 6, 0x11000, 4096, &bytes[..4096]),
        (1, 3, 0x12000, 8192, &bytes[4096..]),
    ] {
        let at = 0x348 + i * 56;
        xbe[at..at + 56].fill(0);
        for (off, v) in [
            (0, flags),
            (4, va),
            (8, 4096),
            (12, raw),
            (16, part.len() as u32),
            (20, 0x10400 + i as u32 * 8),
            (28, 0x10410 + i as u32 * 4),
            (32, 0x10412 + i as u32 * 4),
        ] {
            put(&mut xbe, at + off, v);
        }
        xbe[at + 36..at + 56].copy_from_slice(&super::build::section_digest(part)?);
    }
    xbe.extend_from_slice(&bytes);
    c::write_new(&dir.join("original.xbe"), &xbe)?;
    let mut iso = vec![0; 64 * 2048];
    iso[..34 * 2048].copy_from_slice(&original[..34 * 2048]);
    put(&mut iso, 33 * 2048 + 8, xbe.len() as u32);
    iso[34 * 2048..34 * 2048 + xbe.len()].copy_from_slice(&xbe);
    c::write_new(&dir.join("original.iso"), &iso)?;
    c::write_new(
        &dir.join("build.json"),
        &c::json(
            &json!({"source":c::file_digest(&source)?,"uart_source":c::file_digest(&uart_source)?,"fixture":c::digest(&iso),"iterations_per_phase":4_000_000,"dispatch_expected":expected,"separate_code_data_pages":true,"original_generated_content_only":true,"operations":"alternating indirect calls; exact packed SSE mul/add with inexact initially clear, then sticky","performance_not_game_acceptance":true}),
        )?,
    )?;
    Ok(())
}
pub(super) fn observe(
    repo: &Path,
    root: &Path,
    candidate: bool,
    recheck: bool,
) -> Result<(), Error> {
    let out = root.join("cpu-performance-v1").join(if candidate {
        "candidate"
    } else if recheck {
        "baseline-recheck"
    } else {
        "baseline"
    });
    observe_into(repo, root, candidate, &out, false)
}
fn observe_into(
    repo: &Path,
    root: &Path,
    candidate: bool,
    out: &Path,
    thread_runtime: bool,
) -> Result<(), Error> {
    let r = if candidate {
        resources::Resources::cpu_float(&root.join("cpu-float-v1/package"))?
    } else {
        resources::Resources::apu_quiescent(&root.join("apu-quiescence-v1/package"))?
    };
    observe_resources(repo, root, r, out, thread_runtime)
}
pub(super) fn observe_resources(
    repo: &Path,
    root: &Path,
    mut r: resources::Resources,
    out: &Path,
    thread_runtime: bool,
) -> Result<(), Error> {
    let dir = root.join("cpu-performance-v1");
    let built: serde_json::Value = c::read_json(&dir.join("build.json"))?;
    let media = dir.join("original.iso");
    if built["source"] != c::file_digest(&repo.join("native/fixtures/cpu-performance.S"))?
        || built["fixture"] != c::file_digest(&media)?
    {
        return Err("original workload identity differs".into());
    }
    c::mkdir(out)?;
    let core = r.manifest.core.clone();
    r.window = false;
    let app = out.join("app");
    let mut player = c::desktop::Player::with_resources(&app, r)?;
    let game = player.add(&media)?;
    let entry = c::library::Library::read(&app)?.resolve(&game)?;
    let disk = repo.join(".local/dashboard-v20260516-0955/xbox_hdd.qcow2");
    let eeprom = repo.join(".local/quick-reboot-d87cfc33-run1/eeprom.bin");
    let seeds = (c::file_digest(&disk)?, c::file_digest(&eeprom)?);
    if seeds.0 != "00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd" {
        return Err("original seed differs".into());
    }
    {
        let _p = c::store::Profile::open_seeded(&app, &entry, &player.resources, &disk, &eeprom)?;
    }
    player.launch(&game)?;
    let run = player.live.as_ref().ok_or("live")?.native.run.clone();
    let start = Instant::now();
    let mut text = String::new();
    let mut events = Vec::new();
    let mut log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(out.join("serial.log"))?;
    let markers = [
        "DISPATCH BEGIN",
        "DISPATCH DONE",
        "SSE EXACT BEGIN",
        "SSE EXACT DONE",
        "SSE STICKY BEGIN",
        "SSE STICKY DONE",
        "PERF PASS",
    ];
    let observed = (|| -> Result<bool, Error> {
        let tid = if thread_runtime {
            let cpus = player
                .live
                .as_mut()
                .ok_or("live")?
                .native
                .call("query-cpus-fast", json!({}))?;
            let cpus = cpus.as_array().ok_or("native CPU list")?;
            if cpus.len() != 1 {
                return Err("single original-workload CPU required".into());
            }
            Some(cpus[0]["thread-id"].as_u64().ok_or("native vCPU thread")?)
        } else {
            None
        };
        loop {
            let s = player.live.as_mut().ok_or("live")?.native.serial()?;
            log.write_all(s.as_bytes())?;
            text.push_str(&s);
            if text.len() > 65536 {
                return Err("original output bound".into());
            }
            while events.len() < markers.len()
                && text.contains(&format!("XBX CPU {}", markers[events.len()]))
            {
                let runtime = tid
                    .map(|tid| -> Result<u64, Error> {
                        Ok(fs::read_to_string(format!("/proc/{tid}/schedstat"))?
                            .split_whitespace()
                            .next()
                            .ok_or("thread runtime")?
                            .parse()?)
                    })
                    .transpose()?;
                events.push(json!({"marker":markers[events.len()],"host_received_ms":start.elapsed().as_millis(),"vcpu_runtime_ns":runtime}));
            }
            if text.contains("XBX CPU PERF FAIL") || text.contains("*** bug check") {
                return Ok(false);
            }
            if events.len() == markers.len() {
                return Ok(true);
            }
            if start.elapsed() > Duration::from_secs(120) {
                return Err("original CPU workload deadline".into());
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    })();
    let returned = player.stop();
    log.sync_all()?;
    let native: serde_json::Value = c::read_json(&run.join("native-return.json"))?;
    let unchanged = seeds == (c::file_digest(&disk)?, c::file_digest(&eeprom)?);
    let passed = observed.as_ref().is_ok_and(|v| *v);
    let clean = returned.is_ok() && native["clean"] == true && unchanged;
    c::write_new(
        &out.join("result.json"),
        &c::json(
            &json!({"core":core,"open_system":resources::LOCAL_BASE_OPEN,"fixture":built["fixture"],"events":events,"guest_passed":passed,"native_return":native,"clean":clean,"seeds_unchanged":unchanged,"observation_error":observed.err().map(|e|e.to_string()),"return_error":returned.err().map(|e|e.to_string()),"timing_limit":"host receive timestamps, approximately 10ms serial polling; require repeatability before any speedup claim","commercial_acceptance":false}),
        )?,
    )?;
    if !clean || !passed {
        return Err("original CPU workload or native return failed; preserve evidence".into());
    }
    Ok(())
}
