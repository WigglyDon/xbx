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

The October 22 Fold run used source b654b6d, APK SHA-256
`575e126f6ca88f533ff68c2b6c771727f41219638967955d9686ad53d028f518`,
version 3 / `0.3.0-launch-diagnostic`, unchanged ABI3 core, and trace firmware
`3b6e9485a4224d7f39425e0664e0907be7e65b8e3700ff66d022a76822ce2ab7`.
Patch SHA-256 is `ab4094c24e38875ced724264641adbbdb2bafce1f1739f46f4c9e7599d0a1657`.
Rust library SHA-256 was
`1cf7dd3effafae837cf412f85e22426df87fbcc25c3e67c4a02f747e57ea3a37`.

The first handoff was captured completely within the private log cap: disc load
success, entry dispatch, entry-thunk return, then a different title thread's
HalReturnToFirmware routine 2 with valid launch page, type 1, empty path and
dashboard reason/code 1/1. Page address and fields survived re-adoption. The
empty-path branch selected dashboard; its load returned C0000034, routine 4
followed, then cold fatal 21. No preceding recorded exception or missing import
ordinal was observed. Entry-thunk return alone is not evidence of a failure;
the title-side reboot caller proves execution beyond loader return. This still
does not identify the startup check behind the generic error-dashboard request.
The trace's last-load global stayed zero after dashboard open failed early;
the explicit loader line is the evidence for C0000034.

Provisioning recorded a new profile ID/ancestry and retained the GameId/media
binding. Seven stopped baseline files (firmware, disk, EEPROM, generation,
SavedSession, profile identity and media grant) matched byte-for-byte before and
after. A private visual check showed complete control circles inside the safe
area. This is layout acceptance only, not touch-to-game acceptance. The fatal
marker closed the Rust resume gate; operator force-stop terminated the process,
not a clean checkpoint. The phone was verified locked/screen off and released.
No gameplay, audio, ordinary save/cold reload, snapshot/shortcut or performance
acceptance was earned. Pixel 4 remains deferred. Raw logs/captures and profile
identities remain private; full restore stays disabled.

## Bounded callsite follow-up

The generic title error needs a preceding-check diagnosis. Version 4 adds a
read-only Rust observation before native initialization or storage-epoch writes,
using the same validated retained game grant and unchanged diagnostic firmware.
It activates only with an explicit private `callsite.request` in a completed
diagnostic branch. Format: `xbx-callsite-v1 HEX_VIRTUAL_START HEX_BYTES`.
The operator chooses the range from the observed caller, not a product default.
At most 32 KiB from one executable, file-backed XBE section may be recorded;
header/certificate ranges, ambiguous section mappings, zero-fill and invalid
extents are refused. Numeric kernel import slots are bounded separately. Raw
code stays in private `callsite-evidence/code`; the numeric map stays beside it.
Neither is suitable for public source or issue upload. Existing/partial output
is preserved and refused on repetition. The process exits without starting QEMU.
Synthetic tests cover bounds, header exclusion, refused non-code mapping,
read-only media and preservation of prior evidence. The October 22 Pixel 4 run
collected the requested 12 KiB and numeric import map through this path, then
exited without QEMU entry. Neither raw output is published.

## Pixel 4 priority and first execution

The owner made the accessible stock Pixel 4 the priority compatibility target.
Its active lab policy and complete prior return were checked before acquiring
exclusive custody. Actual Android 13/API33, ARM64, stock user build and 4096-byte
pages matched the qualified binding. The xbx package was absent. Baseline APK
`5ccf0e39bf36af88b017a52b44df57ea98ab2d3d59151fc0c8b334085fb5130e`
(source 6612ba2, baseline firmware above) installed into that absent namespace.
The exact existing Fable document was selected through the stock picker without
copying media. A new phone-local profile executed the kernel and disc loader,
then reproduced quick reboot, missing dashboard and fatal 21; flip samples were
zero. This is compatibility evidence, not a performance benchmark.

The data-preserving update to version 4 / `0.3.1-callsite-diagnostic`, APK
`1e50520270e5498a7ca66e334c17606a742032e8f4b3ba4c20c5af6dc7f40321`,
created an independent v1 diagnostic branch. It reproduced the Fold's first
routine-2 caller, page fields/adoption and downstream failure. Private checks
confirmed distinct profile IDs, matching ancestry, retained GameId/media and
seven unchanged baseline files. The new safe-area geometry showed complete
circles on the Pixel as well. The optional callsite request was created only
after stopping emulation, then removed after collecting its bounded output.
The phone returned booted, USB connected, xbx stopped and physical screen OFF;
no passcode-lock claim, settings changes or owned clients/tunnels remained.
Fold was not queried or operated during this continuation.

Private callsite inspection narrows the observed error to process/heap setup,
drive-link setup, or main returning. It does not yet select which occurred and
does not justify suppressing the error or adding a dashboard.

## Startup trace extension, version 2

The current patch adds the already-existing VM allocation failure counters,
last drive-link result/caller, and at most eight return addresses at each of the
first four firmware returns. Stack walking checks alignment, monotonically
increasing frames, current-thread stack bounds and both mapped ends before
reading; it emits return PCs only, never frame arguments or memory payload.
No full import trace or RAM increase is introduced.

Current patch SHA-256 is
`667b076f48968acb195170168d771930795a4f9caa46ad3a343fc6228b2a0f94`;
built image SHA-256 is
`36cb1425e6d125df899eeb0e2761919309a639c944e6277f1599325ba0cf35d1`.
The pinned source, build flags and commands above still apply. The prior v1
patch/image/APKs remain available at b654b6d and in retained private candidates.
Version 5 / `0.3.2-startup-trace` assets explicitly select a new fixed
`xbx-launch-diagnostic-v2` root, cloned coherently from the stopped baseline.
It cannot substitute firmware beneath v1 or the baseline. Both known modes
have distinct roots; unknown or duplicate modes refuse provisioning.

The extended firmware passed the two-stage native relaunch boundary. Its first
desktop teardown acknowledged QMP but ended with SIGSEGV after a GLib source
reference assertion. The bounded debugger follow-up passed both the title
boundary and normal shutdown; the intermittent cleanup defect is preserved and
is not declared fixed. No core dump was stored. Android's separate process-stop
lifetime remains distinct from clean desktop QMP shutdown.
