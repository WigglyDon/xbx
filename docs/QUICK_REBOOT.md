# Focused upstream quick-reboot qualification

Master [6349453322](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6349453322)
assigned this finite slice. The existing worker acknowledged at
[6349477537](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6349477537), on
clean public source `20d3bfc00b94b1ae000ca85e85b16e0f6ae52fc1`. The implementation
is the commit containing this receipt. One October 22, 2026 desktop cold-start
and quick-relaunch run passed; the required Rust checks passed. This completes
the slice. No further runtime campaign was run.

## Exact source change and preservation policy

Qualification advances OXOS from `c0c887a1263ebf89ba765f34a88ca8f132901e25` to
**`d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2`**, its immediate child. The checked
comparison is exactly `kernel/src/mm/init.cpp`, 16 additions / 14 deletions.
`MmInitSystem` writes each page-table entry directly to its final mapping instead
of clearing executing kernel/stack mappings before rebuilding them during quick
reboot. Persistent allocations and accounting remain in the upstream code. This
is an upstream GPL/per-file correction, not a translated or duplicate backport.
The later exFAT commit is excluded. This does not establish a cause or fix for
Fable's preceding first zero-PC fault.

The old patched source worktree remains at `.local/src/oxos`. A separate sparse
worktree at `.local/src/oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2` retains
the new exact pin. The six-file v4 trace patch applies unchanged; both the actual
working diff and patch file SHA-256 are
`7eb48fb10ca5784a887d93db23f02c07ec83dc7b105930e1a71204724e0a0dc5`.
The Cromwell pin remains `86f547387184d1001d377cce97d3756c8acf91cc`, linked to
the retained verified checkout. The build verifies exact source HEAD, working
patch, absence of untracked source, Cromwell and trace lock binding.

Original MIT Rust build policy now keys all trace/play/self-test output paths by
full OXOS pin, full Cromwell pin, mode and full patch digest (or `none`). Exclusive
directory creation refuses every existing attempt, including interrupted output.
`build.inputs` is written before compilation; successful `build.identity` adds
the actual image digest. Changing source, patch or mode cannot overwrite the old
build. Firmware builds no longer write `.local/selected-launch-build` or any
Android asset selector. Future relaunch fixture builds also use a source-specific,
exclusive output; this slice reused the old sufficient fixture without rebuilding.

Android seed preparation explicitly accepts only the historical exact baseline
or v4 image/identity. It verifies the v4 receipt's old source pin, patch, firmware
and flags without duplicate/conflicting fields. It refuses new firmware before
profile creation/selection. Existing seed identity and flash bytes must match.
Metadata intentionally retains the true historical Android source; no new profile
mode is introduced. Synthetic policy checks prove prior image/receipt preservation
across duplicate/new-source/new-patch/play/self-test attempts and reject candidate
mislabeling, spoofed source, conflicting fields and wrong flags.

## Build inputs and commands

Preflight: 46,217 MiB available storage; 1,469 MiB usable Btrfs metadata; emergency
reserve unused. Actual shared-host available RAM was about 6,410 MiB; unrelated
jobs were left alone. Existing read-only NDK 30.0.16248370 LLVM tools and the
retained Make/Python pipeline were used with one compiler job. The two retained
timer/I/O APC cast warnings remained visible; no errors occurred.

From the existing checkout (the setup commands were performed once; do not
repeat them over retained worktrees/output):

```sh
xbx_kernel_pin=d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2
xbx_kernel_source="$PWD/.local/src/oxos-$xbx_kernel_pin"
git -C .local/src/oxos fetch --depth=2 origin "$xbx_kernel_pin"
git -C .local/src/oxos worktree add --detach "$xbx_kernel_source" "$xbx_kernel_pin"
mkdir -p "$xbx_kernel_source/third_party"
ln -s "$PWD/.local/src/oxos/third_party/cromwell" "$xbx_kernel_source/third_party/cromwell"
git -C "$xbx_kernel_source" apply --check "$PWD/patches/oxos-launch-trace.patch"
git -C "$xbx_kernel_source" apply "$PWD/patches/oxos-launch-trace.patch"
cargo xtask capacity 1
XBX_NDK=/path/to/read-only/ndk/30.0.16248370 cargo xtask kernel-build trace
```

Only the trace firmware was built: IMAGE=open, SELFTEST=0, SERIAL=1,
TRACE_IMPORTS=0, XK_XBX_LAUNCH_TRACE=1, no splash/comparison BIOS/MCPX inputs,
64 MiB. The output basename is:

```text
build-oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2-86f547387184d1001d377cce97d3756c8acf91cc-trace-7eb48fb10ca5784a887d93db23f02c07ec83dc7b105930e1a71204724e0a0dc5
```

Under that retained `.local` directory, SHA-256 identities are:

| Artifact | SHA-256 |
| --- | --- |
| `bios.bin` (1 MiB) | `f5cac7bac71d909772db7be4c73fedbfe935c9ab8001ebd69bc8b79c61de0258` |
| `xboxkrnl.map` | `833d88861aa99d87487f21dfbee0cd4ccbc12f261550efd58539d7ed158d75f8` |
| `build.inputs` | `34a8ace70f7445805e2af3501f446e2db4ef99840b64c3f8ec3cebfdbf5c1e0d` |
| `build.identity` | `9f212b1123656f696df24dbf9dd1d5b1fad7eb8708ec8f42db96a27808ba1d35` |

Kernel date/time macros affect image bytes; retain the actual image and receipt.
The full make command/output is local `build.log` under
`.local/evidence/quick-reboot-d87cfc33`; SHA-256
`e2e7f206f188edff339603c21fb10c964151b413e2adec2f14f944e0c5f1d742`.

## Retained fixture, core and one actual run

The upstream title/wrapping source is unchanged between the two kernel pins.
The existing GPL-lineage fixture, ISO and matching map were hashed before use:

| Input | SHA-256 |
| --- | --- |
| `.local/build-relaunch-title/relaunch.iso` | `8efe94b6f8a26f63247f44bc42da9ed437e431a5ed03180ba73044140cb88491` |
| `kernel-test-title.xbe` | `11cc36698ed0ec10d1d7cbc1a020e3c29736e6e88c9eb1d3fc7584ab4be1956c` |
| `kernel-test-title.map` | `1a45a06f08cc5537b454ed106bcd178d05c1fb955f931d5b6ea6804b814a2f66` |
| `.local/build-desktop/qemu-system-i386` | `51178df74f0b26316c799ff8e8c1a65b1b841b97858cd07dd31903d52ebdcd0d` |
| Empty FATX qcow2 seed | `0f123236d5916e5c455345c4b91cb9876ae821766475e69ede205c8fd9ae2e71` |
| Observing `target/debug/xbx-host` | `4acedebf3dd7eb3dcf600b6c9de2d9de2d6b9f9a1bbbb7c1522b5ad5860f3a9a` |

Native core remains xemu Android fork `4967efd3ee594cc236869a45ee4cb396e25af1ac`,
ABI3, patch SHA-256
`26027ec1436fb6c5f7a51200bf7fe96ceb90ba5695d568a72db29ef446ff461f`.
It was not rebuilt. The existing Rust runner was not changed.

```sh
xbx_kernel_build="$PWD/.local/build-oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2-86f547387184d1001d377cce97d3756c8acf91cc-trace-7eb48fb10ca5784a887d93db23f02c07ec83dc7b105930e1a71204724e0a0dc5"
G_DEBUG=fatal-criticals cargo run -p xbx-host --offline -j1 -- boot-relaunch \
  .local/build-desktop/qemu-system-i386 \
  /home/don/xbx/.local/quick-reboot-d87cfc33-run1 \
  "$xbx_kernel_build/bios.bin" \
  .local/profile-build-0cdedf6d81dabdbeafc1cff78039cf78f19cb7923078f201724e346ca94ee8e8-fatx-v2/assets/hdd.qcow2 \
  .local/build-relaunch-title/relaunch.iso \
  .local/build-relaunch-title/kernel-test-title.map
```

The scratch root must not exist. It contains an independent copied disk and
generated synthetic EEPROM. TCG, SDL offscreen/dummy audio and software Mesa
were used. The runner's existing 60-second bound, capped native log, inherited
profile lease through actual child reaping and subsequent storage sync apply.

Actual guest results:

- First entry executed `\Device\CdRom0\default.xbe`; all required first-stage
  checks survived in launch data. Magic 0x54544b58; result indices
  1,2,3,4,5,7,8,25,26,27 all equaled 1. Index 6 (`keep.bin`) remained 0 and is
  intentionally excluded because its file belongs to the separate FATX self-test.
- One real routine-2 quick reboot occurred. The title's launch-data marker,
  type 1, title ID 0x584b0001, flags and retained data passed its second-stage
  checks. The launch page survived adoption and was relocated below the kernel.
- Second entry executed `\Device\CdRom0\media\second.xbe`. Its guest-produced
  D: mapping check passed against the media directory, and both executable names
  matched in the Rust observer. Loader success and these title results are
  distinct observations; this is actual title execution, not just loader return.
- The bounded serial log (4,054 bytes) contained exactly one cold boot and one
  quick boot; both loads returned status 0. No XBX-EXCEPTION, bugcheck, fatal,
  failed or unexpected-entry-return marker appeared. No later missing-title
  dashboard test was reached. Native log was 6,169 bytes, without a GLib critical.
- QMP stop/quit acknowledged; actual native status 0, forced=false, qmp=Ok(()).
  Storage sync completed before the successful runner return. Guest results and
  native termination remain separate evidence.

Retained `.local/quick-reboot-d87cfc33-run1/relaunch-results.txt` SHA-256 is
`3c6fc9da5aa709005863c9e375af7eb1deb584f4a809a85619a92122b1dae617`;
`termination.txt` is
`35907392d8fc6000474598148a5edd23e7b56e87b2f145467f7b9afbb41e9e8d`.
Synthetic raw logs remain local, alongside the full build/run/hash manifests.

## Checks, preservation and limits

`cargo test --workspace --offline -j1`: **20 passed**, three explicitly gated
native/storage tests ignored. `cargo clippy --workspace --all-targets --offline
-j1 -- -D warnings` passed; format/diff checks passed. An initial Clippy check
flagged test-module placement; moving the test module to the file end resolved
it without a build-policy change. Source checks are additional to guest execution.

Before and after manifests verify **36/36 existing identities unchanged**:
all inventoried retained firmware/maps/receipts, fixture outputs, desktop core,
selected-launch-build, selected-profile-assets and selected Android flash/disk/
profile.identity. Both local manifests hash to
`f6aa52839392f93863e4450d98e6b493784105520de7a6b9b671b868b08feff2`.
Baseline/v1/v2/v3/v4 artifacts remain retained. The old source's patch is preserved.
Owner profiles/media/private evidence were not opened or modified. No cleanup of
other projects, phone use, APK build, game import or binary publication occurred.

This supports only the tested desktop cold-start/quick-relaunch contract with
this firmware/core/fixture/configuration. It does not qualify Android, arbitrary
reboots, gameplay, rendering, audio, ordinary saves, snapshots, shortcuts or
performance. The synthetic carried launch data is not an ordinary game save or
full-machine checkpoint. Fable remains unqualified; its private binary analysis
stays paused under Master6348944459. The prior device return remains settled.
