# Source provenance

The bounded contract observer modifies the retained GPL OXOS import tracer and
annotates its existing diagnostic point. `patches/oxos-observer.patch` and
`kernel-observer.lock.json` preserve the full accepted base composition and pins.
Original `crates/host/src/observer.rs` policy and `native/fixtures/observer.S`
mechanical ABI fixture are MIT; the latter reuses this repository's original
null-call fixture's public CONTEXT layout. No game code is included. See
[OBSERVER.md](docs/OBSERVER.md) for scope, schema, lineage and actual qualification.

`native/fixtures/thread-startup.S` and `npx-ownership.S` are original MIT
synthetic guest ABI/register-state checks using pinned public interface facts.
The targeted retained OXOS FPU ownership correction in
`patches/oxos-npx-owner.patch` retains GPL/per-file terms; its full composition
and unchanged bases are bound by `kernel-npx.lock.json`. Original Rust owns
build/acceptance policy. [THREAD_FPU.md](docs/THREAD_FPU.md) records the actual
before/after proof and unsuccessful ordinary-game result, without game code.

`patches/oxos-npx-owner-functional.patch` carries only that accepted seven-line
repair from xbx `fcbaeab8f544f6ab4b78c027c032ba429549bd49` onto the retained
OXOS `9d8cd58ca19f49f5ecb4b7b55bfa194f4e4a5a96` source. GPL/per-file terms
remain; no historical instrumentation is included. The original MIT regression
`npx-ownership.S` is reused with the original `npx-uart.S` output adapter and
the public UART activation already used by `startup-lifetime.S`. Rust owns
source verification, finite build and actual guest-result/return acceptance.

The separate upstream desktop comparison retains public xemu at
03b988c9d76553b33ed712ba00822ffdf49f0e7d. Its complete targeted integration diff
is `patches/xemu-reference-desktop.patch`, bound by `reference-core.lock.json`.
The character-watch ownership, supervised process exit, Epoxy dependency and
source-built DSP integration retain QEMU/xemu GPL and applicable per-file terms.
No emulated CPU/hardware behavior is changed by that patch. Existing dependency
pins and notices remain. Original Rust build policy is MIT. See
[the exact build/result receipt](docs/REFERENCE_CORE.md); this comparison does
not select a new Android core or authorize a public binary release.

The separate RAMHT candidate adds `patches/xemu-reference-ramht.patch`, bound by
`ramht-core.lock.json`. Its PFIFO arithmetic correction retains that source's
LGPL-2.0-or-later terms and upstream copyrights. The exact retained function and
register header are used only through a mechanical test ABI; original Rust
contract fixtures and build/admission policy are MIT. The public Nouveau driver
is a cited behavioral reference, not an imported runtime dependency. See
[the bounded contract and qualification](docs/RAMHT.md).

The Android debugger metadata restoration reuses `scripts/feature_to_c.py`
(GPL-2.0-or-later) and `gdb-xml/i386-32bit.xml` from the retained public
xemu Android pin `4967efd3ee594cc236869a45ee4cb396e25af1ac`. The XML retains
the Free Software Foundation's 2010–2017 copyright and its stated permission
to redistribute with its notices preserved. The generated table replaces only
the fork's empty `gdb_static_features` object; all other archive objects remain
byte-identical. Original Rust build/audit policy and the metadata contract in
`native/fixtures/gdb-feature-table.c` are MIT. No guest execution code or game
data is copied by this correction.

## Seed inventory

The initial foundation at `59fbcfa` contained documentation/bootstrap only.
The qualification implementation now builds the candidate below from pinned
public source in ignored local storage. No game, proprietary firmware, EEPROM,
save, signing key or font/logo asset is incorporated here. Selected public PSX
Rust layout geometry is reused below with its MIT notice.

Original contributions use the root MIT license. That grant does not cover
software added from elsewhere. A future combined distribution must satisfy the
exact licenses of its core, dependencies, assets, firmware and copied source.

`native/fixtures/nullcall.S` is an original MIT mechanical i386 exception-ABI test
shim, with Rust build/runner/acceptance policy. It contains no game or donor code.
Xbox CONTEXT/registration offsets use interface facts from the pinned kernel.
The pinned OXOS `mkxbe.py` and `mkxiso.py` tools are invoked to wrap the generated
fixture, retaining their own terms; their implementations are not copied here.

`native/fixtures/vm-imports.S` is also an original MIT mechanical i386 ABI shim.
It uses only public interface facts from OXOS `kernel/include/xk/mm.h` and
`kernel/exports.tsv` at the historical `c0c887a1263ebf89ba765f34a88ca8f132901e25` pin: allocation/free ordinals, stdcall argument
sizes, flags and types. No allocator implementation or title code is copied.
The same pinned wrapping tools are invoked under their existing terms. Rust owns
the build and acceptance policy; see [VM_IMPORT_CONTRACT.md](docs/VM_IMPORT_CONTRACT.md).

## Qualification build lineage

The selected **qualification candidate**, not a shipping compatibility claim,
is `izzy2lost/xemu` at `4967efd3ee594cc236869a45ee4cb396e25af1ac`, compared to
`xemu-project/xemu` at `03b988c9d76553b33ed712ba00822ffdf49f0e7d`.
[source-inputs.lock.json](source-inputs.lock.json) records the actual recursive
Git/archive bindings. Its DSP Cargo.lock hash binds registry dependency checksums
at that source revision. Build commands and limits are in [BUILD.md](docs/BUILD.md).

- `patches/xemu-android.patch` modifies retained GPL-lineage source and build
  files: deterministic dependencies, profile paths, EEPROM preservation,
  cross-thread control flags, hidden snapshot servicing, narrow Android serial
  diagnostic registration retention and source-built desktop
  DSP input. Individual upstream terms remain applicable.
  The candidate also removes the arcade path's built-in Chihiro EEPROM fallback
  include/data use. That optional donor path now requires explicit external
  inputs. No EEPROM dump bytes are included in this patch. The initial xbx
  profile never enables Chihiro.
- `native/xbx_xemu_bridge.c` adapts the mechanical init/display/thread entry from
  the pinned `android/app/src/main/cpp/xemu_android.cpp`; GPL-2.0-or-later, with
  [GPL text](LICENSES/GPL-2.0.txt). This is not covered by the root MIT grant.
- The APK compiles SDL's platform Java sources from the exact SDL 3.4.14 archive,
  `android-project/app/src/main/java/org/libsdl/app/`, under SDL's zlib license.
  The small xbx Java subclass is original mechanical platform glue, MIT.
- Original Rust modules are MIT. Landscape side-column geometry and
  `clear_picture` in `crates/host/src/input/layout.rs` adapt
  `crates/player/src/input.rs` (`Layout::ps2`, `Rect`, `clear_picture`) from public
  [PSX at 1e4ba86](https://github.com/WigglyDon/psx/blob/1e4ba86fa0a5fc1470837b6bcc06bd20b6642c11/crates/player/src/input.rs).
  Copyright (c) 2026 Machine Rhythm contributors; retained
  [MIT notice](LICENSES/PSX-MIT.txt). Xbox input mappings, colors, storage and
  runtime remain xbx-owned; no PSX private source or application data was used.
  XDVDFS offsets/layout
  were cross-checked against the pinned fork's MIT `android/third_party/xdvdfs-core`
  by antangelo, specifically `blockdev/offset.rs` and `layout/`; no parser code
  was copied. The synthetic reset ROM contains original diagnostic instructions.
  XBE header/certificate field offsets were checked against Cxbx-Reloaded's
  [Xbe.h at f1e54be](https://github.com/Cxbx-Reloaded/Cxbx-Reloaded/blob/f1e54be515bb7989bf5109ebea2494b6b13eb1bd/src/common/xbe/Xbe.h).
  Only format facts are used; no Cxbx implementation or declarations were copied.
  The original Rust FATX formatter uses layout facts cross-checked with pinned
  OXOS `tests/fatx.py` and independently read back by that reader; no formatter
  or filesystem implementation was translated/copied. SHA-256 uses retained
  GLib through a mechanical adapter, not a new cryptographic implementation.

The actual native graph includes QEMU/xemu (root GPLv2 terms and per-file
exceptions), SDL3 (zlib and component notices), toml++ (MIT), libslirp (BSD-style
and per-file terms), nv2a_vsh_cpu (MIT), adrenotools/linkernsbypass (MIT), volk
(MIT), VulkanMemoryAllocator (MIT), glslang (BSD/MIT/Apache and per-file terms),
SPIRV-Reflect (Apache-2.0), DSP56300 (MIT plus its locked Rust dependencies),
GLib 2.66.8 (LGPL and component terms), libffi (MIT-style), and proxy-libintl
(GNU Library GPL v2 text and source-specific terms). Toolchain/runtime notices
also apply. This inventory does not substitute for the complete file-level
notices/corresponding-source bundle required for a public binary.

**Distribution review remains open:** SPIRV-Reflect's Apache-2.0 source is linked
alongside the donor's GPLv2 combined-work terms; the selected file-level license
compatibility and resulting distribution obligations require explicit review.
An upstream build is not clearance. No public binary is released by this work.

`patches/oxos-launch-trace.patch` narrowly instruments retained OXOS startup,
loader, VM calls, exception dispatch and firmware-return code at the pin in
`kernel-trace.lock.json`; it keeps
OXOS's actual GPL/per-file terms. The existing kernel-test-title and XDVDFS
fixture generator are built/invoked from that source without copying them into
original MIT modules. First-party Rust diagnostic branching, runner and geometry
policy are original MIT work. See docs/LAUNCH_DIAGNOSTIC.md for reproduction.

The targeted `chardev/char-io.c` watch-reference repair and its header comment
remain modifications of retained QEMU source, preserving their existing file
notices and combined-work obligations. No emulator source was translated into
an original Rust module for this repair.

`patches/oxos-loader-complete.patch` is a complete d87-based composition of the
unchanged v4 trace patch plus a scoped `kernel/src/xe/loader.cpp` correction:
required read completion, guarded header access, shared-section failure cleanup
and bounded pre-import integrity counts. It retains OXOS's GPL/per-file terms;
`kernel-loader.lock.json` binds both the composed and original trace identities.
It is applied to clean d87, not on top of an already-patched v4 tree.

`native/fixtures/loader-complete.S` and `xtask/src/loader_fixture.rs` are original
MIT fixture code/build policy. XBE field layouts, encoded entry/thunk constants
and section-reference interfaces are public format facts checked against the
pinned OXOS declarations/wrapping tool, not a copied loader implementation.
The original fixed layout deliberately shares boundary pages. SHA-1 calculation
invokes the existing host `sha1sum`; XDVDFS wrapping invokes the unchanged pinned
GPL/per-file `tests/mkxiso.py` without copying its implementation. Fixture bytes
are source-authored synthetic data, not game-derived content. See
[LOADER_COMPLETENESS.md](docs/LOADER_COMPLETENESS.md).

## Boot qualification and distribution boundary

- xemu derives from QEMU. Its LICENSE describes GPL version 2 overall, with
  other terms in individual components. Do not assume GPL-2.0-or-later for every
  file merely because a standard GPL text contains example wording.
- X1 BOX (`izzy2lost/xemu`) is an Android fork of xemu; its README retains the
  xemu/QEMU licensing lineage. Trace Android-specific changes and dependencies
  separately rather than treating an APK as corresponding source.
- Roswell (`XboxDev/roswell`) is a ReactOS-derived open Xbox kernel. Its README
  describes a GPL-2.0 combined work with per-file terms.
- OXOS (`kooscode/oxos`) advertises GPL-2.0 and an original boot/kernel path. Its
  source/provenance and comparison-image paths require review for distribution.
  Desktop qualification now builds its open route at
  `d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2` with Cromwell
  `86f547387184d1001d377cce97d3756c8acf91cc`; it uses neither a comparison BIOS
  nor MCPX dump. This adopts upstream's one-file quick-reboot mapping correction
  with its original GPL/per-file terms, not a duplicate first-party backport.
  The private Android player retains its existing images from
  `c0c887a1263ebf89ba765f34a88ca8f132901e25`; no new Android mode or shipping
  default is selected. See [QUICK_REBOOT.md](docs/QUICK_REBOOT.md).
- PSX is a related product, not an xbx dependency. Any selected public MIT host
  source may be reused with its original copyright notice. Do not copy its
  GPL cores, fonts or firmware and label the resulting tree MIT-only.

Observed revisions and source URLs are in `upstream-candidates.json` and
[docs/SOURCES.md](docs/SOURCES.md). Neither open-kernel candidate is selected as
a distributable default. OXOS documents non-stock save-signature keys and a
separate proprietary comparison-image route; neither may be silently substituted
beneath an existing profile. Roswell's 128 MiB configuration is unqualified here.

## Before incorporation

The separately source-built Roswell candidate is pinned to
`XboxDev/roswell` `1569e2e89fb47cc72b9c704a8884f98200432bd4`; exact source,
tools, release mode and artifact binding are in `roswell.lock.json` and
[ROSWELL.md](docs/ROSWELL.md). It is ReactOS-derived, GPL-2.0 as a combined
kernel, with per-file terms including GPL-2.0-only video drivers and the
0BSD XZ Embedded decoder. Its complete original source/notices stay retained;
root GPL does not replace those component notices. Original xbx Rust build and
qualification policy remains MIT. No kernel implementation was copied or
translated into Rust, no proprietary boot route was selected, and no Roswell
binary is published by this source milestone.

The separate ordinary reference experiment uses the official unmodified
[OXOS v0.3.3 open release](https://github.com/kooscode/oxos/releases/tag/v0.3.3),
BIOS SHA-256 `8bf516fcb453f18925ad0aad75ff923bd5f225420be6b0175d0fe818f0653314`.
Its annotated source tag resolves to
`9d8cd58ca19f49f5ecb4b7b55bfa194f4e4a5a96`; upstream identifies GPL-2.0 and
Cromwell `86f547387184d1001d377cce97d3756c8acf91cc` lineage. The asset and its
downloaded GPL license remain private experimental inputs. No release binary
is committed or shipped, and no source-build reproducibility or license
clearance is inferred from the release claim. See the
[reference result and decision packet](docs/OPEN_FIRMWARE_DECISION.md).

The internal dashboard control uses the exact public
[xemu-dashboard v20260516-0955 release](https://github.com/xemu-project/xemu-dashboard/releases/tag/v20260516-0955),
source tag `8cb5e525a8d2c980a4daa6b2c2106b53661b5b20`.
Disk SHA-256 `00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd`;
complete bundled license SHA-256
`3c4d8c95fdcd47f61e106f1e597d8c9db37a57193d0239346edcbc9f4f2334f6`.
The [complete component/font notices](LICENSES/xemu-dashboard-v20260516-0955/LICENSE)
are retained verbatim. Root MIT does not replace the combined artifact's
component terms. Only the notices are committed; the exact disk remains private
qualification input. Its workflow's floating nxdk container tag means this is
artifact qualification, not a reproducible-build or shipping-default claim.
See [dashboard qualification](docs/DASHBOARD.md).

Record upstream URL, exact commit, recursive dependency pins, original notices,
file-level license exceptions, local modifications, generated binaries and
reproduction commands. Maintain a patch series or deliberate fork with a known
reference build. Recheck no proprietary inputs entered generated artifacts.

Changes inside a GPL core remain governed by the applicable license. A Rust
translation is not a licensing reset, nor does a C ABI automatically remove
combined-work obligations. Keep first-party and borrowed attribution honest.
No additional marketplace permission or worldwide legal clearance is claimed.

## Bounded public API journal

`patches/oxos-api-journal.patch` composes the accepted d87/v4/complete-read
changes with targeted edits to retained `kernel/src/xe/trace.cpp`; its import
trampoline remains retained OXOS source. GPL/per-file terms apply. It removes
argument/name/payload capture, reduces the ring, guards late-return slot reuse,
and preserves ordinal232 callback identity. The separately retained
`oxos-api-journal-v1.patch` documents the first diagnostic's identity limitation.
The corresponding `kernel-journal*.lock.json` files bind their actual sources.

`native/fixtures/api-journal.S`, `runtime_summary.rs`, `api_journal.rs` and new
Rust build/runner policy are original MIT work. Existing pinned OXOS mkxbe/mkxiso
keep their own terms. Public nxdk a6c31ee90540710a3df26a0712b61492f2ea03dd and
Cxbx f1e54be515bb7989bf5109ebea2494b6b13eb1bd interfaces were read for comparison;
no implementation was copied. See [API_JOURNAL.md](docs/API_JOURNAL.md).

## Scoped ordinary open startup and required reads

`patches/oxos-basic-startup-phases.patch` and
`patches/oxos-required-reads.patch` modify public OXOS loader source at
`9d8cd58ca19f49f5ecb4b7b55bfa194f4e4a5a96`; OXOS GPL-2.0 and applicable per-file
terms remain. The required-read guard reuses the functional behavior previously
accepted in `oxos-loader-complete.patch`, without its diagnostic chain. Original
Rust build/test policy and synthetic required-read fixture are MIT; the C++ file
is a mechanical host ABI adapter. Extracted upstream function bodies and their
linked fixture keep actual upstream obligations. No binary is published. See
[OPEN_STARTUP.md](docs/OPEN_STARTUP.md) and `open-startup.lock.json` for pins,
qualification limits and actual artifact identities.

## Cartridge session policy and headless native transport

GCX at `ac9693c5c3aee9d9407cb8abfa42632414f73020` supplies reviewed MIT policy
ideas: `crates/gcx/src/checkpoint.rs` (blob
`450d68071db78d5aa475feb3b5c54e547807752b`), `crates/gcx/src/storage.rs`
(`5841f97982d7e213f395df27713c84a107559dcf`), `native/bridge.cpp`
(`85746e3a320ea007175ca3180e15acfb0bf21548`) and checkpoint/storage sections of
`docs/ARCHITECTURE.md` (`c21acf2b7d21f20f61cd447e59912357ae6c51e3`).
Copyright (c) 2026 gcx contributors; xbx contributors (bootstrap foundation);
Machine Rhythm contributors (PSX layout and icon adaptations). Its MIT notice
is retained in `licenses/GCX-MIT.txt`. The new Rust cartridge modules adapt
validation before mutation, generation tracking before writes, durable reference
publication and failed-load isolation to Xbox HDD/EEPROM and qcow2 whole-machine
state. They do not copy the GameCube card representation or modify GCX.

`patches/xemu-headless-session.patch` modifies retained xemu `03b988c9` frontend
`ui/xemu.c`, preserving its GPL-2.0-or-later lineage. It adds the missing
no-presentation init/retirement handshake; native snapshot/device owners remain
upstream. The accepted base and RAMHT modifications remain in the linked core.
`native/fixtures/cartridge.S` and its Rust XBE/XISO builder are original MIT
synthetic code. LPC47M157/16550 and ATA register constants are public interface
facts, cross-checked against pinned public device implementations. Terminal input
and a reserved sector marker qualify only these disposable original programs,
not controller input, commercial saves or Fable.

The prepared `xemu-paused-savevm.patch` changes public xemu `03b988c9`
`migration/migration-hmp-cmds.c` under its GPL lineage, reusing the retained
Android wrapper's paused-capture transition. `fixtures/paused_capture_contract.rs`
is original MIT policy regression code; `paused_capture_shim.c` is mechanical
host C-ABI glue. Its generated `subject.inc` is the actual GPL-derived public HMP
function, retained privately with the build; the shim does not relicense it.
The test controls owner-preparation and error dependencies, not emulated i386 ABI,
serializer completeness or native-thread concurrency. Current execution status
is recorded in CARTRIDGE.md rather than inferred from the test's presence.


The ABI4 Android cartridge candidate retains the same pinned Android core and
accepted Android patches, adding the reviewed HMP paused-capture and RAMHT repairs.
`patches/xemu-android-image-reader.patch` modifies the retained fork's
`android/app/src/main/cpp/xemu_hdd_tools_jni.c` under its actual GPL lineage. It
exports read-only block open/length/read/close operations for a separate packaged
Rust helper, reusing QEMU decoding rather than translating its disk format.
The first-party Rust helper/lifecycle/packaging code is MIT; the mechanical
Android bridge/Activity changes retain their existing notices. Candidate assets
include the actual QEMU, SDL, OXOS and first-party notices. No sibling source or
private firmware was added to this Android package.

`patches/xemu-android-session-monitor.patch` connects only the existing native
savevm/loadvm/snapshot-listing routines to the Android donor's previously empty
HMP table. Its entries retain pinned `4967efd3` `hmp-commands*.hx` interfaces;
the name comparator is copied from `monitor/hmp-target.c`, with Fabrice Bellard's
copyright and permission notice retained in the patched source. The combined
monitor integration retains the core's GPL obligations. Rust still owns session
policy, storage safety and publication; no new machine serializer is introduced.
`native/fixtures/android-hmp-session.c` is an original MIT host routing contract.
Its privately generated subject is the actual pinned/patched monitor source;
controlled callbacks qualify command routing, not native machine state.

### Precise x87 restart correction

`patches/xemu-x87-instruction-boundary.patch` changes the retained
`target/i386/tcg/translate.c` at xemu03b988c9d76553b33ed712ba00822ffdf49f0e7d.
The file's LGPL-2.1-or-later notice and authorship remain; this is a targeted
modification of that implementation, not MIT first-party emulation code. The
original Rust build/qualification tooling and original guest fixture remain MIT.
The composition retains the existing xemu/QEMU and dependency obligations. Only
one C unit is rebuilt; accepted RAMHT, UI and paused-capture repairs are retained.
[Source/build identities and scoped proof](docs/evidence/cartridge-x87-boundary-20261027.json).

The targeted `patches/oxos-queued-flush.patch` modifies retained OXOS
`kernel/src/idex/channel.cpp` and `kernel/src/idex/disk.cpp` at the same pinned
`9d8cd58ca19f49f5ecb4b7b55bfa194f4e4a5a96` lineage. OXOS GPL-2.0 and applicable
per-file terms apply. `cartridge-flush-io.lock.json` records the exact changed
source and original negative. The original generated file controls and Rust
build/session policy remain MIT. The retained public QEMU completion path and
Linux v6.12 timeout table are behavioral references; no Linux implementation,
game code, private firmware or console data is copied into this correction.

## User-buffer residency correction

`patches/oxos-buffer-probe.patch` changes retained OXOS
`kernel/src/mm/memory.cpp` from the pinned `9d8cd58c` lineage after the accepted
read, NPX and queued-flush repairs. OXOS GPL-2.0 and applicable per-file terms
continue to apply. The first-party Rust build policy and original demand-buffer
fixture are MIT. The fixture reuses the exception-registration ABI of the
existing original `nullcall.S`; no title implementation or data is included.

Microsoft's public [MmProbeAndLockPages documentation](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-mmprobeandlockpages)
is behavioral guidance for making pages resident before DMA and the IRQL limit
for pageable probes. Xbox's API is distinct; no Microsoft implementation was
copied. `cartridge-buffer-probe.lock.json` binds the source composition, small
functional patch and exact original negative. Build success alone does not
qualify the repair or establish a Fable cause.

## Public I/O failure reporting

`patches/oxos-read-errors.patch` adds bounded negative open/read/completion
reporting to retained `kernel/src/io/file.cpp` and `kernel/src/io/irp.cpp`,
after the accepted user-buffer correction in the same `9d8cd58c` lineage.
OXOS GPL-2.0 and applicable per-file terms apply. The Rust build policy is MIT.
`cartridge-io-errors.lock.json` binds the parent, accepted repairs, two changed
files and patch. Reported fields are public status, byte-count and flag values;
each failure class has32 lines and one limit marker. Paths, payloads, caller
instructions and stacks are excluded. This is diagnostic source for the newly
observed disc-error boundary, with game acceptance still requiring actual
visible results and an independently supported correction.

The targeted `patches/xemu-apu-paused-reset.patch` modifies the retained xemu
APU reset hook under that file's LGPL-2.0-or-later terms, preserving upstream
notices and lineage. The generated-data Rust QTest control is original MIT code.
See `docs/evidence/cartridge-apu-quiescence-20261029.json` for the exact source,
patch/candidate identities and actual red/green/active-control scope.

`patches/xemu-f32-exact-flags.patch` targets `fpu/softfloat.c` at the same pinned
xemu source. Its additions are GPL-2.0-or-later; the retained file's original
SoftFloat 2a, BSD and GPL notices and terms remain applicable. The bounded
binary32 fast path computes an exact binary64 intermediate and the actual
inexact flag, with unchanged software fallbacks. It does not force exception
flags or change guest rounding modes. Independent reference checks reuse the
retained QEMU `tests/fp/fp-test` and bundled Berkeley TestFloat/SlowFloat sources
under their existing notices; none are copied into first-party source.
`native/fixtures/cpu-performance.S` and its Rust generation/qualification owner
are original MIT code. Original workload results establish neither game
compatibility nor Android performance. Existing runtime packages remain retained.

The desktop KVM qualification enables the existing KVM implementation in the
same pinned xemu/QEMU lineage. It adds no accelerator implementation or newly
downloaded dependency. Upstream per-file GPL/LGPL notices and retained repairs
remain applicable; the qualification owner and session-identity policy are MIT.
This is Linux x86-64 work, not an Android acceleration claim. The arithmetic
candidate is not included in this separately identified build.

`patches/xemu-controller-frame.patch` adapts the existing controller state and
USB report path in retained `ui/xemu-input.c` at the same pinned source. Those
additions are GPL-2.0-or-later, with upstream notices retained. Its two monitor
declarations retain `monitor/hmp-target.c`'s permissive notice; command-table
additions retain the applicable QEMU terms. This introduces no machine-state
serializer or game implementation. The Rust controller transport, qualification
owner and original `native/fixtures/clock-continuity.S` are MIT. The clock fixture
reuses xbx's original UART/envelope code and checks imports against the retained
public ABI declaration, without copying a title or kernel implementation.

`patches/xemu-smc-byte-range.patch` enables the retained overlap test in
`accel/tcg/tb-maint.c` at the same pinned xemu source. The file's
LGPL-2.1-or-later terms and upstream notices remain. It retains the existing
precise self-modifying-code machinery. The Rust qualification owner and original
`native/fixtures/smc-data-range.S` and `fp-continuity.S` remain MIT. Original
controls establish the stated invalidation and CPU-state behavior, not a
commercial game speedup or complete GPU-state acceptance.

`patches/xemu-vertex-state.patch` extends the existing VMState description in
`hw/xbox/nv2a/nv2a.c` at that pinned xemu revision, retaining its
LGPL-2.0-or-later notice and authorship. `patches/xemu-prelaunch-savevm.patch`
corrects the accepted HMP hook in `migration/migration-hmp-cmds.c` under that
file's GPL-2.0-or-later terms. Neither adds a second machine serializer.
The original MIT `native/fixtures/nv2a-vertex-state.inc` and its separately
gated monitor adapter are qualification machinery; upstream file notices and
combined core obligations remain. The Rust owner and fixture contain no game
or private-firmware implementation. Old incomplete checkpoints remain retained.

`patches/xemu-vblank-timer.patch` modifies the same retained NV2A source under
its LGPL-2.0-or-later terms. It transfers the qualified nominal60Hz interrupt
generation from host presentation to QEMU's virtual timer and saves its phase
with the existing native serializer. It does not establish programmable scan,
PLL or PAL/interlaced timing accuracy. The original PCRTC counter fixture
`native/fixtures/vblank-cadence.S` and Rust qualification owner are MIT. No game
or private firmware implementation is copied.

`patches/xemu-serial-watch-source.patch` modifies retained public xemu/X1 BOX
`4967efd3ee594cc236869a45ee4cb396e25af1ac` character frontend and UART code
(`chardev/char-fe.c`, `hw/char/serial.c` and their two headers). Their original
Bellard permissive notices and applicable file terms remain; the combined core's
GPL obligations remain unchanged. The Rust owner and original MIT
`native/fixtures/serial-watch-context.c` are qualification machinery. The
generated private test includes preserve the native source notices and unchanged
function bodies. No title or private firmware implementation is included.

`patches/oxos-launch-disposition.patch` changes only public boot/HAL launch
reporting in retained OXOS `9d8cd58ca19f49f5ecb4b7b55bfa194f4e4a5a96` under
its GPL-2.0 terms. No launch path/payload, title code or proprietary firmware is
copied. `patches/xemu-tier1-fault-state.patch` changes the retained Android
`tcg/tier1-opt.c` at the xemu/X1 BOX pin above, retaining its GPL-2.0-or-later
notice and obligations. The Rust owner and original controlled-IR scaffold
`native/fixtures/tier1-fault-state.c` are MIT; generated private test subjects
retain the complete native notice and unchanged function bodies. The fixture
uses the retained QEMU queue header under its existing terms. It does not claim
actual vCPU exception delivery or commercial compatibility.

`patches/xemu-tier1-baseline.patch` changes only the optional extra-tier threshold
in retained `accel/tcg/cpu-exec.c` at the same Android pin, under its existing
LGPL-2.1-or-later terms and notices. It is a separate ordinary-TCG developer
comparison, not a replacement CPU or an attributed game fix. The Rust builder
and ordinary integration-input qualification owner are MIT.

`patches/oxos-startup-handoff.patch` adds bounded public thread-status and
exception-code reporting to the same retained OXOS kernel under GPL-2.0.
It contains no game code, caller address, stack contents or proprietary input.
The Rust build and packaging owner remains MIT. The logging image is a
separately bound developer candidate, with its prior sources and objects retained.

`patches/oxos-android-fault-record.patch` adds one private access-violation
context record per boot to the same GPL-2.0 OXOS exception dispatcher. It records
register/status fields, with no instruction bytes, memory payload, stack walk or
game implementation copied into source. Its output stays private. The Rust
builder retains distinct parent-image/native bindings and prior artifacts.

`patches/oxos-serial-poll-yield.patch` adds an x86 PAUSE spin hint to retained
GPL-2.0 OXOS serial polling, retaining its original finite timeout and absent-device
policy. The pinned xemu pause helper supplies the existing event-loop yield;
no new CPU implementation or game code is included.

`patches/oxos-android-fault-word.patch` refines that same GPL-2.0 developer
exception record with an origin tag and one mapped-address-validated stack word.
It performs no stack walk and includes no game bytes. All resulting values remain
private; the Rust owner retains separate source/image/native bindings.

`patches/xemu-fastmem-stale-alias.patch` modifies retained X1 BOX/xemu
`4967efd3ee594cc236869a45ee4cb396e25af1ac`,
`accel/tcg/xbox-fastmem.c`, under its GPL-2.0-or-later terms. It prevents
stale dirty-tracking aliases from changing an invalidated anonymous window
page's protection. The actual public unmap/reset function bodies are reused
unchanged in a private native before/after contract; their SPDX notice remains.
The original owned-memory fixture `native/fixtures/fastmem-stale-alias.c` and
Rust build/audit owner `cartridge/fastmem_alias.rs` are MIT. No game data or
proprietary firmware is included. The correction preserves the native state
layout; prior checkpoints are retained without retagging.

`patches/xemu-vk-vertex-cache-values.patch` modifies the same pinned lineage's
`hw/xbox/nv2a/pgraph/vk/vertex.c` under LGPL-2.0-or-later. The binding cache
retains layout while refreshing live constant/provoking-vertex values, matching
the uncached path. Exact public binding/conversion bodies and register definitions
retain those terms in private contract builds. The original synthetic RAM fixture
`native/fixtures/vertex-cache-values.c` and Rust build/audit owner
`cartridge/vertex_cache.rs` are MIT. No game input or shader is copied. The native
builder replaces one symbol-identified archive member and audits every other
object unchanged, including the existing fast-memory repair. No state layout
change or checkpoint retag is introduced.

`patches/xemu-vk-frame-vertex-coherence.patch` modifies the same pinned
LGPL-2.0-or-later Vulkan `vertex.c` and `draw.c`. It records pending RAM updates
for each destination frame and copies them after that frame's existing fence,
without writing an in-flight buffer. The original MIT separate-buffer fixture
`native/fixtures/frame-vertex-coherence.c` exercises the exact retained update
function and frame-copy block under their LGPL terms; the Rust owner
`cartridge/frame_vertex.rs` is MIT. These host controls submit no GPU work and
contain no game data. Native integration retains both this and the earlier
attribute-value repair; existing checkpoints are retained without retagging.

`patches/xemu-android-present-surface.patch` modifies that pinned lineage's
LGPL-2.0-or-later Vulkan `display.c`. A lost Android presentation surface is
retired after its swapchain, then recreated through the existing SDL surface
factory when the native window is available. The existing queue retirement and
all prior vertex repairs remain. The independent SDL repair
[`c98b36ff`](https://github.com/libsdl-org/SDL/commit/c98b36ff03023affddd12c9c8951d370f7ccb742)
by Sascha Reuter documents the same surface/swapchain distinction; its GPU
implementation is not copied. Vulkan lifetime rules remain authoritative.
The original mock-handle fixture `native/fixtures/present-surface.c` and Rust
build/audit owner `cartridge/present_surface.rs` are MIT. Extracted retained
creation/error branches keep their LGPL terms. These controls do not submit GPU
work or establish the cause of a game's Home-return result.

`patches/xemu-native-thread-names.patch` modifies `util/qemu-thread-posix.c`
at the same X1 BOX/xemu `4967efd3` pin under its GPL-2.0-or-later terms.
The enabled pthread diagnostic label is bounded to 15 bytes for the Linux/Bionic
API; role labels are not guest or permanent session identity. The exact retained
thread-entry function extracted by the original MIT Rust owner
`cartridge/thread_names.rs` keeps its GPL terms. The independent MIT fixture
`native/fixtures/thread-names.c` uses real host pthread naming and joining; it
checks disabled/null naming as well as the overlong public role names. These
source controls do not qualify Android execution, game performance or a new
native candidate.

The same native-name patch also enables the detected Bionic pthread-name API
in the retained Android `config-host.h`. This mechanical configuration and the
thread wrapper keep their upstream terms. The expanded original control uses
the exact before/after Android header as well as the POSIX API branch; it does
not change radio/debugging settings, emulated state or permanent identities.

The separate developer-only `patches/xemu-cpu-phase-cost.patch` inserts host
CPU-time observations into this pin's `accel/tcg/cpu-exec.c` and `tcg-stats.c`
(LGPL-2.1-or-later). All preexisting source bytes and statement order survive
after removing the marked insertions. The original MIT mechanical counter
header `native/xbx_cpu_cost.h`, fixture `native/fixtures/cpu-cost.c` and Rust
owner `cartridge/cpu_cost.rs` retain only aggregate timing/count fields, never
guest PCs, instruction bytes, stacks or register contents. The selected completed
iterations are a sample, not unbiased whole-worker profiling; abandoned samples
and invalid clocks remain visible. This optional developer candidate does not
change the default core or relax Android profiling/security settings.

`patches/xemu-present-semaphore.patch` changes two semaphore selections in the
same pin's LGPL-2.0-or-later Vulkan `display.c`. Presentation waits use a semaphore
per acquired swapchain image, independently of the rotating rendering-fence slot,
following the [Khronos lifetime guidance](https://docs.vulkan.org/guide/latest/swapchain_semaphore_reuse.html).
The original MIT delayed-presentation fixture `native/fixtures/present-semaphore.c`
executes the exact retained submit/present block under its LGPL terms. Its Rust
owner `cartridge/present_semaphore.rs` is MIT. The model checks semaphore lifetime
with slow numbered-picture updates; it is not actual GPU or game qualification.

`patches/xemu-present-tag.patch` adds a diagnostic-only marker to that retained
LGPL-2.0-or-later display unit after an actual repeated-picture observation. It
keeps submission/presentation choices intact and encodes only sequence, acquired
slot, public overlay-use bits and an ephemeral source-binding token. No game
instructions, addresses, stack or raw native handles are recorded. Original Rust
qualification owner `cartridge/present_tag.rs` and the host shader fixture
`native/fixtures/present-tag-shader.c` are MIT. Extracted display GLSL and resource
limits retain the pinned `display.c`/`glsl.c` LGPL terms. The fixture uses retained
glslang under its existing terms; it is a shader compilation/layout check, not a
GPU or gameplay result. This marker is not included in the ordinary candidate.
