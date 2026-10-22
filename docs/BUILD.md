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

The qualification APK requires a deliberately prepared private
`xbx-qualification` profile containing `flash.bin`, `hdd.qcow2` and `disc.xiso`.
It is not a file-picker/player UX. Firmware is not supplied or qualified by this
recipe. Do not copy owner saves into this scratch profile. Rust reads the disc
structurally, takes the writer lease before final validation, generates fixed
profile paths, supplies validated disc partition offsets through native ABI v2,
fsyncs a dirty epoch before QEMU writes, and holds ownership until
process exit. Automatic snapshot restore is disabled. The 128 MiB diagnostic
configuration is not a qualified Fable profile.

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
blocking snapshot dispatcher. Touch controls, the collection, shortcuts, Fable
gameplay/audio, ordinary saves, full resume and device performance remain unearned.

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
comparison BIOS/MCPX input, no splash asset and one compile job. It builds only
the self-test image; no dashboard/game download, hardware flashing or firmware
default selection is performed. Output stays under `.local/build-oxos`.
The newer LLVM diagnostic for the donor's intentional equal-width i386 timer
APC function cast is retained as a warning with
`-Wno-error=cast-function-type-mismatch`; other donor `-Werror` checks remain.
This build adjustment does not qualify that ABI at runtime.
The pinned open image now source-builds successfully, without proprietary
comparison inputs; kernel self-test execution and Fable compatibility remain
unqualified.

All private artifacts, interrupted logs and synthetic experiment data remain
local. No owner media, EEPROM/key material or signing credential belongs in a
source bundle. License review and a complete corresponding-source/notices bundle
precede any public binary; see [THIRD_PARTY.md](../THIRD_PARTY.md).
