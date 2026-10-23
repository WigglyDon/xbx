# Public thread context and existing dispatch outcome

Continuation of [OBSERVER.md](OBSERVER.md) at public source
af9d56f80ad5efc06a4fe05918fb8205b8b41154. The owner explicitly directed this worker
to continue diagnosis and ordinary repairs toward unchanged Fable launching.
Private game-code/disassembly analysis and withheld-response recovery remain
paused. This is a small extension of the public-kernel observer, without new
exception interception or changed dispatch semantics.

An initial workspace run passed29tests/3gated ignored. A later parallel run
failed the pre-existing diagnostic branch's expected partial-copy assertion;
that failed log is retained. Serialized verification passes29tests/3gated
ignored; Clippy-Dwarnings, fmt and diff checks pass. Concurrent fork/lease
contention is a possible explanation for the parallel failure, not a diagnosed
cause. No product lease exclusion is weakened to make a test pass.

`kernel-context.lock.json` binds the complete `patches/oxos-context.patch`,
SHA256 a4cf10e026d0ca64bdb3f23b648d64547bb022d9cf91ad7570079720995e6377,
to OXOS d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2 and Cromwell
86f547387184d1001d377cce97d3756c8acf91cc. All accepted loader, callback identity,
lazy FPU-owner and frozen observer changes are retained. The additional edits
are in retained `xe/trace.cpp` and `ke/except.cpp`, under their actual terms.
Original schema, binding, capture and acceptance policy remain Rust/MIT.

## Fixed additional schema

The original observer storage/schema is unchanged. An additional fixed64-word,
256-byte kernel export (`XkContractState`, magic58425331/version1) records only
scalars. Its first-event latch uses the same event order and opaque thread label.
Per-lifetime last-argument facts use128 additional16-byte slots; complete static
observer accounting still fits the existing65536-byte limit. No allocation,
blocking I/O, game-memory read, address export or stack walk is added.

| Words | Meaning |
| --- | --- |
| 0..4 | Magic, version, latch, matching event order and thread label |
| 5 | Flags: TLS present1, within stack2, payload aligned16=4, PCR stack top8, limit16, PCR/TIB self pointers32, current thread not terminated64 |
| 6..9 | Stack pages, TLS span in DWORDs, IRQL, lazy NPX state |
| 10..26 | Active return-bookkeeping depth and at most8 token/ordinal pairs; no return addresses |
| 27..30 | Last same-lifetime ordinal291 and217 call tokens and scalar input pointer category/alignment flags |
| 31 | Ordinal102 data export was resolved (boolean) |
| 32..38 | Existing dispatcher outcome and counts of handlers, search, continue, nested/other dispositions and existing invalid-chain guard |
| 39 | Existing chain-head category: null0, sentinel1, aligned/in current stack2, other3 |
| 40..41 | Kernel available/total physical page counts |
| 42..63 | Reserved zero |

Argument flags use the observer's numeric range categories0..5 plus bit8 for
non-null four-byte alignment. No argument pointer is followed. TLS facts compare
the kernel-owned pointer/bounds only: they do not inspect TLS contents or prove
the caller's initialization. Ordinal102 resolution is not proof of later use.

Dispatcher annotations surround its existing handler calls, guard and outcomes.
They count only the selected record; an internal record pointer is cleared when
the outcome is known and never exported. Outcome0 is pending/nonlocal transfer,
1 unhandled, 2 handled, 3 noncontinuable rejection, 4 invalid disposition. No
handler, continuation or unwind behavior is changed. Rust validates the matching
base capture and the fixed schema after acknowledged QMP stop, rereads the
sidecar for stability, and retains failed captures explicitly.

## Actual original guest validation and native result

Reused the retained original observer fixture, not a rebuilt or title-derived
fixture: XBE bffca45ff7a311049a831c0cffba8bf9f46049fcab4fbd597178111a5cf37a3d,
ISO648ae4ac959ac20a68b5ebd004550c78a63b82671f997a3af122be528b0c9c30.
The first attempt failed before guest boot: the QMP inventory initially showed
only the hub, the supervisor rejected the not-yet-attached controller, and the
native child was killed. Its empty kernel log and forced-exit receipt are
preserved; it is not a guest pass.

The Rust supervisor now waits at most10seconds, bounded by the run deadline,
for actual controller attachment. It preserves initial/final inventory and
check count. A final semantic rejection sends acknowledged quit and allows
normal reaping before forced termination becomes the fallback. Lease ownership
and storage synchronization remain through actual native exit. This corrects a
demonstrated startup readiness race; it is not a Fable fix. The successful runs
attached on their first check, so they do not independently exercise delayed
attachment or the final missing-controller path.

`.local/context-fixture-v2` passed the existing guest completion/correlation,
ring reuse/lifetime and post-capture checks. Context flags120, three stack pages,
no TLS, IRQL0, active ordinal255 depth1, one handler/continue-execution, outcome2,
valid stack chain head and no invalid-chain guard. Native exit0, forced=false,
acknowledged stop/quit, capture valid, storage_synced=true; lease through reaping.
The30second budget ends the fixture's deliberate final HLT loop. The guest's
checked final sentinel establishes completion separately from native shutdown.

## Exact built and executed candidate

Build mode `context`, open image/SELFTEST0/SERIAL1/TRACE_IMPORTS1/launch trace1,
64MiB, one compiler job, no proprietary inputs or Android selector changes.

- Firmware f0e8237e22a71c892a8d910273dc3bd0dbbbc9319ff6ce91e6c162b9ea3ccbca.
- Map3188ecbab030d6cd17154d5f1a63c81bb8d9220d6c654fe2a0f9cfd5c6f91028.
- Receipt02c0e2c979c2c0715e9e944a39b60dc1e9545e605a8c899e99c9e1ea74c4294.
- Executed supervisor62ff392d6adbe63f03a085288f03369b66cd105841d1d02c873acd980e934ffc.
- Retained core51178df74f0b26316c799ff8e8c1a65b1b841b97858cd07dd31903d52ebdcd0d,
  source4967efd3ee594cc236869a45ee4cb396e25af1ac/ABI3.

The separate source tree is `.local/src/oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2-context-v1`.
The output path includes the full source/Cromwell/mode/patch identities and is
exclusively created. Prior trees, build attempts and profiles remain retained.
Preflight about31GiB available, more than1GiB usable Btrfs metadata outside the
unused emergency reserve; no other-project cleanup.

```sh
# Prepare a separate sparse source at the pinned revisions; apply the full patch.
XBX_NDK=/path/to/read-only/ndk/30.0.16248370 cargo xtask kernel-build context
cargo build --offline -p xbx-host
# Every ROOT is new. MAP is adjacent to its matching build.identity.
G_DEBUG=fatal-criticals target/debug/xbx-host run-title-journal "$CORE" "$ROOT" "$FLASH" "$HDD" "$EEPROM" .local/build-observer-fixture-v2/observer.iso 30 "$MAP"
target/debug/xbx-host check-context-fixture "$ROOT"
XBX_DESKTOP_VIDEO_DRIVER=x11 G_DEBUG=fatal-criticals target/debug/xbx-host run-title-journal "$CORE" "$NEW_ROOT" "$FLASH" "$HDD" "$EEPROM" "$READONLY_MEDIA" 120 "$MAP"
```

## Actual Fable observation and limits

`.local/fable-context-v1`: same retained core/64MiB/generated templates,
read-only validated partition view, attached ControllerS, X11/software Mesa and
dummy audio. Required headers8760/8760, raw8328772/8328772, extents16/16,
short0/I/O failure0; preloads15/15, digest match15/mismatch0/unusable0.
The existing numeric fault identity matches the previous observer run. One
nonzero-PC/null-adjacent read fault followed by bugcheck1e at117.445seconds;
the owned-window capture shows the open-kernel splash. No menu/gameplay.

First-event order35669, thread label2, cutoff2195, retained1024/overwritten1171,
omitted31276, eight labels/three retired, no label overflow or late completion.
Thread flags127: non-null in-bounds/aligned TLS, matching PCR bounds/self and
live thread; stack32pages, TLS span45DWORDs, IRQL0, NPX state0, no active call.
Last critical-section argument category10 is aligned user range; last query-VM
argument category11 is aligned physical-map/kernel range. The latter API's
public implementation rejects addresses beyond the user limit. No ordinal102
import. One existing handler returned continue-search; outcome1/unhandled,
no invalid-chain guard, aligned current-stack head. Available4095 of16384pages.
Range membership and these invariants do not establish all object contents or
successful API output contracts. The last critical-section call is not a cause.

Native stop/quit acknowledged, exit0/forced=false, valid capture and storage sync.
No phone query/operation, APK, media copy, snapshot restore or owner profile use.
All61 pre-bound retained identities match afterward. Game copies, Android assets
and private evidence remain preserved; PSX and other projects are untouched.
Gameplay/audio, ordinary in-game saves, full resume, shortcuts, mobile performance
and owner acceptance remain unqualified. Diagnosis continues rather than treating
this finite observation as product completion.

Source review also resolved the image TLS-index question: pinned nxdk's pdclib
submodule9d257524e20e3e1ac90163b37f07702053e36321
[`platform/xbox/crt0.c`](https://github.com/XboxDev/nxdk-pdclib/blob/9d257524e20e3e1ac90163b37f07702053e36321/platform/xbox/crt0.c)
initializes its own negative-DWORD image index; nxdk's thread startup initializes
its own TLS self-reference/template. No speculative kernel template/index patch
was made from this question.
