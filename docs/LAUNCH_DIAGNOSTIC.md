# First quick-reboot diagnostic

These versioned sections preserve historical receipts and commands for OXOS
`c0c887a1263ebf89ba765f34a88ca8f132901e25`. The current separate desktop pin,
build policy and successful focused upstream correction are recorded in
[QUICK_REBOOT.md](QUICK_REBOOT.md). Current firmware builds do not update the
Android selectors; new candidates cannot be labeled as baseline/v4 assets.

Master continuation 6345094692 authorizes this bounded follow-up to the Fold
failure at 6612ba2. It does not select a shipping kernel or authorize a public
binary. Baseline firmware, failed profile and both phone media copies remain
preserved. No PC game image is required.

## Source and build binding

At the time of these historical runs, `kernel-trace.lock.json` pinned OXOS/Cromwell and the exact
`patches/oxos-launch-trace.patch`. From a clean pinned OXOS working tree, apply
that patch once. The kernel build refuses any other tracked diff or untracked
source. The old trace variant used its own directory keyed by patch digest, wrote
an image/build receipt, and retained the prior self-test and normal-play builds.
That directory scheme is superseded; do not use this historical Android build
sequence to adopt the new desktop-only candidate.

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
was not declared fixed by those debugger passes. No core dump was stored.

## Scoped native cleanup correction

A bounded critical-handler backtrace identified `io_remove_watch_poll` during
socket chardev finalization after monitor cleanup. The retained watch returned a
borrowed pointer after dropping its initial source reference; its owning context
could release that final reference before the chardev used the pointer. Retain
the initial reference until removal, clear the chardev pointer before finalizing,
then destroy/unref the source. The native patch and original file notices retain
their applicable terms.

After that repair, two non-debugger runs still ended with SIGSEGV. A narrow
stderr marker established that QEMU cleanup had already completed before the
crash in process exit. The donor's desktop QEMU thread calls C-library `exit`
while its frontend render thread remains alive; this can race frontend atexit
handlers/global destructors. The Rust diagnostic supervisor now explicitly sets
`XBX_ISOLATED_PROCESS_EXIT=1` for its owned child. Only that opt-in invokes
immediate process exit after QEMU cleanup closes block storage and stdio is
flushed. Ordinary donor launches retain their existing exit path. Rust retains
the inherited storage lease through actual process termination and then syncs
storage; this is process isolation, not in-process teardown/reinitialization.
Android's existing separate-process exit policy stays separate.

With both changes, two bounded native two-stage relaunch regressions under
`G_DEBUG=fatal-criticals` completed acknowledged QMP stop/quit, cleanup and status
0, without forced kill or a GLib critical. Earlier failures and intermediate
artifacts remain retained. This is evidence for the exercised cleanup path,
not arbitrary fatal-exit, gameplay, save or snapshot acceptance. The current
xemu patch digest is
`26027ec1436fb6c5f7a51200bf7fe96ceb90ba5695d568a72db29ef446ff461f`;
the source pin and native ABI remain unchanged. Version 6 packages this narrow
watch correction alongside the same version-2 trace firmware/profile selection.

Prepared Android version 6 / `0.3.3-watch-lifetime`, application code 47fc9e8:
APK SHA-256 `fa9b0b9c32997137cb386a1d731258719b97ff620d961ba4a70c96cc88a7f077`,
native library `ff9b8d0ccfd81219f3b034c87e8aea34fedc9859e1c662dc78a8177ceff1be29`,
Rust library `fa8b451dec7884a872d6efb1874f54edb80cf311a480999fbbd6012b60ee52a6`.
Firmware remains the version-2 image above. Signature and 16 KiB ZIP alignment
checks pass. Installation/execution is pending: another worker acquired the
Pixel after its successful first return, and the physical-device guard refused
the follow-up acquisition without phone actions. The prepared candidate is not
a new device acceptance result. Fold stays released.

Final workspace checks pass (17 tests, three explicit native fixtures ignored
by default), including a bounded post-reap lease check: another concurrently
forking fixture may briefly retain a pre-exec descriptor, so actual OS lease
acquisition proves release rather than inferring it from one reaped PID. The
previous immediate-acquisition assertion failed once and remains in private
failure evidence. Host and Android Clippy pass with warnings denied.

## Pixel startup localization and landscape candidate

After the other product's verified handback, version 6 was installed and run on
the qualified stock Pixel 4 under a finite exclusive transaction. The independent
v2 profile reached Fable's title thread and reproduced the dashboard-error return.
All seven baseline fingerprints and all seven v1 diagnostic fingerprints remained
unchanged. The return-frame chain plus bounded private executable inspection
localizes that return to process-heap creation returning null, before the title's
drive-link setup or main function. No VM allocation failure was recorded; that
does not establish that no allocation was attempted. The kernel's existing
exception globals describe unhandled exceptions only, so an empty report does
not exclude a caught setup exception. The actual underlying failure is unresolved.

A second bounded read collected only the next 12 KiB executable range needed for
heap setup and numeric import mappings. It ran before native machine entry; all
seven v2 profile fingerprints remained unchanged. Both transactions stopped xbx
and verified physical screen OFF before releasing custody. No security-lock claim,
gameplay, audible audio, ordinary-save, snapshot or performance result is earned.
Raw executable fragments, logs and profile identities remain private.

Version 7 / `0.4.0-xbox-landscape` builds the owner's landscape/original-controller
request. Public PSX Rust side-column/clear-picture geometry is pinned and noticed
in THIRD_PARTY.md; Xbox channels and data policy stay separate. Rust owns geometry,
labels, ARGB colors, label sizing and pressed outlines; Android forwards Canvas,
safe insets and sensor-landscape orientation mechanically. Portrait/square safe
geometry remains available for resized windows. No device rotation setting changes.

APK SHA-256 `cd2ea82cde686f439207060960ce4240c71e8664bb924e4c6027084a6105bbbe`;
Rust library `7330ca521d6367c9fb6fe8926890d9368f7f2c1dfb328fa38c3e4c5eeba59a35`.
Core and v2 firmware hashes remain as above. Reproduce with the existing selected
v2 profile assets and read-only SDK/NDK:

```sh
cargo test --workspace --offline -j1
cargo clippy --workspace --all-targets --offline -j1 -- -D warnings
XBX_NATIVE_LIB_DIR="$PWD/.local/build-android" cargo clippy -p xbx-host --lib --target aarch64-linux-android --offline -j1 -- -D warnings
cargo xtask android-host
cargo xtask android-apk
```

Workspace checks pass: 18 tests, three explicit native fixtures ignored. The new
boundary test checks every landscape hit/channel, cancellation, whole-circle safe
bounds, nonoverlapping controls and a clear aspect-correct picture across inset
arrangements and a transient resize. Host and Android Clippy pass with warnings
denied. APK build/signature/alignment pass. Version 7 was compatibly installed on
the qualified Pixel with the predecessor retained and signer verified equal.
Visual inspection of the xbx screen confirms automatic landscape, all 18 complete
controls, readable original-Xbox labels/colors and a clear center. The app was
stopped during media checking after this layout check; this is not an additional
native boot/gameplay result. Baseline and v1 each retained seven fingerprints.
xbx stopped, physical screen OFF, no owned clients/tunnels or changed settings;
custody released. Prior APKs, firmware and unique failure evidence are retained.

## Bounded heap diagnostic, version 3

The observed heap-null result warrants two small retained-kernel diagnostics:
up to eight VM allocation call/results and eight exception-dispatch events per
kernel boot. The latter includes caught exceptions omitted by the earlier
unhandled-exception globals. Only numeric call/status/range and fault metadata
are emitted; no memory payload, registers, strings or full import trace. This
does not fix the heap failure or establish its cause before a Fable retest.

Patch SHA-256 `98ffc66a54006e4e11455d8cbcc3ea76c9f84b2d4fd379ae8fccb0f603081fa4`;
actual firmware `6ae311f148b50dbccef61657aadbb7f425f3c5447783dfbdb20ffacbebadf271`.
Pins, IMAGE=open/SELFTEST=0/SERIAL=1/TRACE_IMPORTS=0/64 MiB flags and build commands
remain as above. The exact patch/build.identity and actual image are retained;
date/time remain a clean-rebuild variation. A first compilation caught missing
serial declarations in the two new translation units; explicit includes fixed
the build. No broad warning suppression was added.

Rust selects a new fixed `xbx-launch-diagnostic-v3` root, cloning the stopped
baseline coherently under the existing lease, with distinct profile identity and
ancestry. v1/v2/baseline remain recognized and cannot alias this root; their used
firmware/session bindings are not changed. The existing mode-boundary regression
now checks v3 separation too. Full-machine restore remains disabled.

The focused two-stage homebrew regression passed on this exact firmware with the
retained desktop core under `G_DEBUG=fatal-criticals`: first/second entry, carried
launch data and D: remap, acknowledged QMP stop/quit, native status 0, forced=false.
The new VM record was exercised; no exception occurred in that fixture. It is
not a Fable or mobile acceptance test. Private synthetic scratch and output are
retained separately from owner profiles.

Built version 8 / `0.4.1-heap-diagnostic`, same package/signer/ABI3/native core:
APK `3d0cb7f6232f91337af9f892f3a602d3d5ac3e21af9012b583ed8dfdbb6c98ff`,
Rust library `1ae1074d12fed6ea9fb10f0664b2cf5a1b62d48eb76f5a5c332c969467503266`.
Workspace 18 tests and both Clippy checks pass; signature/alignment pass and
signer matches v7. This candidate has not been installed or executed on a phone.
Latest shared-device coordination reports another operator after XBX's verified
return, so no further acquisition or queued device action was attempted. Version
7 remains the installed Pixel layout candidate. Its actual return was posted on
XBX-1 and, with the owner's explicit approval, cross-linked at the existing Pixel
owner. No device is held while awaiting an admitted finite test opportunity.

## Master-directed v8 execution

Master continuation 6347812581 approved using the already-built v8 candidate
unchanged. After reconciling the previous operator's actual terminal return,
one finite Pixel transaction installed that exact APK, cloned a separate v3
profile and retained the read-only game grant. The first failure sequence contains
one VM result (the loader's successful image reservation), then one access
violation with PC and fault address both zero, followed by the same heap-null
error return. Both eight-record budgets retain headroom. There is no heap VM
return record; because logging is after allocator return, this cannot exclude a
caught fault inside that call. The exception record precedes handler completion.
A zero-PC dispatch suggests a null-call path, but does not yet identify its
caller, thread or the handler outcome. No compatibility repair is claimed.

The app was stopped promptly after capturing the first return. All seven baseline,
seven v1 and seven v2 fingerprints match before/after. v3 ancestry matches all
parent hashes; its distinct profile ID matches its SavedSession, with retained
GameId/media/partition identity and the exact v8 firmware/64 MiB binding.
Pixel returned xbx stopped, physical screen OFF, no owned clients/tunnels or
changed settings/pending recovery; custody released, no security-lock claim.
No Fold operation or new media copy. Raw code, addresses and logs remain private.

The source-built export directory also maps ordinal 184 to the actual
NtAllocateVirtualMemory body, matching its linker map; this does not establish
what the title's runtime thunk contains at the fault.

To resolve this specific ambiguity, v4 adds CPU-vector versus software-raise
origin and current-thread identity to the same bounded dispatch record. Only for
zero saved EIP, it reads one candidate return PC at saved ESP, after checking
alignment, overflow, current-thread stack bounds and both mapped ends. This is
a candidate until correlated with an actual call instruction; no frame arguments
or arbitrary memory payload are dumped. Exception dispatch/unwind behavior is
unchanged. A new fixed v4 root preserves baseline/v1/v2/v3 firmware/data bindings.

Built v9 / `0.4.2-nullcall-caller`, same package/signer/ABI3/native core:
APK `76739a8e348338ee50c024c04ace4fad58b541ab07db5ff9b03ec8fa2dc19e76`,
Rust library `fd275abad352478b6c963d4276f69ed87f77e2d8fafccf10773144d46f163e1f`.
Patch `7eb48fb10ca5784a887d93db23f02c07ec83dc7b105930e1a71204724e0a0dc5`;
firmware `0cdedf6d81dabdbeafc1cff78039cf78f19cb7923078f201724e346ca94ee8e8`.
Pins/64 MiB/build flags remain unchanged; actual build.identity/images retained.
Signature/alignment and predecessor signer checks pass. The same focused native
relaunch boundary passed on this final firmware with normal status 0/QMP shutdown,
forced=false. A shared-device custody check found another operator, so no v9
phone acquisition was attempted at that point.

## Original handled-null-call fixture

While the phone was occupied, a small original mechanical i386 shim was added at
`native/fixtures/nullcall.S`. Rust owns its build and bounded runner; there is no
new application layer or kernel implementation. It makes a real indirect call to
zero, registers a handler that checks the access-violation/null-PC context and
actual pushed return address, then resumes and publishes guest success flags.
It has no SDK, game code, imports, writable disk operation or injected QMP result.
The assembly declares its COFF feature and SafeSEH handler metadata explicitly.

```sh
cargo xtask nullcall-build
cargo run -p xbx-host --offline -- boot-nullcall \
  .local/build-desktop/qemu-system-i386 /new/short/nullcall-scratch \
  /selected/build/bios.bin /isolated/seed/hdd.qcow2 \
  .local/build-nullcall-fixture-v2/nullcall.iso \
  .local/build-nullcall-fixture-v2/title.map
```

Use the existing read-only NDK for the build, as for other xtask commands. Building
refuses an existing fixture directory; use retained successful outputs for a new
run instead of replacing build/failure evidence. The pinned OXOS format tools
wrap the generated PE/XBE/disc; no upstream test campaign/default HDD is invoked.

On the exact v4 firmware and retained desktop core under `G_DEBUG=fatal-criticals`,
the guest handled and resumed from the null call, reported all three expected
flags, and generated exactly one exception event: CPU vector 14, null PC/fault,
valid saved stack, candidate return PC matching the fixture's known return symbol.
QMP stop/quit acknowledged, native status 0, forced=false, storage lease held
through reaping/sync. This validates correlation for that synthetic contract;
it does not identify Fable's caller or prove its handler outcome. Build failures
and earlier diagnostic variants remain retained privately.
