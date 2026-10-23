# Separate pinned upstream xemu desktop comparison

This comparison follows the owner's continued public-platform diagnosis request.
The retained Android fork4967efd3ee594cc236869a45ee4cb396e25af1ac, its binary,
Android assets and prior profiles are preserved. This is the same xemu lineage,
not selection of another donor or a new Android integration. Build success must
remain distinct from guest and unchanged-game execution results.

The retained public fork and already pinned reference xemu
03b988c9d76553b33ed712ba00822ffdf49f0e7d diverge substantially. Local fork source
contains additional CPU translation, floating-point, cache and ARM-only fastmem
paths. The source-directed existing-core software-FP control still failed, as
recorded in [API_JOURNAL.md](API_JOURNAL.md). No individual optimization is named
as the cause. A source-built upstream comparison changes more than one internal
component and cannot establish that causal attribution by itself.

## Source/build preparation

`reference-core.lock.json` pins public xemu03b988c9d76553b33ed712ba00822ffdf49f0e7d
and full patch4e317283ddb368c1c80243804aa7fcfdc1e5228ac6d7ddec9ca84cb5cdd138e1.
Targeted retained C/build integration changes only:

- The already justified character-watch owner-reference repair and declaration.
- Opt-in whole-process exit after actual QEMU cleanup for the existing Rust
  supervisor, protecting the profile lease through death/reap/storage sync.
- The already exercised Epoxy declaration on the offscreen graphics target,
  providing its required include/link dependency without changing graphics code.
- Source-built DSP integration using the already retained Rust library from
  dsp56300 bde7c233a447d32d558c48cb4438e1425fe0bbcc, rather than a release archive.

No CPU, graphics or emulated hardware behavior is patched. Applicable QEMU/xemu
GPL and per-file notices remain; DSP retains its own terms. First-party build
policy is original Rust, in `xtask/src/reference.rs`; existing Meson/Ninja/LLVM/
GCC tools remain donor build machinery.

An initial capacity check allowed an8GiB estimated peak:36,130MiB free,
1,394MiB usable Btrfs metadata, emergency reserve unused,5,937MiB available RAM.
The Rust writer lease and fresh capacity checks guard configure/build. One
compiler job; no cleanup of old artifacts or other projects.

A separate source tree was fetched at the exact pin with depth1. Required
upstream Git/wrap dependencies keep their declared pins and package hashes.
Only exact clean already-retained dependency roots were reused. All eight reused
roots remained tracked-clean; the two package overlays are byte-identical to the
retained source. Newly required dependency revisions were fetched into this
separate source. DSP was not rebuilt or downloaded. The v1 configuration receipt
is preserved; v2 corrects the dependency-root inventory to exclude non-Git
subdirectories inheriting the parent's Git identity. No v1 core was built. The v2 build stopped on the missing Epoxy include path;
its original patch1ae1160388b0022019a5a76ec134564d0793afdfc35dba2844ee321988d6fc80,
lock, partial objects and failure log are retained. The corrected v3 attempt
has its own full-patch output directory.

```sh
git init .local/src/xemu-reference-03b988c9d76553b33ed712ba00822ffdf49f0e7d
git -C .local/src/xemu-reference-03b988c9d76553b33ed712ba00822ffdf49f0e7d remote add origin https://github.com/xemu-project/xemu.git
git -C .local/src/xemu-reference-03b988c9d76553b33ed712ba00822ffdf49f0e7d fetch --depth=1 origin 03b988c9d76553b33ed712ba00822ffdf49f0e7d
git -C .local/src/xemu-reference-03b988c9d76553b33ed712ba00822ffdf49f0e7d checkout --detach FETCH_HEAD
git -C .local/src/xemu-reference-03b988c9d76553b33ed712ba00822ffdf49f0e7d apply "$PWD/patches/xemu-reference-desktop.patch"
cargo xtask reference-configure v3
cargo xtask reference-build v3
```

Build directories bind full source pin, patch hash and attempt. Configure refuses
an existing attempt. Build refuses changed inputs or a completed identity;
interrupted outputs/logs remain. Configure disables docs/tools/guest-agent/QEMU
Rust devices, targets i386-softmmu with XBOX, and uses the existing task-local
host sysroot/tools. Dependency pins, DSP library digest, actual executable digest
and reproducible command/configuration outputs are retained in the attempt.

## Actual build and first guest qualification

The corrected v3 build completed all1,668 Ninja actions with one compiler job.
Actual core SHA-256:
`31e1c0f5f3c165d25e9c6315ecaf3191b972f57ae22616d03de6ee8d6243be3d`.
DSP library SHA-256:
`5e78afecce79231338bd6c1838a55d300886de3be4e38f3f82229013ac23f1a7`.
Dependency pin-receipt SHA-256:
`d1dc2723f0812e018ffa934aaa5dc9aa5e494534df1afdf79347ed69acf764ca`.
Compiler/dependency warnings remain in untouched donor components; the initial
missing Epoxy dependency was corrected. This is a private comparison build,
not a public binary release or an Android ABI3 engine.

Existing original valid three-section fixture8948d277465e41d82926a3467d3f1fd311090123866d058ae6f3276e226e0390
ran under the new core in fresh `.local/reference-loader-v1` with retained
complete-read firmwareea5aad470a8a47748ed1ad22cbf9d43cebb335a0d33801ccec6230c746a43fef,
64MiB and independent synthetic disk/EEPROM. Actual guest words:
`[1280852568,1,0,512,504,64,448,1]`; initialized data/zero-fill/shared-page checks
passed. Header1088/1088, raw2566/2566, extents4/4, match3/mismatch0/unusable0.
No unexpected fixture exception/bugcheck. QMP stop/quit acknowledged, actual
native exit0, forced=false; Rust retained the lease through reaping then synced
storage before successful acceptance. This qualifies this fixture on the new
core; the accepted old-core loader campaign was not restarted.

Executed observer host remains
`c5a25650730b7f45bd5be5eaa905286c4821070d4747d07c640d554f0917fa17`.
Build/result logs and source/dependency/preservation receipts are private under
`.local/evidence/api-journal-v1`.

## Unchanged-game comparison

One normal launch used a fresh `.local/fable-reference-v1` profile, the same
retained complete-read firmware, generated EEPROM/disk templates, unchanged
read-only selected media and 64MiB. X11/OpenGL/software Mesa and dummy audio
match the retained-core control. Controller S was actually present on port1
in the QMP device inventory. No snapshot, owner profile or Android asset was
used or replaced.

Required header bytes8760/8760, raw bytes8328772/8328772, extents16/16;
short0/I/O failures0, preloads15/15, digest matches15/mismatches0/unusable0.
Loading and entry dispatch succeeded. The title then produced one nonzero-PC,
null-adjacent c0000005 data-read exception and bugcheck0x1e at19.859seconds.
Existing numeric exception records (PC, fault address, code, origin) are
identical to the retained-core non-wrapper control. No title instructions or
buffers were read. This weakens a retained-fork-only explanation; it does not
attribute the cause to OXOS or rule out shared components. These startup times
are not gameplay performance measurements.

QMP stop/quit acknowledged; native exit0, forced=false, storage_synced=true.
The Rust writer lease stayed held through actual native reaping and sync.
The own-window capture attempt arrived after process exit and safely refused;
there is no reference-game screenshot or visual menu acceptance. Gameplay,
audio, ordinary saves, full resume, shortcuts and mobile performance remain
unqualified. The desktop progression gate for a Pixel trial remains unmet.

Reproduction after the build above (each scratch root must be new):

```sh
REF=.local/build-reference-03b988c9d76553b33ed712ba00822ffdf49f0e7d-4e317283ddb368c1c80243804aa7fcfdc1e5228ac6d7ddec9ca84cb5cdd138e1-v3/qemu-system-i386
FLASH=.local/build-oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2-86f547387184d1001d377cce97d3756c8acf91cc-loader-420490cdde28fb40a882c179aa735ae6bd6c20aa4a056bed5131dcabfb4d5fcb/bios.bin
HDD=.local/profile-build-0cdedf6d81dabdbeafc1cff78039cf78f19cb7923078f201724e346ca94ee8e8-fatx-v2/assets/hdd.qcow2
G_DEBUG=fatal-criticals target/debug/xbx-host boot-loader-fixture "$REF" /new/short/fixture "$FLASH" "$HDD" .local/build-loader-fixture-v1/valid.iso .local/build-loader-fixture-v1/title.map valid
# EEPROM and MEDIA identify only the authorized local generated seed and game.
XBX_DESKTOP_VIDEO_DRIVER=x11 G_DEBUG=fatal-criticals target/debug/xbx-host run-title "$REF" /new/short/game "$FLASH" "$HDD" "$EEPROM" "$MEDIA" 300
```

Neither phone was operated or queried. All prior device-return receipts remain
settled; no current screen state is inferred from those earlier returns.

After execution, all41/41 retained artifact identities still match the
pre-comparison manifest, all eight reused dependency roots remain tracked-clean,
and the full selected-media hash matches. Workspace checks:23 passed,3 gated
tests ignored; Clippy with warnings denied, format and diff checks pass.
These source checks remain separate from the actual original guest result and
the unsuccessful unchanged-game launch.
