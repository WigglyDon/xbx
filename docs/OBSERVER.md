# Bounded public-kernel contract observation

Master [6356462260](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6356462260)
authorizes this finite observation, original guest validation and one matched
ordinary Fable run. The accepted FPU correction and completed loader/thread/
reference-core evidence remain intact. Private game-code analysis remains
paused. No game-specific patch, new exception interception, dispatcher change,
caller-address trace, stack walk, output-buffer/path capture or game-code read
is added by this observer.

## Event and capture semantics

The criterion is the first existing observation after normal title entry with
code `c0000005` and origin14. The observer is armed at the existing entry
dispatch point, and annotated inside the retained eight-event diagnostic budget.
It records the existing observation index, global event order, access kind,
opaque current-thread label and call-token cutoff. It is a **first observed
qualifying event**, which may be handled; fatality requires separate terminal
evidence. Exhausting the existing budget produces explicit unavailability.
The old private fault identity remains in the existing diagnostics, outside the
new exported schema. No new raw address is exported.

The ring contains at most1,024 calls. Entry and observed function return each
receive a global order; every recorded call has a monotonically increasing
token. A long call's return can update only a row with its exact token. An
overwritten row cannot acquire that old completion. Calls without available
return tracking, and calls whose thread exits without returning, are explicit
unavailable states. An entry still active at cutoff is pending-at-cutoff.

Thread labels are run-local monotonic IDs with birth/retirement orders. They
are separate from reusable tracer bookkeeping slots. Actual thread exit retires
the label; a reused kernel thread pointer receives a new ID. Ledger flags1/2
mean an old retired pointer was reused / this new lifetime reused one. These
flags derive only from the internal kernel-owned registry; no pointer values
are exported. The registry has128 lifetime rows, never recycles IDs, and reports
unknown/overflow explicitly. Current execution identity does not establish an
asynchronous request's originating thread. This is cold-boot diagnostic state,
not a durable GameId or SavedSession identity.

At the first selected event the header/window/lifetime ledger freeze together.
Later entries and returns keep mandatory trampoline bookkeeping but cannot
change captured rows or their results. Separate post-capture health counters
report entries, returns, exits, last entry/return ordinal, unavailable tracking
and completion of calls that began before capture. Those counters are **not**
part of the captured pre-fault sequence. No allocation, blocking I/O or unbounded
scan occurs at the event point.

## Fixed schema and types

The export is one fixed kernel-owned structure:32 header words,128 four-word
lifetime rows,1,024 eight-word call rows, totaling **34,944 bytes**. A compiled
static assertion includes internal pointer/return bookkeeping, target/ordinal
tables and maximum thunk storage: **56,976 bytes**, below the64KiB total budget.
Call rows are `{token, entry_order, return_order, thread, ordinal, kind, value,
flags}`. Flags1/2 mean tracking unavailable / thread exited without a return;
flag4 is nonnull pointer four-byte alignment. Header counters include overwritten
records, omitted calls, label overflow, unavailable returns and late completions.
Import-storage/order failure is explicit capture failure, not silent coverage.

Supported result kinds are deliberately small:

| Kind | Public ABI ordinals | Exported result |
| --- | --- | --- |
| NTSTATUS |184–231,233–238,254–257,139,142,159,181 | Actual32-bit status;103 is explicitly STATUS_PENDING. |
| Pointer |14,15,165–169,177 | Null, low, user-range, direct-range, system-range or other-range; optional alignment flag. |
| Void |17,108,258,284,285,289–291,298,320 | No result; EAX is discarded. |
| Unclassified | Other recorded functions | No result; EAX is discarded. |

These definitions use pinned public export declarations. Range categories are
numeric membership, not proof of mapped storage, allocation extent, ownership
or a satisfied pointer contract. No argument pointer is dereferenced or output
buffer copied. A function return, including STATUS_PENDING or success, is not
proof that asynchronous work completed. Current page-fault source supplies
read/write access kind; it does not independently distinguish instruction fetch.

The retained frequent exclusions remain103,104,119,125,126,128,129,160,161,221,
234,238,277,278,294,295. Their calls are counted as omitted where observed.
Ordinal232 remains completely unwrapped because the I/O manager requires its
exact callback identity. Data exports are not wrapped. Absence from this window
does not prove an API was never called.

Rust binds source/full patch/flags/firmware/map before launch. After acknowledged
QMP stop it reads only the bound symbol's fixed storage, with bounded kernel
range checks and a second header stability check. It rejects malformed schema,
cutoffs, token reuse, impossible completion/lifetime ordering, untyped values,
missing rows and unavailable capture. Partial/raw captures and rejection reasons
remain private. Capture rejection still permits normal quit/reap/storage sync;
it fails observation acceptance without prematurely dropping the writer lease.

## Original guest validation

`native/fixtures/observer.S` is original MIT mechanical guest ABI code. Rust
owns its build, capture and acceptance. It uses the existing synthetic fixture's
public SEH CONTEXT layout, without changing kernel handling. The guest checks
ordinal232 identity, stdcall/fastcall stack balance, EBX/ESI/EDI/EBP, bounded
synthetic pool/fill/read operations and canaries.

A child atomically signals readiness then blocks in a real wait. The parent
executes1,100 recordable calls, overwriting that wait's ring row, and releases
and joins the child. The late completion is counted without changing a newer
completed call. A second child exits; the kernel reuses its predecessor's thread
pointer but assigns a distinct lifetime label. During a third create call, the
registered original callback performs one deterministic synthetic data-read
fault through unchanged dispatch. Its original handler resumes at the known
following fixture label. The outer call then returns normally. Another1,100
calls and a success-only final marker prove continued bookkeeping and retained
capture. Failure paths use a different final ordinal.

The initial fixture/build/capture remain retained. Its early type samples were
correctly overwritten before the event; the completed revision retains samples
near the event and exposes opaque pointer-reuse flags. No accepted earlier
allocator, relaunch, loader, thread/TLS, FPU or reference-core campaign was rerun.

Actual completed validation in fresh `.local/observer-fixture-v2`:

- First existing event1: code c0000005/origin14/read, thread label1, global
  order2247, call cutoff1122;1,024 retained/98 overwritten/1 omitted.
- Three lifetime labels, two retired. Rows2/3 have reuse flags1/2, proving the
  same underlying kernel pointer did not join those lifetimes.
- One late completion. All newer bulk-call completions precede the release
  event; the old completion did not rewrite them. The outer create call is
  still pending in the captured fault-thread window.
- Two expected nonreturning thread-exit calls are unavailable; no label overflow
  or post-capture unavailable tracking. Actual pointer/void/NTSTATUS/unclassified
  samples have their declared representation.
- After capture:1,104 entries and1,104 returns, one exit, one pre-cutoff call
  completion and final entry/return ordinal307. Guest acceptance passed.
- Normal QMP stop/quit acknowledged, native exit0, forced=false, capture valid;
  lease held through actual reaping then storage sync. No unexpected bugcheck.

Host controls reject typed-value leaks, incomplete capture, exhausted budget,
late/post-cutoff completion, duplicate tokens and joined/recycled lifetime IDs.
STATUS_PENDING as an actual function return remains distinct from an in-flight
call and from completion of asynchronous work. All27 workspace tests pass,
three environment-gated checks remain ignored; Clippy-Dwarnings/fmt/diff pass.

## Source/build identity and reproduction

OXOS `d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2`, Cromwell
`86f547387184d1001d377cce97d3756c8acf91cc`. The complete composed observer patch
is `0be0b4b91460a748ad4a1cbc57793a57b3d0895524f8f71b7fbd1f9dcc7c6626`,
bound by `kernel-observer.lock.json`, with the unchanged v4/loader/journal/npx
base hashes. Beyond accepted npx, only retained trace.cpp and annotations in
loader.cpp/except.cpp differ. Targeted retained code keeps GPL/per-file terms;
new Rust policy and original fixture are MIT. No public binary release.

Firmware `b2a6bc6a63d6bc1db4ab76f216a432144acca7f10aa538269b32416ad0426bc5`;
map `6d75052c882092a5decb413eb3b8b336a83c0cc544abadc6eff227c899493963`;
build.identity `31d02c07e70b2dada74726a47e2fa2e2e026145f231e2b879aeb8781cb26585f`.
Executed supervisor `77da61a202d6b0560f9c8e3461f18926ad19a5fbe0b839c0939f7a0d44d2a69e`;
retained core `51178df74f0b26316c799ff8e8c1a65b1b841b97858cd07dd31903d52ebdcd0d`.
Fixture source `04e3a64b5948b3f9f6e6c5308c49e4069043c66610383c104725aff605a1196f`;
XBE `bffca45ff7a311049a831c0cffba8bf9f46049fcab4fbd597178111a5cf37a3d`;
ISO `648ae4ac959ac20a68b5ebd004550c78a63b82671f997a3af122be528b0c9c30`.

Builds use one compiler job, open image/SELFTEST0/SERIAL1/TRACE_IMPORTS1/launch
trace1/64MiB and no proprietary boot/splash inputs. The separate sparse observer
source begins at the exact pin plus accepted npx composition. Full diff and
Cromwell are verified before compilation. Failed/intermediate attempts and
date/time-dependent build identities are retained; no old image is overwritten.
Capacity was rechecked after the interruption:31,591MiB free/1,327MiB usable
Btrfs metadata, emergency reserve unused, no owned build/native job running.

```sh
# In a separate same-pin sparse source, apply patches/oxos-observer.patch.
XBX_NDK=/path/to/read-only/ndk/30.0.16248370 cargo xtask kernel-build observer
XBX_NDK=/path/to/read-only/ndk/30.0.16248370 cargo xtask observer-fixture-build
cargo build --offline -j1 -p xbx-host
# CORE/FLASH/MAP/HDD/EEPROM identify the exact retained builds and generated
# templates above. Every ROOT is new. Firmware and MAP must share build.identity.
G_DEBUG=fatal-criticals target/debug/xbx-host run-title-journal "$CORE" "$ROOT" "$FLASH" "$HDD" "$EEPROM" .local/build-observer-fixture-v2/observer.iso 30 "$MAP"
target/debug/xbx-host check-observer-fixture "$ROOT/observer-raw.bin"
XBX_DESKTOP_VIDEO_DRIVER=x11 G_DEBUG=fatal-criticals target/debug/xbx-host run-title-journal "$CORE" "$GAME_ROOT" "$FLASH" "$HDD" "$EEPROM" "$AUTHORIZED_MEDIA" 300 "$MAP"
```

Native termination, actual gameplay/audio, ordinary saves, full resume, shortcuts
and mobile performance remain separate acceptance boundaries. Neither a passed
fixture nor a captured last call establishes Fable compatibility or causality.

## Single matched Fable observation

Fresh `.local/fable-observer-v1` used the identities above, retained64MiB core,
same generated disk/EEPROM templates, verified unchanged read-only disc view,
actually attached ControllerS, X11/OpenGL/software Mesa and dummy audio. Required
headers8760/8760, raw8328772/8328772, extents16/16, short0/I/O failure0;
preloads15/15, match15/mismatch0/unusable0. Loading and entry succeeded.

The first existing event was a read access violation: observation index1,
global order32242, opaque thread2, cutoff2124. The retained1,024 calls include
801 from that thread, all with observed function returns. There were1,100
overwritten entries,27,991 omitted calls,
eight labeled lifetimes, three retired, zero label overflow and zero late
completions. Two thread-exit calls had unavailable returns. The first-event
window remained valid; one later entry/return was recorded only in separate
post-capture health. Those health counters are not appended to the old window.

The fault thread's recent sequence consists of successful semaphore creation,
returned thread creation and void critical-section initialization; its last
recorded call is RtlInitializeCriticalSection. Void results supply no value.
All recorded pointer results in the entire window are nonnull. Recorded
non-success NTSTATUS values are invalid-parameter from NtQueryVirtualMemory,
missing-name/path queries and flush access-denied results. Without their input
and object contracts these are not demonstrated defects. The final call and
temporal proximity alone do not establish causality.

Existing private numeric PC/fault/code/origin equal the accepted npx run. One
exception was recorded; the closing bugcheck0x1e's existing exception/access/
PC/fault parameters match that first event. Thus the **separate terminal
evidence** identifies this particular observed event as unhandled, while the
observer criterion itself remains first-observed. The owned-window image was
opened and showed the open-kernel splash. Terminal startup failure occurred at
83.125seconds; no accepted Fable menu/gameplay. This duration is not a gameplay
performance benchmark. QMP stop/quit acknowledged, native exit0/forced=false,
capture valid, lease through actual reap then storage_synced=true.

The observation is inconclusive about a specific violated public-platform
contract. No repair or second game run is justified by these results. The
smallest missing evidence is the relevant object's expected public input/output
contract at the faulting consumer: a status or pointer range does not identify
that object or verify its contents. Older/omitted calls and asynchronous origin
remain explicit limitations. Private game-code analysis remains paused; no
unrelated defect hunt or additional observation campaign is undertaken.

No phone was operated or queried; prior returns remain settled without claiming
a fresh screen-state check. Both phone media copies, previous profiles, firmware
generations, Android selectors and private evidence remain untouched. The full
host media hash and55 retained artifact identities match before and after launch.
Gameplay/audio, ordinary saves, full resume, shortcuts, mobile performance and
owner acceptance remain unqualified. PSX and other projects are untouched.
