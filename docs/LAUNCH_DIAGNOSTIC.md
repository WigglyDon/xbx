# First quick-reboot diagnostic

Master continuation 6345094692 authorizes this bounded follow-up to the Fold
failure at 6612ba2. It does not select a shipping kernel or authorize a public
binary. Baseline firmware, failed profile and both phone media copies remain
preserved. No PC game image is required.

## Source and build binding

`kernel-trace.lock.json` pins OXOS/Cromwell and the exact
`patches/oxos-launch-trace.patch`. From a clean pinned OXOS working tree, apply
that patch once. The kernel build refuses any other tracked diff or untracked
source. The trace variant uses its own directory keyed by patch digest, writes
an image/build receipt, and retains the prior self-test and normal-play builds.

```sh
git -C .local/src/oxos apply "$PWD/patches/oxos-launch-trace.patch"
cargo xtask kernel-build trace
cargo xtask relaunch-build
cargo xtask profile-assets trace
cargo xtask android-host
cargo xtask android-apk
```

Use the existing read-only SDK/NDK described in BUILD.md. Flags are IMAGE=open,
SELFTEST=0, SERIAL=1, TRACE_IMPORTS=0, XK_XBX_LAUNCH_TRACE=1, no splash/comparison
BIOS/MCPX, and 64 MiB. Kernel date/time macros affect the built digest; retain
`build.identity` and actual image. Original baseline image remains
`668d08af6de204ce07a1479c8854d8f688af312ed6104e8ab56391dffc301fa6`.
The first trace image preserves it and adds no dashboard.

The trace logs entry dispatch and unexpected return, firmware-return routine,
caller and current thread address, quick-reboot count, launch-page validity,
type/title/flags, bounded sanitized path, and dashboard reason/code only when
applicable to an empty-path dashboard handoff. It also logs selected branch,
load status, missing ordinal and preceding exception snapshot. At most 48 trace
events survive across quick reboots; there is no full import trace or launch-data
payload dump. Raw title fields and addresses remain private. Successful loader
return, dispatched entry, confirmed execution and produced frames are distinct.

## Focused native fixture

`cargo xtask relaunch-build` builds the retained kernel-test-title and invokes
the pinned XDVDFS generator directly. It fetches no default HDD/dashboard and
runs no upstream campaign. The fixture is GPL-lineage source, not an original
xbx title or commercial-game result.

```sh
cargo run -p xbx-host --offline -- boot-relaunch \
  .local/build-desktop/qemu-system-i386 /new/short/scratch \
  /selected/build/bios.bin /isolated/seed/hdd.qcow2 \
  .local/build-relaunch-title/relaunch.iso \
  .local/build-relaunch-title/kernel-test-title.map
```

The Rust runner uses a new owned profile, independent writable disk/EEPROM,
read-only synthetic disc, 60-second bound and capped native log. It polls the
retained title results for actual first entry, carried first-stage results,
second entry, launch-page preservation and D: mapping to media. The missing
`keep.bin` test is excluded because that file belongs to the unrelated FATX
self-test. The fixture later deliberately launches missing.xbe/dashboard; the
runner stops after the successful two-stage boundary, before that later test.
QMP stop precedes quit, the request is acknowledged, the child is reaped, and
ownership remains through termination. Earlier forced-reap failure is retained
separately from the successful title boundary.

Baseline normal-play firmware passed the two-stage boundary and normal shutdown.
The instrumented image also passed the boundary; one first shutdown attempt
required forced reaping. Stop-before-quit completed normally once but did not eliminate the intermittent
shutdown issue. The runner also keeps the acknowledged QMP transport alive through
child termination and records actual status/forced cleanup separately. The
latest focused run completed the boundary and acknowledged normal shutdown with
that retained transport. Earlier failures remain preserved.
None of these establish Fable entry, graphics, sound or saves.

## Separate Android diagnostic branch

The diagnostic asset identity selects only `xbx-launch-diagnostic-v1`, keeping
`xbx-qualification` unchanged. First provisioning acquires the stopped parent's
OS lease, verifies its firmware/session binding, then independently copies disk,
EEPROM, dirty epoch and retained URI. A new profile ID shares the stable GameId
and media fingerprint. The branch binds the instrumented firmware and records
parent profile/epoch and exact copied-file fingerprints. Copies are verified;
no writable inode or RAM snapshot is shared. The completed ancestry manifest is
published last. Incomplete branches and any firmware mismatch refuse reuse.
No existing EEPROM is regenerated and no parent firmware/session is overwritten.
The normal profile mismatch guard remains active.

Synthetic tests prove source-writer refusal, external qcow2 backing/data-file refusal before copying,
unchanged parent bytes, new profile
identity, independent later disk writes and preservation/refusal of incomplete
branches. Android must additionally confirm the baseline fingerprints before
and after provisioning/run. Diagnostics automatically request pause on an
observed guest fatal-error marker and close the resume gate; this preserves the
error, not a successful boot or checkpoint. Process termination remains separate.

## Safe content geometry

The mechanical Android adapter obtains system-bar/cutout insets and supplies one
safe rectangle in View coordinates. Rust applies that rectangle to surface,
control drawing and hit testing. Whole circles, including the rightmost button,
are clamped inside it; geometry changes neutralize captures. Focused bounds,
4:3 surface and transformed-hit checks pass. Actual Fold visual verification and
touch-to-game acceptance remain required.

## Device result

No diagnostic-device result is claimed by the preparation above. Obtain a fresh
operating window, fingerprint the stopped baseline, install the separately
signed candidate, collect the first causal handoff and verify safe layout. Stop
a terminal failure promptly and compare baseline fingerprints before locked
return. Supplementary Pixel 4 performance comparison waits for a meaningful
running workload. All owner data and raw logs remain private; full restore is
still disabled.
