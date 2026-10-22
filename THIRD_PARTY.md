# Source provenance

## Seed inventory

The initial foundation at `59fbcfa` contained documentation/bootstrap only.
The qualification implementation now builds the candidate below from pinned
public source in ignored local storage. No game, proprietary firmware, EEPROM,
save, signing key, font/logo asset or PSX implementation is incorporated here.

Original contributions use the root MIT license. That grant does not cover
software added from elsewhere. A future combined distribution must satisfy the
exact licenses of its core, dependencies, assets, firmware and copied source.

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
- Original Rust modules are MIT. No PSX files were copied. XDVDFS offsets/layout
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
  Private qualification now builds its open route at
  `c0c887a1263ebf89ba765f34a88ca8f132901e25` with Cromwell
  `86f547387184d1001d377cce97d3756c8acf91cc`; it uses neither a comparison BIOS
  nor MCPX dump. The private player candidate includes that normal-play image,
  not a proprietary console image or an approved distributable default.
- PSX is a related product, not an xbx dependency. Any selected public MIT host
  source may be reused with its original copyright notice. Do not copy its
  GPL cores, fonts or firmware and label the resulting tree MIT-only.

Observed revisions and source URLs are in `upstream-candidates.json` and
[docs/SOURCES.md](docs/SOURCES.md). Neither open-kernel candidate is selected as
a distributable default. OXOS documents non-stock save-signature keys and a
separate proprietary comparison-image route; neither may be silently substituted
beneath an existing profile. Roswell's 128 MiB configuration is unqualified here.

## Before incorporation

Record upstream URL, exact commit, recursive dependency pins, original notices,
file-level license exceptions, local modifications, generated binaries and
reproduction commands. Maintain a patch series or deliberate fork with a known
reference build. Recheck no proprietary inputs entered generated artifacts.

Changes inside a GPL core remain governed by the applicable license. A Rust
translation is not a licensing reset, nor does a C ABI automatically remove
combined-work obligations. Keep first-party and borrowed attribution honest.
No additional marketplace permission or worldwide legal clearance is claimed.
