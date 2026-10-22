# Required image reads and pre-import integrity

Master [6352926390](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6352926390)
assigned this finite correction; the existing worker acknowledged at
[6352960673](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6352960673)
on clean source `85340ca02871a4078b775925e76f0e0e8f9fee0f`. The implementation
is the commit containing this receipt. The original fixtures prove a real loader
defect and its correction. **Fable still fails before a menu/gameplay**, despite
complete, digest-matched preloads. One authorized uninstrumented control also
fails. No completed VM/relaunch campaign, phone trial or private game-code
analysis was repeated.

## Public correction and exact candidate

At OXOS `d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2`, successful XDVDFS reads
may correctly report fewer bytes at EOF. Required section/header-remainder reads
discarded that count. The loader could mark a partially copied section loaded.
Some failure paths also fetched image-header fields before backing was assured.

The scoped retained C++ correction:

- Requires exact completion for each required raw payload and header remainder;
  successful short reads become `STATUS_END_OF_FILE` (`c0000011`) before entry.
- Preserves the initial probe's valid short-read behavior once enough header
  bytes exist. Required header fields are fetched only after availability/success
  checks. Certificate/section-table bounds use checked ordering/subtraction.
- On section failure, unwinds shared-page references through the existing unload
  path; failed initial image loads release the reservation. Exported interfaces
  and permissive digest/signature policy remain unchanged.
- Rechecks all preloaded raw section digests after all preloads complete and
  before import-thunk resolution mutates raw contents. The existing algorithm
  hashes little-endian raw length followed by payload. Expected all-zero/all-FF
  digests are reported as unusable, separately from match/mismatch.

At most eight image attempts emit two short count/status lines each:
`XBX-IMAGE` reports required/completed header/raw bytes, extents, short reads and
I/O failures; `XBX-INTEGRITY` reports preloads/loaded, audit readiness and digest
categories. Computation is before imports; final emission also reports the load
status. No payload, address, instruction trace or game string is added to these
records. A digest mismatch still does not reject the image. The summary does
not certify header/BSS/TLS/import semantics, later writes or independent image
authenticity. On-demand section reads also require completion; their failure
cleanup is source-reviewed, not a separately expanded guest campaign.

`kernel-loader.lock.json` binds the full d87-based composition:

- Existing v4 patch SHA-256 remains
  `7eb48fb10ca5784a887d93db23f02c07ec83dc7b105930e1a71204724e0a0dc5`.
- Full `patches/oxos-loader-complete.patch` SHA-256:
  `420490cdde28fb40a882c179aa735ae6bd6c20aa4a056bed5131dcabfb4d5fcb`.
  Beyond v4, only `kernel/src/xe/loader.cpp` differs. The other five traced files
  are byte-identical. The full patch applies to clean d87, not an already-patched
  v4 tree. The old patch, lock, worktrees and artifacts are preserved.
- Cromwell remains `86f547387184d1001d377cce97d3756c8acf91cc`.
- The new source is isolated at
  `.local/src/oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2-loader-complete-v1`.
  Rust verifies exact HEAD, complete working diff, untracked-source absence,
  Cromwell and both patch identities. No SDK/dependency campaign occurred.

Build preflight passed: initially 38,407 MiB free /1,414 MiB usable Btrfs metadata;
fixture preflight 37,774 /1,400 MiB. Emergency reserve unused; about 5,604 MiB RAM
available. Other host jobs were untouched. Existing read-only NDK30.0.16248370
LLVM, Make/Python tools and one compiler job were used. Two retained APC-cast
warnings remained visible; no build error. IMAGE=open, SELFTEST=0, SERIAL=1,
TRACE_IMPORTS=0, XK_XBX_LAUNCH_TRACE=1,64 MiB, no splash/comparison BIOS/MCPX input.

The exclusive output basename under `.local` is:

```text
build-oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2-86f547387184d1001d377cce97d3756c8acf91cc-loader-420490cdde28fb40a882c179aa735ae6bd6c20aa4a056bed5131dcabfb4d5fcb
```

| Actual artifact | SHA-256 |
| --- | --- |
| `bios.bin` | `ea5aad470a8a47748ed1ad22cbf9d43cebb335a0d33801ccec6230c746a43fef` |
| `xboxkrnl.map` | `29ecb87395fa54b44feab02f82ae15aabe9604c0d04756a93bb7732ed3bfaae9` |
| `build.identity` | `0ca7201b44fe67af5ac294e321f143d216a992a7816f55908f6d8cc6e6f9d20f` |
| Executed Rust host | `502a239cade48a7e5f9c938ee8c7c7cb03a3c518ab26f2c2384c94b89500f079` |

The `loader` build mode retains full source/Cromwell/mode/composed-patch output
identity, exclusive creation and input/image receipts. It writes no Android
selector and cannot be mislabeled as accepted baseline/v4 assets. Kernel changes
retain GPL/per-file terms; original fixture/build/runner policy is MIT. No binary
release is made. See `THIRD_PARTY.md`.

## Original finite fixture and actual guest results

`native/fixtures/loader-complete.S` SHA-256 is
`ba8a86b5924fef31f53dd49d0a52d81a96f300eaa314b4c91d620ab3bf75e18c`.
Rust builds an original fixed-layout three-section XBE from this mechanical ELF
shim and source-authored patterns. It uses public format/interface facts and
existing host SHA-1, not a copied loader/SDK. Retained pinned `mkxiso.py` wraps
only the authored title. Directory lengths are actual XBE lengths, so sector
padding cannot hide EOF.

The valid image is 3,664 bytes, headers1,088 bytes: the initial 4 KiB probe must
legitimately succeed short. Text/initialized data and initialized/tail data share
boundary pages. The title checks 512 initialized A dwords,504 zero A dwords,
64 initialized B dwords,448 zero B dwords and both shared boundary references
(two each), then publishes actual results and idles. Raw payload total is2,566
bytes. Four original variants share this layout/build:

- Valid XBE SHA-256 `4c56299e8afc0e9ca4af587cf8775eb54a124b2546b5a589b4bef8f612fa5ecf`;
  ISO `8948d277465e41d82926a3467d3f1fd311090123866d058ae6f3276e226e0390`.
- Truncated XBE is3,600 bytes: final preloaded raw section lacks64 bytes, while
  headers retain the complete declared extents. XBE SHA-256
  `332bf812fa45c2f3feba73ce94e9c0dae80d429e2b3a33ae6f49ee6f747bdcbf`;
  ISO `16417cfe855b49461c84e80f70d11c392961f1db4588c5b7cc60682bd91f2997`.
- Required-header failure:5,120 actual bytes,6,144 declared header bytes. The
  initial4,096 bytes arrive, then the required2,048-byte remainder completes only
  1,024. This focused case rejects before section loading. ISO SHA-256
  `d966e9224af6cbf9e3a9e7099a33c013c9e790ba19110fa1f1673f34f73b47c3`.
- Digest-policy variant retains valid payload but one altered expected digest
  and one absent expected digest. ISO SHA-256
  `7b2a266b777bea6ed9dfbb03552d7d627182dfcd76543a8fa4890348a55eca46`.

| Guest/native regression | Actual result |
| --- | --- |
| Old retained d87/v4 + truncated title | Load was not rejected; guest entered and reported failure3, A512/zero504/B48. This reproduces the defect. |
| Candidate + valid title | Guest entered/completed: failure0, counts512/504/64/448, shared references passed. Headers1,088/1,088, raw2,566/2,566, extents4/4; loaded3, ready1, match3/mismatch0/unusable0. |
| Candidate + truncated title | Explicit c0000011 before entry; header complete, raw2,502/2,566, extents3/4, short1; loaded2/3, ready0. No exception/bugcheck. |
| Candidate + short required header | Explicit c0000011 before entry; headers5,120/6,144, extents0/1, short1; raw0, loaded0, ready0. No exception/bugcheck. |
| Candidate + permissive-digest title | Same successful guest data/zero/shared-page result; complete reads; match1/mismatch1/unusable1, no digest rejection. |

All five checks passed their distinct Rust acceptance conditions. All acknowledged
QMP stop/quit, actual native exit0, forced=false; inherited writer lease remained
through actual reaping, followed by storage sync. Tests used isolated synthetic
scratch only. A loaded marker alone is not any of these complete guest checks.

## Reproducible commands

Prepare the new clean pinned worktree, link its retained verified Cromwell at
`third_party/cromwell`, then apply the full composed patch once. Do not replace
the old source/output or repeat these commands over existing attempts.

```sh
cargo xtask capacity 2
git -C .local/src/oxos worktree add --detach \
  "$PWD/.local/src/oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2-loader-complete-v1" \
  d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2
# Link the retained pinned Cromwell checkout at this worktree's third_party/cromwell.
git -C .local/src/oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2-loader-complete-v1 \
  apply "$PWD/patches/oxos-loader-complete.patch"
XBX_NDK=/path/to/read-only/ndk/30.0.16248370 cargo xtask kernel-build loader
XBX_NDK=/path/to/read-only/ndk/30.0.16248370 cargo xtask loader-fixture-build
cargo build -p xbx-host --offline -j1
G_DEBUG=fatal-criticals target/debug/xbx-host boot-loader-fixture \
  ENGINE NEW_SCRATCH FLASH EMPTY_SEED_DISK FIXTURE_ISO TITLE_MAP EXPECTATION
```

Actual scratch/ISO/expectation rows: `loader-old-truncated-v1`/truncated/
`truncated-old` with retained d87 firmwaref5cac7ba; `loader-new-valid-v1`/valid/
`valid`; `loader-new-raw-v1`/truncated/`reject-raw`; `loader-new-header-v1`/
header-short/`reject-header`; `loader-new-permissive-v1`/permissive/`permissive`.
The latter four use the new candidate. ISO/map paths are under
`.local/build-loader-fixture-v1`; the common seed is the retained empty FATX
asset whose SHA-256 is
`0f123236d5916e5c455345c4b91cb9876ae821766475e69ede205c8fd9ae2e71`.
The existing Rust probe retains its60-second bound, capped logs and native cleanup.

## One unchanged Fable launch and one authorized control

The actual retained media remains mode0400, unchanged and structurally identified
as original Fable4d53000d/version3, partition405,798,912. No game instruction,
payload dump, extraction, conversion or new media transfer was used. The existing
ordinary Rust runner/core/validated read-only FD/raw-partition view are unchanged.
Use the candidate output above for FLASH, and the same inputs as B1:

```sh
XBX_DESKTOP_VIDEO_DRIVER=x11 G_DEBUG=fatal-criticals \
  target/debug/xbx-host run-title \
  .local/build-desktop/qemu-system-i386 /new/private/scratch \
  FLASH EMPTY_SEED_DISK .local/quick-reboot-d87cfc33-run1/eeprom.bin \
  "$AUTHORIZED_MEDIA" 300
```

Native core is unchanged fork4967efd3ee594cc236869a45ee4cb396e25af1ac /ABI3,
SHA-256 `51178df74f0b26316c799ff8e8c1a65b1b841b97858cd07dd31903d52ebdcd0d`.
64 MiB, TCG, X11/OpenGL/software Mesa/dummy audio and identical synthetic
disk/EEPROM templates match B1; each run copies independent writable storage.

Candidate `.local/fable-loader-v1`:

```text
status=00000000 hdr_req=8760 hdr_done=8760 raw_req=8328772 raw_done=8328772
ext_req=16 ext_done=16 short=0 io_fail=0
preloads=15 loaded=15 ready=1 match=15 mismatch=0 unusable=0
```

These are actual kernel-owned results before import mutation. Final load status
was0 and entry dispatched. At93.677 seconds the existing trace recorded a
nonzero-PC c0000005 access violation with a nonzero null-adjacent fault address and bugcheck0x1e. The exact owned window was
captured/opened: open-kernel splash, no Fable menu/gameplay. The runner stopped
promptly, acknowledged QMP stop/quit, reaped exit0/forced=false and synced storage.
Earlier Android evidence continued to heap failure/error-dashboard handoff; this
desktop failure is not claimed to be one identical chain.

Because loading evidence was clean and startup still failed, the single authorized
control reused `.local/build-oxos-play/bios.bin`, SHA-256
`668d08af6de204ce07a1479c8854d8f688af312ed6104e8ab56391dffc301fa6`, uninstrumented
c0c887a1263ebf89ba765f34a88ca8f132901e25. `.local/fable-baseline-control-v1`
loaded the title (status0), displayed the same open-kernel splash, then stopped
at128.467 seconds on standard bugcheck0x1e. No structured v4 exception or new
loader-audit record exists in that image; no missing record is treated as a pass.
QMP acknowledged, exit0/forced=false, storage synced. This baseline already failed
on both phones; this fills a desktop control, not a first firmware-free test.
Both baseline and prior A2 use c0, but their patch/trace flags, code layout and
date/time-dependent builds differ. No difference is attributed solely to tracing.

## Checks, preservation and evidenced next boundary

Workspace tests:21 passed,3 explicitly gated native/storage tests ignored; Clippy
with warnings denied and format/diff checks passed. Actual five-case guest checks
are separate from those Rust checks, build success and source review. Exact
build/fixture/run/summary/hash receipts remain under ignored
`.local/evidence/loader-complete-v1`; all scratch and prior evidence remain intact.

Before/after checks match all38 retained artifact identities: old firmware/maps/
receipts, fixture outputs, native core, selected Android assets/selectors and
synthetic template. The old source's v4 diff was verified separately. Full
post-run game hash matches the retained
identity. Neither phone was operated; both verified phone originals and old
profiles remain untouched, with the previous verified device return settled.
No PSX/other-project action, APK build, game patch, public binary release or
proprietary input request occurred.

The corrected completeness boundary is demonstrated; the candidate's preloaded
raw bytes match their declared digests. This narrows the remaining failure to
startup after complete preloads/successful loading, without certifying subsequent
import execution, BSS/TLS/thread startup, hardware interfaces or all platform
semantics. No causal correction beyond the scoped loader defect is established.
Fable remains unplayable here. No stable desktop title/menu or gameplay satisfied
the Pixel gate, so no unchanged phone trial was made. The setup-free/distributable
requirement is binding; proprietary comparison is closed and private game-code
analysis remains paused. Audio, ordinary saving, full resume, shortcuts,
performance and owner acceptance remain unqualified.

Receipt SHA-256: candidate observation
`ab150ca7af711021da812148feffa4957f2eea878886973169b78d0bd84903d3`;
baseline-control observation
`6761813acb259518d8589f61d0c829b05779d470e1c89bab57ca790e2a5a8f04`;
both ordinary termination receipts
`9dfd2c0bd250dd7006f233ed21926e85e38ee7db9709c0cde666fb7b66e31d57`;
the bounded Fable loader summary
`c49c322a96a7fc2fd5f6f2c9e7504e9b819279f0afcebbe881aeda4cb58d6a96`;
identical retained before/after manifests
`d50f67293e9ec874f5ec6da58116586c46636047333f8b9320e0626f876b168b`.

The original handback incorrectly called the desktop fault zero-PC. Exact-field
rechecking of the preserved record corrects that statement; earlier phone
zero-PC evidence is separate. The raw records have not been rewritten.
