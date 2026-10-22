# Desktop API boundary and corrected failure evidence

Don authorized continued public-platform diagnosis toward the finished product
in the existing worker after source `576b2fa19b40754d8b27dae69cd33230a28cd178`.
This work does not establish a playable product. Private title-code analysis
remains paused; no game instructions, executable fragments or withheld responses
were inspected or recovered. No phone or APK operation occurred.

## Correction to the preceding receipts

Exact-field rechecking of the preserved desktop A1/A2/B1 and loader-candidate
records establishes a **nonzero saved PC**, a **nonzero null-adjacent fault
address**, CPU vector14/c0000005, then bugcheck0x1e. The prior public handbacks
incorrectly called the desktop PC zero. Earlier phone zero-PC/heap-null evidence
is separate. The desktop records include seven successful VM returns and one
conflicting-address return in the eight-record budget, including successful title
allocations after the loader reservation. A failed reservation followed by a
successful retry is not by itself a defect.

`runtime_summary.rs` parses exact whitespace-delimited names, rejects duplicated
or malformed PC fields, and distinguishes missing, zero and nonzero values.
An auxiliary `stack_pc=00000000` cannot classify the saved PC. This correction
uses existing records; it adds no exception instrumentation. Raw evidence was
preserved, not rewritten.

```sh
cargo run -p xbx-host --offline -j1 -- summarize-runtime /retained/kernel.log
```

## Bounded public-kernel API journal

The donor's existing import-call trampoline is retained with its actual GPL/
per-file lineage. The separate diagnostic reduces its ring to **1,024 entries,
16KiB**, each containing only sequence, ordinal, return-observed flag and EAX.
It records no caller address, thread address, arguments, strings, paths, buffer
contents or game instructions. Return EAX can be a pointer or an undefined void
return: raw numeric journal files stay private. Only correctly typed API results
are suitable for a public summary. The existing exclusions of frequent APIs are
retained; this is a bounded recent-call window, not a complete execution history.
At return, sequence matching prevents a late blocked call from assigning its
result to a ring slot reused by a newer call.

Rust verifies the full firmware/source/patch/mode/map receipt before native entry.
After acknowledged QMP stop, it reads only the bound kernel's journal globals and
fixed-size ring, refusing low/title pointers, misalignment and range overflow.
It never reads title memory in an ordinary game run. Quit/reap/storage sync and
profile ownership remain in the existing cleanup path. Failed capture retains
evidence and fails the run rather than fabricating a clean journal.

The first diagnostic exposed a source-supported tracer limitation: OXOS's
`IopCompleteRequest` compares its callback against `NtUserIoApcDispatcher` to
select user versus kernel APC delivery. Wrapping that exported callback changes
its identity. Version2 therefore leaves ordinal232 unwrapped. No exception or
I/O completion implementation was changed. This is a diagnostic correction,
not an attributed Fable compatibility repair. The prior v1 patch, lock, source,
firmware, host and observations remain retained and separately identified.

## Source/build identities

Both generations use OXOS `d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2`, Cromwell
`86f547387184d1001d377cce97d3756c8acf91cc`, unchanged v4 base patch
`7eb48fb10ca5784a887d93db23f02c07ec83dc7b105930e1a71204724e0a0dc5`, and accepted
complete-read base patch
`420490cdde28fb40a882c179aa735ae6bd6c20aa4a056bed5131dcabfb4d5fcb`.

| Artifact | SHA-256 |
| --- | --- |
| v1 composed patch | 41952329c8b2b243786e13af231ada35afd56b07eccac9350fda063b7b719d40 |
| v1 firmware | 760b40f1d0af3982b26a3e1683ea816f56626af5dcc62223af78be998abf2065 |
| v1 map | 30ce35764cdc218e3e3bd6c400a01735bdbf485b96f5407343bb02c4babd73be |
| v1 receipt | 46e9199775324699260201732d496eb9cca80f2265c2429e4ce5df8b0659e0f5 |
| v2 composed patch | 05392437f9dbf14ee01b08e6ed31ffd97e894e55c723df3f6001ee07c2901da5 |
| v2 firmware | ced722fe94acd08a81a15c1e45b44b0f9b948033c8e045f1746651090ea0ad53 |
| v2 map | 8faaeef9f5f721f586961af65dcce53b58703927e7a2aacbe50921f7654bff15 |
| v2 receipt | 58744c6dd29077b20cafcecb3f03aa9a70fd808509f378b593f28696ad38cfb8 |
| executed v2 Rust host | 8aae4d40a1f55ff77f1317f99cdd73b2b5383af20342649b62c6343c3a712028 |
| original v2 fixture assembly | e6104a095896d6f47cdb438142533fec56c28683b60681ee55790e2f6414d8f3 |
| v2 fixture XBE | 1ffc401205ffe7960c6d121738127fb5da7807dff37773c08b3f349d3ea45390 |
| v2 fixture ISO | 75118d08d7a544edc2dff37137403ab65be0494769aec46f75ab9165f93b9d16 |

Flags: IMAGE=open, SELFTEST0, SERIAL1, TRACE_IMPORTS1, launch trace1, no splash or
proprietary inputs, **64MiB**. Existing read-only NDK30.0.16248370, one compiler
job. Outputs are exclusive full-pin/mode/patch directories under `.local`.
The v2 source path is `.local/src/oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2-api-journal-v2`.
Apply `patches/oxos-api-journal.patch` once to a clean pinned worktree, using the
existing pinned Cromwell dependency. No Android selector changes.

```sh
cargo xtask capacity 1
XBX_NDK=/path/to/ndk/30.0.16248370 cargo xtask kernel-build journal
XBX_NDK=/path/to/ndk/30.0.16248370 cargo xtask journal-fixture-build
cargo run -p xbx-host --offline -j1 -- run-title-journal \
  CORE /new/short/scratch FLASH SEED_HDD SYNTHETIC_EEPROM \
  .local/build-api-journal-fixture-v2/journal.iso 20 KERNEL_MAP
cargo run -p xbx-host --offline -j1 -- check-journal-fixture \
  /new/short/scratch/api-journal.txt
```

## Actual original-guest regression

Original MIT assembly uses three loader-resolved calls: bounded fill64bytes,
compare64bytes and fastcall byte swap. It checks two adjacent canaries, ESP and
EBX/ESI/EDI/EBP before publishing the final sentinel through the third return.
The v2 fixture additionally compares ordinal232's resolved pointer with the
kernel's own PE export entry. This reads only this public kernel and synthetic
fixture, not a game. Existing pinned mkxbe/mkxiso wrap the fixture; Rust owns
build/runner/acceptance policy. No SDK or new application framework.

- Original v1 fixture/v1 candidate: all three expected returns, success sentinel.
- v2 identity fixture/v1 candidate: guest detected the wrapped pointer and
  published the failure sentinel through one call, rather than reaching success.
- Same v2 fixture/v2 candidate: preserved callback identity and all three checked
  calls; compare returned64, byte-swap returned04030201, no failure call.
- All three separate runs: acknowledged QMP stop/quit, native exit0, forced=false,
  lease through actual exit/reaping, then storage sync. Synthetic data only.

## Desktop launch setup and ordinary game observations

Source review found the ordinary desktop runner omitted port1 binding. The
retained frontend creates a keyboard source but only binds it on an explicit
matching configuration entry. Rust now selects one Controller S on port1, with
automatic binding of other host controllers disabled. QMP `info usb` separately
confirms the actual Microsoft Xbox Controller S attachment. Keyboard control is
an observation route, not Android touch/input acceptance. An explicit diagnostic
`XBX_DESKTOP_CONTROLLER=none` retains the prior disconnected configuration.
No global keyboard settings or unrelated windows are changed.

All game runs use the unchanged retained read-only Fable image, validated raw
partition view, original core4967efd3/ABI3 with executable SHA-256
`51178df74f0b26316c799ff8e8c1a65b1b841b97858cd07dd31903d52ebdcd0d`,64MiB,
X11/software Mesa/dummy audio, independent scratch disks and synthetic EEPROMs.

| Run | Actual outcome |
| --- | --- |
| v1 journal, preceding disconnected setup | Terminal startup failure at102.574s; nonzero PC/null-adjacent read; open-kernel splash captured/opened; no menu. Recent kernel calls include successful I/O/device/thread setup. |
| v1 journal, verified Controller S | Same classified terminal failure at92.743s. Attachment is established; input/gameplay is not. |
| v2 callback-preserving journal, verified Controller S | Same classified terminal failure at91.829s. Seven successful VM records and one conflict/retry record within budget. No menu/gameplay result. |
| Retained loader firmware without API wrappers, verified Controller S | Same classified terminal failure at84.903s; normal acknowledged exit0, forced=false, storage sync. No gameplay acceptance. |

Every listed run acknowledged stop/quit, exited0 without force, and synced storage.
The journal shows successful public API calls, not the correctness of their entire
contract or all output data. Recent missing-file/path and flush-access-denied
statuses do not independently identify a platform defect. API order alone does
not identify the faulting thread or causality. Wrapping affects layout, memory
use and scheduling; v1 additionally had the documented callback identity issue.
No raw journal, addresses, strings, private game content or logs are published.
The separate non-wrapper firmware/control observation is recorded with its own
receipt; it must not be inferred from diagnostic results.

The public thread/TLS interfaces were also compared with pinned nxdk
`a6c31ee90540710a3df26a0712b61492f2ea03dd` and Cxbx
`f1e54be515bb7989bf5109ebea2494b6b13eb1bd`. No code was copied. The nxdk startup
owns template copying/self-reference and requests rounded TLS storage; this
supports retaining the kernel boundary rather than adding speculative title TLS
initialization. No original VM/relaunch campaign was restarted.

## Checks and preservation

Host preflight reported37,314MiB available and1,398MiB usable Btrfs metadata;
emergency reserve unused, approximately4,945MiB available RAM. Each build uses
the existing xbx writer lease and fresh accounting. Other jobs were untouched.
The41 retained firmware/maps/receipts/core/fixture/Android-selector/template
identities matched before/after. Full retained game hash matched after the runs;
all old profiles, synthetic test generations and private evidence remain intact.
No phone was acquired, so the previous verified device return remains settled.
No PSX or other-project change, public binary release, save/snapshot/shortcut/
performance or owner-acceptance claim. Scratch game writes are not proof of an
ordinary in-game save; no snapshot was restored.

Workspace tests:23 passed,3 gated tests ignored; Clippy with warnings denied,
format and diff checks pass. These are separate from original guest and native
results. Evidence under `.local/evidence/api-journal-v1` includes actual build,
fixture, ordinary-run, exact-field correction and preservation receipts. The
current supported boundary is execution after intact loading, successful title
allocations and observed I/O/device/thread setup. No causal Fable fix is claimed.

## Source-directed floating-point control

The retained fork diverges from pinned upstream xemu03b988c9d76553b33ed712ba00822ffdf49f0e7d.
The public comparison reports287 fork-side commits and240 upstream-side commits;
its300-file response is truncated, so it is not a complete difference inventory.
Local pinned source confirms fork-specific floating-point JIT and deferred x87
FIP/FDP updates default enabled on desktop. ARM-only fastmem is not active on
this x86_64 host; superblock formation is explicitly disabled at this pin.

The bounded existing-core control selected `perf.fp_jit=false` and process-local
`XEMU_TCG_DEFER_FIP=0` through original Rust diagnostic policy:

```sh
XBX_DESKTOP_FP=software XBX_DESKTOP_VIDEO_DRIVER=x11 G_DEBUG=fatal-criticals \
  cargo run -p xbx-host --offline -j1 -- run-title \
  CORE /new/short/scratch LOADER_FLASH SEED_HDD SYNTHETIC_EEPROM FABLE_ISO 300
```

Executed host SHA-256:
`c5a25650730b7f45bd5be5eaa905286c4821070d4747d07c640d554f0917fa17`.
Same retained non-wrapper loader firmware, actual Controller S, core and64MiB.
Result: same nonzero-PC/null-adjacent c0000005 and bugcheck0x1e at75.483s;
normal acknowledged native exit0/forced=false/storage sync. Its own-window
capture was opened during execution and showed the open-kernel splash.
No menu/gameplay acceptance. The two toggles are one conservative control;
this does not attribute failure to either feature or reproduce all upstream
CPU semantics. No new native core build was performed for this control.
