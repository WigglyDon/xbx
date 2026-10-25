# Basic-logging open title startup

Master [6393149897](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6393149897)
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
| Required-read correction, serial off | `c29e4f5edb36807ef497e0ad8a63a984bea26daa1c075285f496815501f079b8` | Focused600s Fable observation in progress; result pending |

The first two Fable observations ended on confirmed terminal output rather than
waiting600seconds. Native/operator exit0, forced=false, QMP stop/quit acknowledged
and storage synced. Bounded captures were complete/untruncated. Reviewed images
show the open-kernel splash, with no game/menu/control milestone. Fixed messages
are the explicitly authorized three kernel-owned selection/handoff/return
strings in public loader source, not an import observer. The return message
proves control returned from the entry call; it does not prove Fable's main
function completed or identify a faulty thread/service. Raw addresses stay local.

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

## Validation and preservation

The focused serial-cap/early-exit, fixture-completion and exact-budget tests,
Clippy with warnings denied and formatting passed on the completed isolated tool
snapshots. The first snapshot's test-module-placement lint failure and all source/
logs are preserved; the corrected checks have now actually completed. The actual
C++ function regression compiles with warnings denied and the original Rust tests
run serially. Supervisor SHA-256 remains
`f0e8ce3d8f4d292b9b2e5cecc96e32260f38e8ad90d64781a9da3cf101c76736`.

Every source export, tool/build generation, profile, media and evidence is
retained. All16 paused paths remain outside this slice. No phone operation,
other-project change, proprietary code analysis or public binary release occurs.
Neither diagnostic logging nor the synthetic repair claims gameplay, sound,
saving, cold reload or a qualified shipping firmware/profile default.
