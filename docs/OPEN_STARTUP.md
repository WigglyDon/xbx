# Basic-logging open title startup

## Functional-only integration of the accepted FPU-owner repair

After the original-cartridge Android milestone at620a49b, public-source review
found a concrete integration gap: the current9d8cd58 corrected logging/release
source still contains the uncorrected KeRestoreFloatingPointState. Its source
hash is4caab333…8fd9. The independently reproduced displaced-owner repair at
fcbaeab had not been carried from the earlier composed diagnostic candidate
into this source. This is evidence for a functional port, not attribution of
Fable's post-handoff access violation.

`patches/oxos-npx-owner-functional.patch` contains exactly the seven accepted
added lines, with no old diagnostic chain. New source verification checks all
retained corrected source blobs before copying them and confirms that only
`kernel/src/ke/npx.cpp` changes. Existing required-read and fixed-phase changes,
pinned Cromwell and retained native-v3 core remain. All old trees/builds/profiles
and sixteen paused paths are preserved. The ordinary runtime and installed
Android candidate are unchanged while this separate candidate is qualified.

The original MIT `npx-ownership.S` regression remains byte-identical. A small
UART adapter replaces its three result-report calls, activates the same public
UART used by the lifetime fixture, and preserves registers/flags. Its register
checks, persistent context, events, finite waits, parent/child ordering and
original constants are retained. Rust verifies original input identity, creates
the flat test image, and requires both child and final parent responses plus
clean native retirement. No import journal, guest-memory read, private firmware
implementation, exception/FPU observer or commercial-game instruction is used.

Tools-v37 compiled; its focused completion-policy test, warnings-denied workspace
Clippy and formatting passed. The separate SERIAL1/TRACE_IMPORTS0/SELFTEST0 open
image built with one compiler job. Exact source/patch/fixture and actual build
identities are in [cartridge-npx-owner.lock.json](../cartridge-npx-owner.lock.json).
The first adapter flattened the original `.data` into its executable page.
Both images then failed parent stage4, followed by the dependent child timeout;
the retained private reference produced the same failure. Original-owned output
showed x87 values already zero immediately after initial loads, even with an
explicit FNINIT/FLD1 control, while integer constants and XMM checks were intact.
These observations do **not** qualify a kernel restore or Fable failure. A bounded
QEMU generated-code record for original addresses0x11000..0x11080 showed code-page
stores causing translation restarts. No game/kernel instruction or live register
capture was enabled. The reference is an execution control only.

The adapter now preserves separate code and writable-data pages, as the original
source does. Rust links text at0x11000 and data at0x12000, obtains the actual linked
thunk address, emits separate executable/read-only and writable/non-executable
XBE sections, and binds both section digests. Original source, checks, ordering,
constants and existing UART adapter are unchanged. The original initial failure
is not used as evidence for another kernel/core patch.

With the corrected layout, the baseline reproduces the historical child-integrity
failure BAD00302 and dependent parent BAD00205 with clean native return. The
unchanged seven-line firmware candidate produces **both600DCAFE and4E505801**.
Its first QMP return times out and requires SIGKILL/fsync; that dirty profile is
retained. A single fresh native-return qualification, reusing the clean negative
control, then produces both success markers and normal native exit0/QMP/sync in
5,821ms. It changes no guest, firmware, core or timeout. The original red/green
regression is qualified; the exact-image retained relaunch and changed-firmware
Fable follow-through are next, with all previous failures preserved.

The legacy retained-relaunch run also completed both loads before its3s QMP
read timed out and it was forcibly retired. The qualification now uses the
existing cartridge Player/Monitor/Native owners, without changing that paused
launcher or any monitor timeout. Its first preparation attempt ended with
SIGXFSZ before guest launch: Profile::open generated an unused8GiB sparse blank
before substituting the matched seed, exceeding the finite per-file ceiling.
The developer comparison now initializes fresh storage directly from the verified
seeds under the same writer lease and durable pointer publication. Existing
storage and incomplete profiles remain authoritative and are never replaced.
The ordinary generated-storage default is unchanged. A new synthetic test checks
no generator is needed, newer current disk/EEPROM survive reopening with seeds,
and a partial seed copy stays retained/refused without modifying earlier data.

Exact-image retained relaunch now passes through the actual product session in
2,674ms: cold/quick boots and both successful load results, acknowledged native
quit, exit0, reaped and synced, seeds unchanged. This is first-stage relaunch
qualification; second-stage execution is not independently qualified. Tools-v48
build, all11 focused cartridge/storage checks, warnings-denied workspace Clippy
and formatting pass. Prior native, profile and preparation failures remain.

Ordinary runner corrections preserve failed attempts in separate evidence/profile
paths, keep QMP sockets within their actual Unix path limit, record missing native
termination evidence as failure, and reuse a verified clean negative control
without overwriting its raw receipt. The result parser rejects incomplete or
mixed pass/fail output and requires both completion markers plus clean return.
Tools-v45 focused checks, warnings-denied workspace Clippy and formatting pass.

Early attempts failed before guest execution: a long QMP socket path, and
insufficient virtual address space for retained JIT/graphics mappings. Task
admission now distinguishes8GiB virtual space from aggregate resident memory.
Original-only build/check/run phases measured below570MB; the later task applies
a stricter1GiB RSS cap plus unchanged1GiB recovery headroom. The shared6GiB
cumulative growth allowance and12GiB reserve never change. Two refused3GiB memory
admissions ran no guest and returned their leases. No shared helper/guard, phone
or sibling changed. All source, profiles, builds, evidence and paused edits remain.

[Sanitized actual observations](evidence/cartridge-npx-owner-20261027.json) retain
these limits. Only after complete original regression and exact-image retained
relaunch qualification may this separately changed firmware receive its matched
Fable observation. No ordinary runtime default or installed candidate changes
from a build or guest sentinel alone.

## Cartridge continuation: corrected logging and initial-thread lifetime

After the first XBX-CARTRIDGE-1 continuity/storage-safety slice, the single
corrected-logging Fable observation used already-qualified image
`8f1d5ac9ec1b8d97eabf13fdca53f0f995f19b56951e1ddf8dca370cf71c070e`.
It ran through the shared Rust product session policy in fresh storage, with
the retained pristine disk/EEPROM seeds, unchanged read-only external game,
64MiB, native FP, actual Intel OpenGL, attached ControllerS, offline networking
and dummy audio. The integrated core is native-v3 `b302cafa…8ead6`, retaining
the accepted base/RAMHT repairs plus the independently evidenced headless and
paused-capture corrections; their changed branches do not run during ordinary
visible cold startup. This core change is explicit, not a bit-identical rerun
of the earlier reference campaign.

The visible window remained the open-kernel splash. Ordinary logging reported
successful load, selection complete, handoff begin and title returned, then
bugcheck0x1e/access violation. The emitted location0x000f8f17 is outside the
matching public kernel map (preferred base0x80010000); it identifies no public
service or responsible thread. The observation/return interval was4.632s,
ending promptly on the established terminal output; both actual owned-window
captures were retained and the terminal frame reviewed. No normal game input
was scheduled before this early terminal result. Native quit acknowledged,
exit0/forced=false/reaped/storage synced. No healthy game checkpoint or menu,
gameplay, sound, in-game save or commercial resume milestone was earned.

An earlier admission was retired before native launch: unoptimized development
SHA-256 consumed about200 CPU seconds for2GB of the image, so the full import
could not fit its finite window. Only the verified childless owned importer was
terminated; the Rust guard reaped it and released. Its empty app directory and
receipt remain. Optimizing the standard sha2 dependency's development/test
compression code allowed the actual import in109.428s, without copying media.
This is a development-runner correction, not game progression or qualified
low-friction import latency.

Public dashboard main at xemu-dashboard8cb5e525 (blob4126b66e) remains in its
menu loop until firmware return. The retained thread fixture waits for its
children. Neither proves a worker survives the initial title thread's return.
Original `native/fixtures/startup-lifetime.S` therefore adds two finite cases.
Both use persistent image-owned context, explicit callbacks/arguments, preserved
callee-saved registers, ordinary explicitly initialized COM1 and checked handles.
The control waits for child termination before its parent completes. In the
distinguishing case the initial entry returns normally; the worker must succeed
at a finite wait on an independently opened parent-thread handle before checking
its persistent inputs and emitting completion. A pre-return flag or elapsed
delay alone is not accepted.

Both identical original images passed on the retained private execution control
and corrected open image, using the same native-v3 core,64MiB and fresh matched
seeds. Reference control/exit took13.771s/15.255s; open control/exit
22.449s/13.481s, including ordinary supervision/retirement. All four verified
ordinary guest output/completion and clean native returns. The open exit case
logged title return before the worker's after-parent-termination completion.
No call journal, exception/allocator/FPU observer, game-memory read or private
firmware implementation source was used. Private input provenance/identity
checks remain private; no new reference dashboard or commercial control ran.

Original source SHA-256
`219e084656e1ea648b89ed0c197780a1a9b69fa40b28d094768292fb60351b60`;
control ISO `b856a00a55d919f860e1d56f0a8305b75ddbe71fc482f945d1bd3cbf30957ea9`;
exit ISO `a4f1c4ae11b00cc8a639f0a7ec9c8c3287b81b5aecb43352f4af9711bbeb2ce0`.
The source uses public ABI facts from nxdk a6c31ee90540710a3df26a0712b61492f2ea03dd.
Tools-v19's seven focused storage cases, workspace Clippy with warnings denied
and formatting pass. Exact commands/source snapshots, native return receipts,
private comparison inputs, images and failures are retained in the owned
cartridge workspace.

This lifetime mechanism is not reproduced as defective; no termination repair
or additional Fable trial follows from the pass. The remaining Fable boundary
is the post-handoff access violation outside the public map, with no demonstrated
public-contract cause. Further compatibility work needs a concrete public
contract/original reproducer or Master's newly scoped evidence decision, rather
than an invented thread fix or speculative matrix. Android session integration
continues independently; synthetic passes do not substitute for playable Fable.

## Earlier basic-logging and loader-correction history

The earlier Master [6393149897](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6393149897)
accepts the completed official dashboard, private Fable reference and matched
splash-only open result. This continuation does not repeat those campaigns.
It builds the exact official source with basic logging, qualifies the retained
homebrew fixture and observes unchanged Fable on the corrected core.

## Exact build and logging qualification

`open-startup.lock.json` binds OXOS
`9d8cd58ca19f49f5ecb4b7b55bfa194f4e4a5a96`, Cromwell
`86f547387184d1001d377cce97d3756c8acf91cc`, the retained fixture and corrected
RAMHT core. Builds use IMAGE=open, SELFTEST=0, TRACE_IMPORTS=0, DEBUG=0, SERIAL=1,
upstream optimized release settings, one compiler job and SPLASH=none. Date/time
macros mean actual output identities must be retained separately. This source
build is not bit-identical to the downloaded official image.

Only kernel/tools/LICENSE/README are exported:122files,1,348,879bytes. Retained
Git objects supply all but three exact public blobs, fetched into the new
external depot. All selected source bytes match their Git identities before
build. Cromwell comes from retained pinned objects; no upstream run target,
disk download, native-core/toolchain rebuild or full xemu expansion is used.
An unused partial-history read warning was retained; no missing required source
was accepted. Later candidates reuse those same objects without another fetch.

All new outputs use one separately counted1GiB external physical-allocation
envelope above11,285,286,912allocated bytes, maximum12,359,028,736bytes. Earlier
allowances are not reset, and the shared12GiB floor and actual leases remain.
A previous occupied-lock refusal is historical; subsequent fresh admission
allowed actual work. No cross-agent scheduler, pauses or routine notices occur.

The ordinary supervisor routes existing QEMU serial output through its bounded
host stream when explicitly selected. Native and serial output are combined,
with a4MiB cap and explicit truncation/capture status; the existing1MiB active
observation limit remains. No guest-memory read, import tracer, debugger, call
journal or new exception mechanism is used. An existing terminal bugcheck or the
public kernel's already-terminal unexpected-exception message permits normal
stop/reap/sync. A screenshot/key delivery alone does not establish acceptance.

The retained homebrew relaunch image is unchanged, SHA-256
`8efe94b6f8a26f63247f44bc42da9ed437e431a5ed03180ba73044140cb88491`.
Qualification requires cold boot, successful default-image load, a guest-driven
quick reboot and successful load of its named second image. This demonstrates
first-stage guest execution/completion through ordinary logs; it does not infer
second-stage execution or inspect old title-result memory. It was qualified
on each distinct logging image before its dependent observation.

## Actual ordinary startup result

| Candidate | Exact BIOS SHA-256 | Actual stage |
| --- | --- | --- |
| Basic upstream logging | `ed905fa328b146a148b09ba67dbfb196b6f79c955dc69f1ffee5e26b93be4bfd` | Fixture2.486s; Fable load success then terminal bugcheck1e/access violation,5.766s |
| Fixed phase messages | `1b25ea00ab3ad71a3d771fd02fc0e13f3ee35e94f8ea1c8031812259d0b11639` | Fixture2.459s; Fable selection complete, entry call attempted and returned, then bugcheck1e/access violation,14.898s |
| Required-read correction, logging | `8f1d5ac9ec1b8d97eabf13fdca53f0f995f19b56951e1ddf8dca370cf71c070e` | Retained fixture qualified,2.245s |
| Required-read correction, serial off | `c29e4f5edb36807ef497e0ad8a63a984bea26daa1c075285f496815501f079b8` | 604.365s actual; splash only, no visible input response; forced retirement after QMP timeout |

The first two Fable observations ended on confirmed terminal output rather than
waiting600seconds. Native/operator exit0, forced=false, QMP stop/quit acknowledged
and storage synced. Bounded captures were complete/untruncated. Reviewed images
show the open-kernel splash, with no game/menu/control milestone. Fixed messages
are the explicitly authorized three kernel-owned selection/handoff/return
strings in public loader source, not an import observer. The return message
proves control returned from the entry call; it does not prove Fable's main
function completed or identify a faulty thread/service. Raw addresses stay local.

The corrected serial-off candidate uses SERIAL=0 in a separate fresh build and
fresh writable profile. The retained core, read-only game, pristine disk/EEPROM
seeds,64MiB, actual Intel OpenGL4.6, native FP, offline networking, ControllerS
and dummy audio are otherwise matched. All18 scheduled captures completed.
Decoded pixels in17 captures from10.695s through585.641s are identical and show
the open-kernel splash; the initial3.481s capture differs. Start, Down, Up, A and
later Start were delivered at320.058/350.059/380.154/410.125/500.114s. Reviewed
frames and pixel equality show no visible response, menu or gameplay. Ordinary
input delivery alone is not acceptance. Serial output is empty as expected;
it cannot establish that the earlier exception recurred after the correction.

Retirement was **not clean**: QMP stop/quit timed out, native SIGKILL was required,
the native process was reaped and storage was synced; supervisor/operator exit1.
The dirty profile is retained without reuse, not accepted as a coherent checkpoint.
At13:55:35UTC the native process was absent and shared/project/profile locks were
reacquirable; the phase receipt records stopped, no held lease and zero outstanding
growth commitment. No other project was asked to stop or reserve a slot.

## Independently supported required-read correction

Public-source review found that required section/header extents still discarded
successful short-transfer counts. The section loader then accepted the section;
its permissive homebrew digest policy did not supply a completion check. This is
the public contract already addressed by the accepted loader-completeness repair,
not evidence attributing the current Fable exception to short reads.

`patches/oxos-required-reads.patch` ports only that functional guard: a required
read with successful status and a byte count different from its requested extent
returns STATUS_END_OF_FILE. It covers section payloads and required header
continuations. The initial4KiB header read remains a permissive probe. Existing
I/O failures, digest policy, exported ABI and section cleanup are preserved. No
old journal/exception/allocator chain is imported.

Original Rust `fixtures/required_reads_contract.rs` exercises the exact extracted
public XeReadFile/XepLoadSection functions, with the corrected required-read
helper after the patch. A mechanical host adapter widens only pointers; byte
counts/statuses remain32bit. Synthetic I/O comes from Rust. Commit, hashing and
zero-fill dependencies are controlled stubs to isolate read completion. This
is not an i386 ABI, full memory-model, crypto or whole-loader qualification.

Before the correction, the successful-short-read case failed exactly as expected:
status0 incorrectly published a partial section; the other two cases passed.
Afterward all three pass: short reads rejected without publishing references,
full reads succeed with expected references, and an underlying I/O failure is
preserved with references unwound. The current source's actual function bodies
are compiled in both cases, not a second handwritten implementation.

The repair reuses the accepted functional behavior from
`patches/oxos-loader-complete.patch`; it omits that historical patch's counters,
raw digests, exception work and diagnostics. Fable causality remains unattributed.
The separate serial-off build follows the independent regression and retained
fixture qualification; its result must be judged from actual visible responses.

## Remaining implementation boundary

The logging baseline completes selection and returns from its entry call before
the existing terminal access-violation report. It identifies no responsible
thread/service or independently demonstrated cause of Fable's failure. The
independent required-read correction passes its focused regression but earns no
Fable progression. No corrected logging Fable run was made: the next game check
was the separately identified serial-off release above. Further speculative
call/exception/allocator/FPU work is not supported by this evidence and remains
paused under the assignment. The next scoped decision needs a public contract
or original reproducer for the post-handoff failure; phone availability is not
the missing dependency. No shipping firmware default is changed.

## Validation and preservation

The focused serial-cap/early-exit, fixture-completion and exact-budget tests,
Clippy with warnings denied and formatting passed on the completed isolated tool
snapshots. The first snapshot's test-module-placement lint failure and all source/
logs are preserved; the corrected checks have now actually completed. The actual
C++ function regression compiles with warnings denied and the original Rust tests
run serially. Supervisor SHA-256 remains
`f0e8ce3d8f4d292b9b2e5cecc96e32260f38e8ad90d64781a9da3cf101c76736`.

Final measured external project allocation is11,514,675,200bytes, an increase of
229,388,288bytes over the unchanged baseline, within the1GiB envelope. Available
space is96,134,737,920bytes, above the shared12GiB floor. Detailed commands,
identities, ordinary logs, captures and termination/storage receipts remain in
the private local `open-startup-v1` evidence and external workspace. This records
filesystem sync and process retirement, not power-loss recovery certification.

Every source export, tool/build generation, profile, media and evidence is
retained. All16 paused paths remain outside this slice. No phone operation,
other-project change, proprietary code analysis or public binary release occurs.
Neither diagnostic logging nor the synthetic repair claims gameplay, sound,
saving, cold reload or a qualified shipping firmware/profile default.

## Qualified NPX-port Fable observation — 2026-10-27

The one changed-firmware observation used exact qualified8477c047, retainedB302,
the fully verified read-only external game, matched fresh disk/EEPROM seeds,
64MiB, Intel OpenGL, ControllerS, native FP and offline operation through the
cartridge Player. The game import read and hashed media without copying it.
It completed selection/handoff and returned from the title entry before the
same terminal access-violation class outside the public kernel map. All four
captures were opened: only the open-kernel splash, no Fable menu or gameplay.
No scheduled input was delivered before the terminal result. The last terminal
capture completed by26,327ms; total observation/retirement was81,829ms, including
shutdown. QMP acknowledged, native exit0/reaped/synced; immutable inputs unchanged.
Measured aggregate resident peak545,230,848bytes, maximum single-process virtual
size5,033,115,648bytes; the stricter1GiB resident guard was not hit. The preceding
request was refused before launch for memory admission and returned cleanly.
No phone or healthy commercial checkpoint. The repair remains independently
qualified; it did not earn Fable progression or identify its cause.

The earlier original flat-layout result independently exposes native x87 state
loss on a code-page-write restart. Retained public code caches x87 results across
instructions; precise SMC restarts through CPU state without restoring those
translation temporaries. A minimal canonicalization candidate is being prepared
for the original failing image and working separate-page control, not an unchanged
commercial repeat. It was subsequently built and qualified by the actual original checks below.
No private-game code, live-register observer or prior diagnostic branch is used.

## Native x87 precise-restart correction

The original flat image's failure is a native state-boundary defect, separate
from its suitability as an isolated kernel-owner test. A code-page write can
restart the current instruction through the retained precise-SMC path. Completed
x87 instructions still held values in translation temporaries, while the restart
restored only canonical CPU state. The correction invokes existing gen_flush_fp
before each instruction boundary. It changes no guest/game instruction, kernel
ABI or machine serializer. No special case recognizes a title or fixture.

One compiler job rebuilt target/i386/tcg/translate.c and relinked with accepted
UI/HMP repairs and other retained objects. B8d9ce80 is separate from B302; the
ordinary runtime, existing profiles and installed candidate stay unchanged. The
6,034,895byte pinned compiler database uses its own bounded16MiB build reader;
the initial wrapper's64KiB session-manifest reader refused it before compilation.
The session-metadata bound remains unchanged and that preparation is preserved.

Actual product-path regression: the exact former flat image now passes both
checked child/parent results and clean native return in9,823ms. The separate-page
control also passes both checks and clean return in20,189ms. Both use repaired8477
firmware,64MiB, Intel OpenGL and fresh matched seeds; the old clean negative is
reused. Ordinary and old logging resource constructors both reject this isolated
candidate. Tools-v51 all11 focused cases, strict workspace Clippy and formatting
pass. [Build and original results](evidence/cartridge-x87-boundary-20261027.json).
This is an independent platform correction, not proof of a Fable cause. Next is
exact-core retained relaunch, then one matched changed-core Fable observation.
