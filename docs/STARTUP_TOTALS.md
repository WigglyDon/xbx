# Typed startup totals before the observed event

The owner-directed continuation after d9c8492 closes one specific diagnostic gap:
the recent1024-call window does not rule out an earlier allocation/API failure.
This adds fixed whole-startup totals to the existing observer; no game code,
new exception interception, buffer/path capture or pointer-following is involved.
Private game-code/disassembly work remains paused.

`kernel-totals.lock.json` binds OXOSd87/Cromwell86f, the complete
`patches/oxos-startup-totals.patch`
464f82127919cef2e75455d97401823523b388f55afa4aef096ac4797c5b0c7b and all accepted
base patches. Retained kernel modifications keep their terms; Rust/MIT owns
new build/schema/capture/acceptance policy. Observer version2 additionally
classifies the public NTSTATUS returns of ordinals24/37/99/246. Version1 captures
retain their original classification and remain readable.

`XkStartupTotals` is fixed5888bytes: eight header words and366 four-word rows.
Header: magic58425431/version1/size, matching event order/call cutoff, explicit
overflow/pre-arm/out-of-range failure flags, entry total and observed-return
total. Each ordinal row contains entries, observed returns, typed signal count
and first negative NTSTATUS. A pointer-return row's signal is its null count,
with no pointer value retained; void/unclassified rows have no result. Negative
statuses and null results do not themselves establish an invalid contract.

The totals freeze with the first event. Late observed returns before capture
contribute even when their individual ring slots were overwritten; post-capture
bookkeeping continues without changing totals. Omitted frequent APIs remain
omitted; a return remains distinct from asynchronous work completion. Full static
observer accounting is65176bytes, including512 possible import stubs, under the
existing65536-byte bound. Rust rejects bad schema/size/event/flags, contradictory
counts or result kinds and disagreement with the recent window. Reads occur
only from the bound fixed kernel storage after acknowledged stop.

Original `native/fixtures/startup-totals.S` prefixes the retained observer fixture
with a one-GiB allocation request (must return null on this64MiB machine) and an
invalid-handle wait (must return c0000008 without waiting). Both check stdcall
stack balance. The existing fixture subsequently overwrites their ring slots,
completes a late return, performs its controlled handled read fault and checks
all original lifetime/continuation/final-completion conditions. No owner input
or runtime title-derived fixture is used.

Actual `.local/startup-totals-fixture-v1` passed: allocation row3/3/null1,
wait row3/3/negative1(c0000008), late wait2/2, create3/2 and termination2/0 at
cutoff1124. The early controls are absent from the recent ring. Actual guest
completion, correct handled dispatch, normal native exit0/forced=false,
acknowledged stop/quit and storage sync all verified independently. The original
fixture's final HLT loop is stopped at30seconds.

Built/used identities:

- Firmware7f3d1e72c185605834bab030d0f15bddef1e59a7e8a7c22bc6c0e477206db0ef.
- Mapf6aec23c59ab9514107cf2c3494c6052d1e9a55e93635302d3eb7cdfc23e1ed0.
- Build receipt9790eb7f4d64395476c1714ddfc43ef19296df18e20a41cff8f321771eb445f7.
- Supervisorb3a08c6aabf7a009bb953fa392ea9d04db324b1be0fc62041c5c49a9e9b81115.
- FixtureXBEf9e0031f79ec3fcb714294027fd89d7615aec6d222b92ddd6e05d9618994161d;
  ISOa03dd759f572e6647bfa2374ed3d1ab8d7bb83ae69d2c517d55d23e7fd6093df.
- Same retained desktop core51178df/source4967efd/ABI3,64MiB.

One job, open image/SELFTEST0/SERIAL1/TRACE_IMPORTS1/launch trace1; no Android
selector changes. Capacity31233MiB available/1321MiB usable metadata, reserve
unused. Separate source `oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2-startup-totals-v1`
and full pin/mode/patch output identity. Previous source/build/profile/Android
artifacts remain preserved.31 serialized workspace tests pass/3 gated ignored;
Clippy-Dwarnings, fmt and diff pass.

```sh
XBX_NDK=/path/to/ndk/30.0.16248370 cargo xtask kernel-build totals
XBX_NDK=/path/to/ndk/30.0.16248370 cargo xtask startup-totals-fixture-build
cargo build --offline -p xbx-host
G_DEBUG=fatal-criticals target/debug/xbx-host run-title-journal "$CORE" "$NEW_ROOT" "$FLASH" "$HDD" "$EEPROM" .local/build-startup-totals-fixture-v1/startup-totals.iso 30 "$MAP"
target/debug/xbx-host check-startup-totals-fixture "$NEW_ROOT"
```

Actual unchanged Fable `.local/fable-startup-totals-v1` used fresh independent
storage, retained read-only partition view, ControllerS, X11/software Mesa/dummy
audio. Cutoff2151, observed returns2148; no null returns among the covered
allocation APIs throughout startup. Eleven EEPROM queries and five object-handle
references returned successful statuses. One virtual-address conflict is followed
in the existing bounded VM records by a successful alternate reservation; queryVM
has one invalid-parameter status, as previously observed. File opens report
missing names/paths and flushes access denied; no cause is established by these
statuses. Fourteen NtReadFile calls returned success; this does not verify their
output bytes or asynchronous completion. No scatter-read call was observed.

The game still reaches the same fault boundary, no menu/gameplay. Open-kernel
splash reviewed in the exact owned window. Terminal result at92.009seconds;
normal native exit0/forced=false, valid captures, acknowledged stop/quit and
storage sync. All67 pre-bound retained identities match afterward. No phone/APK,
game transfer, profile reopen, snapshot rollback or PSX change. Gameplay/audio,
in-game saves, full resume, shortcuts and mobile performance remain unqualified.

Continuing: four observed public NTSTATUS APIs remain unclassified in version2
(IoCreateDevice65, IoCreateSymbolicLink67, XeLoadSection327, XeUnloadSection328).
Their public declarations, rather than untyped EAX values, determine the next
small classification extension. An independently noticed possible pool tail
publication/preemption race remains a source finding pending focused proof;
it is not attributed to Fable or patched speculatively here.
