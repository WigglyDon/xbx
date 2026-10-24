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

The separate upstream desktop comparison retains public xemu at
03b988c9d76553b33ed712ba00822ffdf49f0e7d. Its complete targeted integration diff
is `patches/xemu-reference-desktop.patch`, bound by `reference-core.lock.json`.
The character-watch ownership, supervised process exit, Epoxy dependency and
source-built DSP integration retain QEMU/xemu GPL and applicable per-file terms.
No emulated CPU/hardware behavior is changed by that patch. Existing dependency
pins and notices remain. Original Rust build policy is MIT. See
[the exact build/result receipt](docs/REFERENCE_CORE.md); this comparison does
not select a new Android core or authorize a public binary release.

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
