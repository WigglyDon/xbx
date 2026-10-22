# Bounded loaded-title VM contract

Master [6348944459](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6348944459)
authorized this independent finite test while private Fable binary analysis is
paused. The October 22, 2026 run passed. No allocator defect or repair was
demonstrated; the slice ends here. Fable remains unresolved and unqualified.

The implementation is the commit containing this document, based on
`022b0950e520c44b4290e51e576cf24e2a8ad173`. Activation found a clean tracked
worktree. Retained relaunch/null-call fixtures and results did not cover actual
loader-resolved allocate/free calls with caller-stack checks. The pinned kernel
self-test exercises internal VM behavior, not this title-import boundary; its
exhaustion title was not run.

## Source and build identity

- Original MIT mechanical guest shim: `native/fixtures/vm-imports.S`, SHA-256
  `64eed388a41af7a083c52fdd18f15e154a8cb812b8681d9fa9578b8e5e2ca041`.
  Original Rust `xtask` and host runner own build, bounds, observation and acceptance.
- Public OXOS `c0c887a1263ebf89ba765f34a88ca8f132901e25`; interfaces
  `kernel/include/xk/mm.h` and `kernel/exports.tsv`: ordinal 184,
  `NtAllocateVirtualMemory` stdcall20; ordinal 199, `NtFreeVirtualMemory`
  stdcall12. Existing pinned `tools/mkxbe.py` and `tests/mkxiso.py` wrap the title.
  Their GPL/per-file terms remain applicable; no implementation was copied.
- Existing v4 OXOS trace patch SHA-256
  `7eb48fb10ca5784a887d93db23f02c07ec83dc7b105930e1a71204724e0a0dc5`;
  Cromwell `86f547387184d1001d377cce97d3756c8acf91cc`.
  Existing firmware SHA-256
  `0cdedf6d81dabdbeafc1cff78039cf78f19cb7923078f201724e346ca94ee8e8`;
  open image, SELFTEST=0, SERIAL=1, TRACE_IMPORTS=0, XBX launch trace enabled,
  64 MiB. No firmware rebuild, proprietary boot image or owner EEPROM input.
- Existing xemu Android fork `4967efd3ee594cc236869a45ee4cb396e25af1ac`, desktop
  build, ABI 3; working patch SHA-256
  `26027ec1436fb6c5f7a51200bf7fe96ceb90ba5695d568a72db29ef446ff461f`;
  executable SHA-256
  `51178df74f0b26316c799ff8e8c1a65b1b841b97858cd07dd31903d52ebdcd0d`.
  Retained QEMU/xemu GPL and per-file terms apply; see `THIRD_PARTY.md`.
- Read-only NDK 30.0.16248370 clang/lld: i386-pc-win32, pentium3, no default
  libraries, fixed base 0x10000, native entry, reproducible PE link.
- Built XBE SHA-256
  `f4f24fd0df1579d2535a38ee84b9268934ca9f43a2ed0ae5738fc25fbd627541`;
  synthetic ISO SHA-256
  `982d2727f58c4ce185689089323e152379855841091685c0bdd6a0b9f52efadf`.
  XBE size 0x1460, entry 0x11000, two-import thunk table 0x13000,
  results 0x1300c, title ID 0x58425802.
- Observing host executable SHA-256
  `4acedebf3dd7eb3dcf600b6c9de2d9de2d6b9f9a1bbbb7c1522b5ad5860f3a9a`.

## Reproduction and isolation

The existing host preflight passed: 47,516 MiB free and 1,485 MiB usable Btrfs
metadata initially; build preflight reported 46,990/1,470 MiB. Emergency reserve
was unused. Approximately 5,306 MiB RAM was available; other host jobs were
left alone. One compiler job was used. Existing source/dependencies/tools suffice.

From this checkout, with retained inputs (substitute the read-only NDK path):

```sh
cargo xtask capacity 1
XBX_NDK=/path/to/ndk/30.0.16248370 cargo xtask vm-import-build
G_DEBUG=fatal-criticals cargo run -p xbx-host --offline -j1 -- boot-vm-imports \
  .local/build-desktop/qemu-system-i386 \
  /home/don/xbx/.local/vm-import-contract-v1 \
  .local/build-oxos-launch-7eb48fb10ca5/bios.bin \
  .local/profile-build-0cdedf6d81dabdbeafc1cff78039cf78f19cb7923078f201724e346ca94ee8e8-fatx-v2/assets/hdd.qcow2 \
  .local/build-vm-import-fixture-v1/vm-imports.iso \
  .local/build-vm-import-fixture-v1/title.map
```

The builder refuses to overwrite `.local/build-vm-import-fixture-v1`. The runner
requires a nonexistent scratch directory; use another short name for a future
explicit rerun, preserving the recorded directory. A fresh checkout can use the
commands as written after preparing the documented pinned inputs. No additional
native campaign was run after this pass.

The seed disk SHA-256 was
`0f123236d5916e5c455345c4b91cb9876ae821766475e69ede205c8fd9ae2e71`.
The runner copies it into the new isolated scratch and generates a synthetic
EEPROM there. It never opens an owner game/profile. SDL offscreen/dummy audio,
software Mesa, TCG and 64 MiB were used. Existing writer ownership remains held
and inherited through actual child reaping, followed by storage sync. A 60-second
guest deadline and bounded native/serial logs apply. Host QMP reads results;
it does not write guest completion or fabricate operation results.

## Actual guest and termination results

The ordinary loader resolved the two import slots to 0x8002e100 and 0x8002e690,
matching the retained firmware map. Calls used those resolved slots.

| Guest operation | Actual result |
| --- | --- |
| Reserve 64 KiB, base NULL, MEM_RESERVE, PAGE_READWRITE | status 0; base 0x00020000, size 0x10000 |
| Separately commit two interior pages, MEM_COMMIT | status 0; base 0x00021000, size 0x2000 |
| Initial contents | all 2,048 dwords zero |
| Bounded write/readback | all 2,048 dwords matched `0x58425800 XOR index` |
| Release original reservation, input size zero, MEM_RELEASE | status 0; returned base 0x00020000 and released size 0x10000 |
| ESP before/after each of the three calls | 0xd001cd6c / 0xd001cd6c for every pair; mask 7 |
| EBX / ESI / EDI / EBP after each call | 0x11223344 / 0x55667788 / 0x99aabbcc / 0x13579bdf; mask 7 |
| Guest completion | magic 0x4d564258, final stage 6, failure 0 |
| Existing v4 serial allocation records | one reserve and one commit record matching title return PCs 0x11052 / 0x11144, flags/ranges/status; loader allocation is separate |
| Unexpected exception records | 0 |
| Native termination | acknowledged QMP stop/quit; exit 0; forced=false; storage synced |

The guest publishes completion only after release/range/ABI checks, then idles.
Its 36-word result contains statuses, ranges, measured ESP pairs and preserved
registers, actual loop counts and resolved IAT values. The Rust runner checks
all these independently and correlates the two allocation records. A timeout,
explicit guest failure, wrong data/stack, exception or abnormal native shutdown
returns failure and preserves scratch. Release success is the actual guest call
and returned range, not an independent post-release inaccessible-page probe.

Retained local evidence under `.local/vm-import-contract-v1`:

- `vm-import-results.txt` SHA-256
  `4e46a7479d69690bdee8eb4b89e13d72fc33d6ebc95f16c33d186da91b39af04`.
- `vm-import-acceptance.txt` SHA-256
  `b68b4044ac160c2c6f6fc45592108c10a351eaf2408f0df5c5edbbd5da01bc61`:
  guest_contract=true, allocation_calls_correlated=true, unexpected_exceptions=0,
  accepted=true (normal termination recorded separately).
- `termination.txt` SHA-256
  `35907392d8fc6000474598148a5edd23e7b56e87b2f145467f7b9afbb41e9e8d`.

`cargo test --workspace --offline -j1` passed 18 tests; three explicitly gated
native/storage tests stayed ignored. `cargo clippy --workspace --all-targets
--offline -j1 -- -D warnings` passed. Build/source checks and those tests are
distinct from the actual guest result above.

This establishes only this synthetic imported VM/ABI path on this desktop build.
It does not diagnose Fable, test Android, qualify performance, rendering, audio,
ordinary game saves, snapshots, shortcuts or owner acceptance. No phone was used
or APK/firmware rebuilt. No game bytes were read. Existing builds, profiles,
media and private evidence were preserved; the independent scratch is retained.
