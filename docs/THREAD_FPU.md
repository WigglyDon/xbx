# Loaded-title thread startup and displaced FPU owner

This work follows the owner's continuing autonomous public-platform repair
direction after the separate upstream comparison in [REFERENCE_CORE.md](REFERENCE_CORE.md).
Private Fable game-code/disassembly analysis remains paused. No game-specific
patch, exception campaign, phone transaction or Android build is involved.
These are original finite ABI fixtures, not gameplay acceptance or a new SDK.

## Thread/TLS boundary

The kernel's retained self-test already covers internal thread/TLS creation.
The retained relaunch title uses the simpler PsCreateSystemThread wrapper.
Neither establishes this exact loader-resolved ten-argument PsCreateSystemThreadEx
call, supplied startup callback, suspension and public caller-owned TLS path.
Successful thread-create statuses in the bounded ordinary-game journal alone
also do not establish those results.

Original MIT `native/fixtures/thread-startup.S` creates two sequential threads,
one initially suspended, using synthetic sizes/patterns. Guest checks:

- Exact supplied system/start/context arguments, nonzero handle/ID, suspended
  child not yet running, resume previous count1, bounded wait and successful close.
- Parent stack-pointer and preserved EBX/ESI/EDI/EBP after every checked API.
- KeGetCurrentThread and PCR identity, TLS within stack bounds, PCR bounds,
  all36 TLS bytes initially zero and TLS payload aligned16.
- This guest's startup initializes its own self-reference and bounded32-byte
  pattern; actual callback entry/return, negative-DWORD TLS-index lookup and all
  written values agree after callback execution. The kernel is not made to copy
  an application TLS template.
- Both children terminate normally, then the parent emits a final sentinel only
  after all checks and both waits/handle closes succeed.

Public ABI facts were checked against OXOSd87 and nxdk
a6c31ee90540710a3df26a0712b61492f2ea03dd (`xboxkrnl.h`, `thread.c`). No copied
implementation or private title instructions. A heuristic source audit found no
convention/argument-size mismatch in324 comparable public export prototypes;
that source audit is separate from actual guest ABI execution.

Actual first run, `.local/thread-startup-v1`, passed both guest sequences under
retained journal-v2 firmwareced722fe94acd08a81a15c1e45b44b0f9b948033c8e045f1746651090ea0ad53,
retained desktop core51178df74f0b26316c799ff8e8c1a65b1b841b97858cd07dd31903d52ebdcd0d,
64MiB and independent scratch disk/EEPROM. Ten journal entries end in
ordinal307/EAX01534c54. Both creates/resume/waits/closes returned0 and both
PsTerminateSystemThread entries correctly never returned. Frequent ordinal104
is intentionally omitted by the retained journal; the guest checks its actual
results. The initial host acceptance incorrectly expected two104 records and
refused the otherwise successful result. Corrected the expectation and
validated the retained result, with no repeated guest run.

Executed supervisor SHA25638ce509b9f266a0a5456de766a098aca29a90ae198ecd90a1764d86151d79812;
corrected acceptance-only supervisor1c67bfefa609fcee8dec2c30d3098ba283d71733c9f4cdcb230f1f656c4c131c.
Guest sourcec9e756a701b39bcdd74ff1d5544f00896baebfcf26932101fca088ff6cb3d95d;
XBEe9e27d2db373eec547eba282319451adccc6d947ea534ab274d23e218b5e4a18.
No unexpected exception/bugcheck. QMP stop/quit acknowledged; native exit0,
forced=false, lease through reaping then storage sync. The observation budget
ends the fixture's deliberate final HLT loop; completion comes from the guest
sentinel, not from the timeout or a screenshot.

## Source-supported FPU defect and original before/after proof

Review of the existing lazy FPU machinery found a separate concrete defect:
`KeRestoreFloatingPointState` restored the caller's saved register file without
saving a different thread that still owned the live FPU. A caller can resume
after a context switch and enter this service without any FP instruction, so
CR0.TS does not itself preserve that owner's state. The displaced thread was
also left marked loaded. A subsequent #NM could return without restoring it.
This is a public-source finding, not attribution of Fable's exception.

Original MIT `native/fixtures/npx-ownership.S` reproduces the exact transition:

1. Parent loads synthetic x87 value1.25 and one128-bit SSE pattern, then calls
   the real loader-resolved KeSaveFloatingPointState.
2. Child loads value2.5 and a distinct SSE pattern, signals readiness and waits.
3. Parent resumes without FP use, restores its saved state and verifies its
   x87/SSE values, then signals the child and waits for its termination.
4. Child verifies its own x87/SSE values after the switch, emits its result and
   terminates. Parent emits a final success only after child success and close.

On the retained journal-v2 firmware, `.local/npx-ownership-old-v1`, all relevant
API statuses were0. The child nevertheless produced failurebad00302 and parent
failurebad00205, encoded by ordinal307 as0203d0ba and0502d0ba. Actual displaced
FP-state check failure was reproduced; that failure stage groups the x87/SSE
comparisons and does not separately identify which values were lost.
There was no unexpected exception/bugcheck. Native acknowledged exit0,
forced=false and storage sync. Executed supervisor1c67bfef above is retained.

The correction adds seven lines to retained `kernel/src/ke/npx.cpp`: while
interrupts remain disabled, FXSAVE a different current owner into its own save
area and mark it not loaded before restoring this caller. It uses the same
ownership transition already used by the existing #NM/save paths. No exported
ABI, thread scheduler, exception handling, game code or RAM size changes.
Kernel work retains GPL/per-file terms; original build/acceptance policy is Rust.

The identical fixture on the separately built candidate,
`.local/npx-ownership-new-v1`, produced child600dcafe and final parent4e505801
(recorded asfeca0d60 and0158504e). Both x87 and all128 SSE bits survived; close
also returned0. All relevant statuses0, no unexpected exception/bugcheck,
acknowledged native exit0/forced=false/storage sync. Same core and64MiB, each
test with fresh independent synthetic storage; no prior profile reopened.

Fixture source70d6e0a45c60dcc9b2b0b524c3dd6f8a07d5558c78cc31074db90e00bf278591;
XBEd634b1e22f7383c73e5f8dce294b8fabaebf11e14fc5ee5b17adb328f088f04b;
ISOc3cf182c7d53a0d3782813fa572d88755570bc2bb8c6bd71cc02b0833b8df2f1.
Candidate supervisor93deebb0cf8262c052f0dc102884777766f1c3cea4f434beb4864662056b5aed,
retained privately. This regression qualifies only the exercised transition.

## Exact separate source/build

OXOSd87cfc33ceb493711b545e83d5b7fd32f1cdf0c2 and Cromwell
86f547387184d1001d377cce97d3756c8acf91cc remain pinned. Full composed
patch712f233e8713d6934c4aa65cc5a460caea98072c7b457d8313d00c07e901fdbc,
`patches/oxos-npx-owner.patch` / `kernel-npx.lock.json`, includes unchanged v4,
complete-read loader and callback-preserving journal-v2 bases. Comparing all136
available retained tracked source files found only npx.cpp different beyond
journal-v2. Old source trees, patches and outputs remain unchanged.

New sparse source is `.local/src/oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2-npx-owner-v2`.
An initial local shared-object copy had missing unused partial-clone blobs; it
is retained. The separate v2 copy uses the retained sparse paths. Git emitted
unused .vscode blob warnings there too, but actual HEAD, complete diff, required
source paths and Cromwell binding were verified before building. No missing
source is treated as a successful build input. No old worktree was repaired or
modified to prepare this candidate.

Rust build mode `npx` binds its full patch and all three base-patch hashes, uses
exclusive source/patch/mode output paths, and does not update Android selectors.
One compiler job, open image/SELFTEST0/SERIAL1/TRACE_IMPORTS1/launch trace1/64MiB,
no proprietary boot or splash inputs. Preflight34,961MiB available/1,367MiB
usable Btrfs metadata, emergency reserve unused; no other-project cleanup.

Firmware SHA256eb9b3eb164961b58ddc3acd521d238bddb7f40d8be1a32e696426f93b657d992;
mapa9a703846685cbfcb70b423849f19db9d33308e9b20605df970b6ac24a1c8f88;
build.identityce25da532b36e51799990fe34c675b37483e271ffbb88dba327e80d4b89f3dcd.
Date/time-dependent donor generation means the retained receipt binds the
actual tested image; a future rebuild need not have the same binary digest.

```sh
XBX_NDK=/path/to/read-only/ndk/30.0.16248370 cargo xtask thread-fixture-build
XBX_NDK=/path/to/read-only/ndk/30.0.16248370 cargo xtask npx-fixture-build
# Prepare the separate pinned sparse source and apply the full composed patch.
XBX_NDK=/path/to/read-only/ndk/30.0.16248370 cargo xtask kernel-build npx
cargo build --offline -p xbx-host
# Each ROOT must be new; CORE is the retained desktop engine; HDD/EEPROM are
# the generated templates. MAP must be adjacent to the matching build.identity.
G_DEBUG=fatal-criticals target/debug/xbx-host run-title-journal "$CORE" "$ROOT" "$FLASH" "$HDD" "$EEPROM" .local/build-thread-startup-fixture-v1/thread.iso 30 "$MAP"
target/debug/xbx-host check-thread-fixture "$ROOT/api-journal.txt"
G_DEBUG=fatal-criticals target/debug/xbx-host run-title-journal "$CORE" "$NEW_ROOT" "$FLASH" "$HDD" "$EEPROM" .local/build-npx-ownership-fixture-v1/npx.iso 30 "$MAP"
target/debug/xbx-host check-npx-fixture "$NEW_ROOT/api-journal.txt" preserved
# Before correction, use the retained journal-v2 firmware/map and a different
# root; check-npx-fixture ... displaced-owner requires the observed old failure.
```

Rust acceptance requires the guest's final checked sentinels as well as API
outcomes. A regression refuses all-success statuses with a missing/incorrect
guest completion or missing child termination. Native termination remains a
separate required receipt, not an inference from these sentinels.

## Actual unchanged Fable result and limits

One new `.local/fable-npx-owner-v1` launch used the corrected image, retained
core,64MiB, same generated templates/read-only media view, X11/OpenGL/software
Mesa/dummy audio and actually attached ControllerS. Header8760/8760,
raw8328772/8328772, extents16/16, short0/I/O failures0; preloads15/15,
match15/mismatch0/unusable0. Entry dispatch succeeded, followed by the same
nonzero-PC/null-adjacent c0000005 and bugcheck0x1e at74.754seconds. Existing
numeric PC/fault/code/origin records equal the preceding journal-v2 run.
The exact owned-window capture was opened and showed the open-kernel splash.
No menu/gameplay acceptance. Stop/quit acknowledged, native exit0/forced=false,
lease through reaping and storage_synced=true.

The retained recent1024-entry API window contains no ordinal139/142 calls.
It is not a complete startup history. Neither that observation nor the failed
game run erases the independently reproduced FPU defect; the repair is not
claimed as Fable's cause or a compatibility improvement. This startup time is
not an emulated-game performance result. No unchanged phone retry is justified.

Workspace checks:24 passed,3 gated ignored; Clippy-Dwarnings passes. A fixture
pass does not qualify gameplay/audio, ordinary in-game saves, full snapshot
resume, shortcuts, mobile performance or owner acceptance. All remain unearned.
Neither phone was operated or queried; previous return receipts remain settled.
No game copies, previous profiles, firmware generations, Android assets or
private evidence were deleted or replaced. PSX and other projects are untouched.

Post-run identity checks:41/41 historical artifacts and all9 bound pre-correction
firmware/fixture identities match; full selected host-media hash matches.
Retained journal-v2, accepted loader and new candidate source diffs each match
their full published patches. Private receipts/results remain under
`.local/evidence/thread-startup-v1`; captures and raw journals remain private.
Phone media was not reread merely to restate preservation.
