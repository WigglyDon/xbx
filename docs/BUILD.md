# Qualification builds

This builds a diagnostic candidate, not the finished player or an approved
binary release. The active issue owns progress and acceptance evidence.
`source-inputs.lock.json` and `patches/xemu-android.patch` bind the engine graph.
First-party Rust has no external Cargo dependencies; DSP dependencies use its
pinned upstream Cargo.lock with `--locked`.

## Host preparation

Use existing read-only tool installations. The measured pass used Linux x86_64,
Rust 1.96.1, CMake 4.3.0, Ninja 1.13.2, Meson 1.9.2, NDK 30.0.16248370,
Android platform/build tools 37.0 and OpenJDK 25.0.4.1. Target API is 36; native
minimum API 29 follows the port's ELF TLS requirement. These are build inputs,
not Fold observations. No SDK directory is modified by xtask.

```sh
python3 -m venv .local/tools
.local/tools/bin/pip install meson==1.9.2 PyYAML==6.0.3
export XBX_NDK=/path/to/read-only/ndk/30.0.16248370
export XBX_SDK=/path/to/read-only/android-sdk
cargo xtask capacity 6
cargo xtask source-prepare
cargo xtask source-check
cargo xtask android-configure
cargo xtask android-build
cargo xtask android-host
cargo xtask kernel-build play
cargo xtask profile-assets
cargo xtask android-apk
```

`xtask` checks free space and Btrfs metadata/allocation/global-reserve accounting,
retains 12 GiB plus estimated growth and at least 1 GiB usable metadata when
device-unallocated space is low, and holds one xbx build-writer lease. It does
not reserve space against other projects, qualify performance, repair a
filesystem, install packages or delete a cache. Inspect host jobs before growth;
the native/DSP builds use one compiler job. The first 6 GiB peak estimate includes
source/dependencies/objects/evidence and must be re-evaluated for media conversion.
One interrupted compilation is resumed in the same build directory.

The output is `.local/xbx-qualification.apk`, package `com.machinerhythm.xbx`,
activity process `:engine`, ARM64 only.
This local diagnostic manifest is debuggable so an authorized device transaction
can prepare only xbx's own scratch profile through `run-as`; it is not a release
manifest or authority to access any other app's data. Signing is generated under
protected `.local/signing`; preserve that directory and password file. The build
does not access PSX signing/data. It does not install an app, access a phone or
publish an APK. Verify a fresh device operating window before any phone action.

The current APK is a bounded one-game player. It seeds a new private
`xbx-qualification` profile from the separately built normal-play OXOS image
(`SELFTEST=0 SERIAL=1`) and a generated 8 GiB FATX disk. C/E/X/Y/Z are formatted;
E contains empty TDATA/UDATA directories. The qcow2 seed occupies about 1.2 MiB,
not 8 GiB. No game, owner save, EEPROM, MCPX or comparison BIOS is bundled.
This is a private qualification firmware choice, not a cleared shipping default.

A stock scoped-document picker grants read access to the existing Fable copy.
Rust retains that seekable read-only FD, inspects XDVDFS/XBE on the same object,
checks a full SHA-256 through the existing GLib implementation, and supplies a
capability duplicate to QEMU's FD set. No provider path is reopened and no disc
copy/conversion is required. Nonseekable, unstable-size or writable provider
objects refuse before machine launch. A persisted URI can reopen the same game;
a different media fingerprint refuses while preserving its prior profile.
Random durable GameId/profile IDs and SavedSession are separate from LiveRuntime's
File/FD. This first player always cold-boots current storage and never restores RAM.

Native ABI v3 separates machine acceptance, display readiness and termination.
The core starts with `-S`; Rust resumes only after successful initialization,
display readiness, foreground and audio focus. A rejected EEPROM is terminal,
with no later foreground override. Missing EEPROM creation is allowed only for
an intentional first profile generation; a missing identity in a used profile
refuses. Firmware/profile identity is checked before writable native entry.
Rust advances the fsynced dirty epoch before QEMU writes and holds the lease
until process death. Startup has a 60-second budget plus five seconds of stop
cleanup; exit draining has a ten-second watchdog. Forced termination remains an
unclean result with retained data/epoch, never an accepted checkpoint.

Rust owns simultaneous touch capture, radial sticks/dead zones, analog trigger
channels, all Xbox buttons including Black/White and stick clicks, D-pad and
Start/Back. Face-button touch uses full digital pressure; triggers begin fully
pressed and downward dragging reduces pressure. Cancellation/focus loss/resize
neutralizes all channels. Java only adapts document, Canvas, lifecycle and audio
focus APIs. A reserved control band keeps the 4:3 surface clear of the controls.
Actual Fold touch, audio and surface transitions remain unqualified until run.

Private native and kernel stdout/stderr are drained through a capped 4 MiB log
with periodic fsync; the cap never blocks native output. Guest flip-counter
samples are counts per wall interval, not an emulation-speed certification or
a universal game-FPS target. The player provides no automatic full-machine
restore, collection editor, shortcuts or multiple resident engines.

For a full-disc XDVDFS layout, the engine uses QEMU's existing read-only raw
block view at the validated partition offset. The source image is not repacked
or patched. Native media-view fixtures establish offset/bounds/read-only behavior;
the supplied Fable image passes the bounded filesystem/executable/certificate
inspection and identifies itself as Fable. Android disc execution remains untested.
Certificate metadata does not certify edition, authenticity or every payload byte.
Byte offset belongs
to future checkpoint media identity, alongside the source image fingerprint.

## Boundary checks

```sh
cargo test --workspace --offline -j 1
cargo test -p xbx-host --offline -j 1 --test boundaries qcow2_current -- --ignored --nocapture
cargo test -p xbx-host --offline -j 1 --test boundaries raw_disc_partition -- --ignored --nocapture
cargo run -p xbx-host --offline -- inspect-disc /authorized/complete/game.iso
```

The explicit qcow2 experiment requires host `qemu-img` and `qemu-io` (tested
10.2.2). It uses only synthetic 16 MiB disks and retains one named result under
ignored `target/scratch`. Reopened newer bytes, a refused older generation and a
deliberate rollback copy are **block-storage** evidence, not in-game saving or
full-machine resume. Other checks cover cross-process writer exclusion,
inherited ownership after supervisor loss, crash-release, interrupted manifest
publication and bounded XDVDFS/XBE parsing. The parser ignores filename suffixes;
it checks certificate bounds and reads title/ID/region/version without reading
certificate keys. It does not certify all disc files, game compatibility, edition or completeness
of an ongoing download. Do not inspect an owner's partial download as a ready game.

## Native contract and remaining qualification

The retained source owns TCG x86-to-AArch64 execution and core/render threads;
SDL3 supplies platform windows/controllers/audio, and the initial GPU route is
system Vulkan. `native/xbx_xemu_bridge.c` exposes configuration, QEMU entry,
display readiness/loop, thread completion and pause/resume requests. Rust owns
profile/launch/lifetime policy. A pause request is not a quiescence acknowledgment.
Native display-ready and shutdown paths still need actual Android failure and
surface-transition testing. Source review does not prove 16 KiB JIT/runtime safety.

The patch preserves incompatible EEPROMs rather than regenerating/repairing them,
binds caches/previews to the selected profile, uses atomic cross-thread control
flags and services snapshot requests even while the display is hidden/paused.
The retained paused-device-state capture path remains in place. Snapshot I/O,
durable completion/cancellation and coherent firmware/media/EEPROM/disk manifests
are not qualified; the Rust host does not expose autosave or the donor menu's
blocking snapshot dispatcher. Touch control source/tests exist; actual controls, the collection, shortcuts,
Fable gameplay/audio, ordinary saves, full resume and device performance remain unearned.

For an optional desktop CPU diagnostic, build the same source and DSP using
`cargo xtask desktop-dsp`, `desktop-configure`, `desktop-build`, then run
`cargo run -p xbx-host --offline -- boot-probe /path/to/source-built/xemu /new/short/scratch`.
The desktop configure uses task-local pkg-config headers under
`.local/host-sysroot` when system development headers are unavailable; no package
installation is performed. The probe generates an original 1 MiB reset ROM,
checks two guest CPU register markers through QMP and waits for quit acknowledgment
and native process exit. The original synthetic ROM contains only reset-vector
and register-marker instructions. Its inherited profile
lease survives supervisor loss. It contains no game, MCPX dump or Xbox kernel and
does not establish gameplay, GPU presentation, sound, saves or Fold acceptance.
The measured source-built desktop pass observed both markers and normal QMP
shutdown with a software OpenGL context. Earlier failed probes are preserved:
SDL3 requires `SDL_VIDEO_DRIVER`/`SDL_AUDIO_DRIVER`, the default sandbox refused
the local Unix socket, the Xbox target omits generic `isa-debugcon`, and closing
the monitor transport before quit acknowledgment did not complete shutdown.
These observations do not establish a core shutdown defect or an Android result.

For bounded open-image qualification, sparse-checkout only `kernel`, `tools` and
`tests/kernel-test-title` of OXOS `c0c887a1263ebf89ba765f34a88ca8f132901e25` under
`.local/src/oxos`; keep the specification corpus out of the working tree. Its
build requires Cromwell `86f547387184d1001d377cce97d3756c8acf91cc` at
`.local/src/oxos/third_party/cromwell`. `cargo xtask kernel-build` uses the existing
NDK's LLVM tools and the retained Make/Python pipeline with `IMAGE=open`, no
comparison BIOS/MCPX input, no splash asset and one compile job. Without `play` it builds only
the self-test image; no dashboard/game download, hardware flashing or firmware
default selection is performed. Output stays under `.local/build-oxos`.
The newer LLVM diagnostic for the donor's intentional equal-width i386 timer
APC function cast is retained as a warning with
`-Wno-error=cast-function-type-mismatch`; other donor `-Werror` checks remain.
This build adjustment does not qualify that ABI at runtime.
`cargo xtask kernel-build play` uses a separate `.local/build-oxos-play`
directory with self-tests disabled. `cargo xtask profile-assets` creates a
versioned private seed and selects its three-file asset inventory for packaging.
Retain the original self-test artifact and every seed; do not overwrite a used
profile with a new firmware build. Kernel date/time macros affect image identity,
so retain the actual built image/digest as well as its source/command.

The bounded desktop self-test attempt used the retained core, 64 MiB and a new
formatted disk, with no MCPX/game/dashboard. It observed KiBootStage=0x30,
KiBugCheckCode=0, KiTestMmDone=1, KiTestFatxDone=1, KiTestFatxWriteOk=1, followed
by acknowledged QMP shutdown. This is a selected boot/storage mechanism result,
not the complete upstream self-test suite or commercial-game compatibility.

```sh
cargo run -p xbx-host --offline -- empty-disk /new/private/empty.raw
qemu-img convert -f raw -O qcow2 -S 4096 /new/private/empty.raw /new/private/hdd.qcow2
cargo run -p xbx-host --offline -- boot-kernel .local/build-desktop/qemu-system-i386 /new/short/profile .local/build-oxos/bios.bin /new/private/hdd.qcow2 .local/build-oxos/xboxkrnl.map
XBX_TEST_ENGINE=/absolute/source-built/qemu-system-i386 cargo test -p xbx-host --offline -j 1 --test boundaries native_rejection -- --ignored --nocapture
```

FATX format facts were checked with the retained independent Python reader. The
refusal fixture verifies native exit code 1 before guest execution and unchanged
synthetic EEPROM/disk bytes; Rust tests separately verify rejected->display->
foreground cannot resume. A failed diagnostic attempt used QEMU's error_report
before monitor initialization and asserted; plain stderr corrected that issue.
Owned desktop test children now disable core dumps and retain bounded logs.

All private artifacts, interrupted logs and synthetic experiment data remain
local. No owner media, EEPROM/key material or signing credential belongs in a
source bundle. License review and a complete corresponding-source/notices bundle
precede any public binary; see [THIRD_PARTY.md](../THIRD_PARTY.md).
