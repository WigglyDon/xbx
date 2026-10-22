# Unchanged-game desktop comparison

Master [6352032192](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6352032192)
assigns ordinary execution of the unchanged selected game, keeping private
game-code/disassembly analysis paused. The existing worker acknowledged at
[6352057981](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6352057981)
on clean source `baef6de6879402e3e0f9ed22604f4912291e05d2`. The completed
VM-import and quick-relaunch slices are not repeated.

## Ordinary launch boundary

`xbx-host run-title ENGINE NEW_SCRATCH FLASH HDD EEPROM MEDIA SECONDS [MCPX]`
uses only a new private scratch root. The source-built desktop core is retained;
RAM stays 64 MiB, TCG, OpenGL/software Mesa and dummy audio. The default SDL
driver is offscreen; `XBX_DESKTOP_VIDEO_DRIVER=x11` selects an ordinary window.
The new
Rust runner copies firmware/disk/EEPROM into independent storage and holds the
OS writer lease through actual native reaping and storage sync. It generates a
new dirty epoch before native entry. It does not load a machine snapshot.

Media inspection uses the existing bounded XDVDFS/certificate metadata parser.
A retained read-only file capability is duplicated into QEMU's existing FD set;
the desktop DVD filename uses QEMU's JSON raw block view at the validated
partition offset. No game extraction, repacking, conversion, writable media
handle, game-instruction inspection or guest-memory marker is used.

The finite observation budget is 15–300 seconds. The initial A attempt used 180;
the corrected visible A/B comparisons use 300 and stop promptly on terminal failure.
Existing serial runtime status and the ordinary frontend window are observed.
Explicit startup-error dashboard requests or bugchecks stop observation promptly;
an exception alone does not. QMP stop/quit is acknowledged and the transport
stays alive through actual exit. Forced termination is recorded honestly.
Native logs are capped at 4 MiB, serial observation at 1 MiB. The retained core
has CONFIG_PIXMAN disabled, so generic QMP screendump is unavailable. After that
concrete integration finding, the corrected runner uses its ordinary X11 window;
the private operator selects the exact native PID/window and captures only that
window. No native rebuild or new guest diagnostic was needed. Capture output
requires separate visual review; command success never certifies gameplay.
Dummy audio is not audio acceptance.

## Fixed comparison inputs

- Retained core: fork `4967efd3ee594cc236869a45ee4cb396e25af1ac`, ABI3,
  executable SHA-256
  `51178df74f0b26316c799ff8e8c1a65b1b841b97858cd07dd31903d52ebdcd0d`;
  working patch
  `26027ec1436fb6c5f7a51200bf7fe96ceb90ba5695d568a72db29ef446ff461f`.
- A: retained Android-equivalent v4 open image, SHA-256
  `0cdedf6d81dabdbeafc1cff78039cf78f19cb7923078f201724e346ca94ee8e8`,
  OXOS `c0c887a1263ebf89ba765f34a88ca8f132901e25`.
- B: accepted retained d87 open image, SHA-256
  `f5cac7bac71d909772db7be4c73fedbfe935c9ab8001ebd69bc8b79c61de0258`,
  OXOS `d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2`.
- Both use unchanged v4 patch
  `7eb48fb10ca5784a887d93db23f02c07ec83dc7b105930e1a71204724e0a0dc5`
  and Cromwell `86f547387184d1001d377cce97d3756c8acf91cc`.
- Common generated empty FATX qcow2 seed SHA-256
  `0f123236d5916e5c455345c4b91cb9876ae821766475e69ede205c8fd9ae2e71`;
  the same synthetic EEPROM template is independently copied for each run.
  No owner save/profile is used.
- C is conditional on already-owned reference boot inputs. The documented
  `firmware-private/` input location is absent; no suitable reference flash or
  MCPX is inventoried. No unrelated project/folder search or firmware acquisition.
  The [conventional required-files contract](https://xemu.app/docs/required-files/)
  requires appropriate boot ROM/flash/disk inputs; a retail BIOS alone is not
  presumed sufficient.

The [original Fable community report](https://xemu.app/titles/4d53000d/) uses
xemu0.8.130/a672d723b8156a5d5ea9ee7e50faa9b999535460 on Windows and does not
specify firmware. It is not evidence for these configurations or Android.
The owner's edition is established from actual selected media metadata below.

## Input recovery and actual results, October 22

The original PC copies/archive were absent by the retained removal receipt.
Host preflight passed for 8 GiB growth: 45,696 MiB available, 1,422 MiB usable
Btrfs metadata, emergency reserve unused, approximately 6,236 MiB RAM available.
Other host jobs were untouched. No cache/project cleanup was performed.

The activated Pixel 4 lab policy, latest accountable operator return and pending
recovery were checked before supported exclusive USB custody. This dispatch's
one-image transfer is the specific size authority beyond the generic small
fixture ceiling. Exactly one complete 7,825,162,240-byte ISO was copied read-only
from the named shared-storage source into private task storage. Source and host
full hashes matched the retained identity shared by both verified phone copies;
source size/mtime/inode remained unchanged. The host image is mode0400, shared
read-only by the experiments. Its full hash still matched after all runs.

The first host-only admission used the wrong locator field and refused without
a device query. A corrected admission verified the source but hit a host file-
permission API error before transferring bytes; its zero-byte partial/receipt
were preserved and its phone return verified. The corrected transfer used a
separate task path and fresh normal admission, not a control override. It
completed and returned Pixel 4 booted/USB connected/physical screen OFF, with
no app launch, settings changes, owned clients/tunnels or pending recovery.
No security-lock claim is made for the nonsecure lab unit. The helper lease
released normally. Fold was not queried; its verified original remains untouched.
No phone profile was accessed or firmware/package replaced.

Actual certificate metadata reports **Fable**, title ID **4d53000d**, region mask
00000005, version3. This matches the original Fable title identity, rather than
an edition guessed from the filename. It does not certify authenticity or every
payload byte. XDVDFS partition offset is **405,798,912 (0x18300000)**; the existing
parser read metadata only, not game instructions.

| Run | Observation | Actual native boundary |
| --- | --- | --- |
| A1, v4/offscreen, 180-second maximum | Entry dispatch, existing runtime records zero-PC access violation c0000005 and bugcheck0x1e; stopped at88.894s. Generic QMP screenshots unavailable. | QMP acknowledged; exit0, forced=false, storage synced |
| A2, v4/ordinary X11 window, 300-second maximum | Same entry/startup-failure boundary; stopped at105.964s. Own-window image reviewed: open-kernel splash, no Fable menu/gameplay. | QMP acknowledged; exit0, forced=false, storage synced |
| B1, retained d87/same X11 settings/template, 300-second maximum | Same entry, zero-PC access violation and bugcheck0x1e; stopped at92.710s. Own-window image reviewed: open-kernel splash, no Fable menu/gameplay. | QMP acknowledged; exit0, forced=false, storage synced |
| C | Unavailable: no inventoried already-owned reference BIOS/MCPX bundle; owner subsequently confirmed none available. | Not run |

A2 is the single focused repeat to correct the demonstrated screenshot integration
limit, not another firmware/configuration campaign. A2/B1 use identical core,
media view, RAM, synthetic disk/EEPROM templates, graphics/audio settings and
observation policy. Their difference is the exact firmware plus independent
scratch storage. All runs stopped on the terminal status rather than consuming
the full loading budget. Serial logs were2,773 bytes each; native logs6,210 /
6,104 /6,104 bytes, below their bounds. This is startup failure, not merely a
black-screen inference, renderer acceptance or gameplay.

The phone's early failure therefore also occurs off Android on this retained
fork with these two OXOS images. This neither identifies the responsible kernel/
emulator subsystem nor contradicts an upstream report using a different build
and unspecified firmware. The d87 mapping correction did not produce a playable
Fable route here. No game-specific repair or new exception instrumentation was
attempted. The firmware/self-test regressions remain complete and were not rerun.

## Reproduction and receipts

Keep the recovered game path private. With retained inputs, run A2 then B1 using
different nonexistent short scratch roots:

```sh
cargo build -p xbx-host --offline -j1
XBX_DESKTOP_VIDEO_DRIVER=x11 G_DEBUG=fatal-criticals \
  target/debug/xbx-host run-title \
  .local/build-desktop/qemu-system-i386 /new/private/scratch-A \
  .local/build-oxos-launch-7eb48fb10ca5/bios.bin \
  .local/profile-build-0cdedf6d81dabdbeafc1cff78039cf78f19cb7923078f201724e346ca94ee8e8-fatx-v2/assets/hdd.qcow2 \
  .local/quick-reboot-d87cfc33-run1/eeprom.bin "$AUTHORIZED_MEDIA" 300
```

For B use `/new/private/scratch-B` and replace only the flash argument with:

```text
.local/build-oxos-d87cfc33ceb493711b545e83d5b7fd32f1cdf0c2-86f547387184d1001d377cce97d3756c8acf91cc-trace-7eb48fb10ca5784a887d93db23f02c07ec83dc7b105930e1a71204724e0a0dc5/bios.bin
```

The executed A2/B1 Rust runner SHA-256 is
`20e7216e3b05cbc707adf54d05d1b2dbc191b39eae63d62c034cc82a264cef1c`.
Its exact source is `471f6cb4ccf0de439390042a02b2c2e70ca1c4f6`; a rebuilt
binary from that source matched the executed digest, and the executable is
retained privately. A subsequent source-review correction moves fallible native
PID-receipt publication inside the native termination/reaping/sync path and
updates CLI usage. This is host cleanup policy, not a Fable repair; workspace
checks were rerun, with no further game trial or Android acceptance claimed.
The final source-review build SHA-256 is
`f6aaeeae2e5f03c31b7356cc1e7347f258361806323714c31635abb59319444e`;
it is distinct from the actually executed A2/B1 artifact above.
A1's earlier preparation build was
`f5ae943ff6a03ca17e8118f656c923376d1f44a6165ee3f6adcce3341e7daee8`.
The ordinary runner and JSON-view policy are original MIT Rust. Existing QEMU/
xemu/OXOS/Cromwell lineage and licenses remain as recorded in `THIRD_PARTY.md`.
No public binary, game, firmware, EEPROM, save, capture or raw owner log is released.

Local scratch `.local/fable-desktop-A1`, `A2`, `B1` remains intact. Each records
config, media metadata, dirty epoch, native/serial log, observation and termination.
Private `.local/evidence/fable-desktop-comparison` retains input inventory,
copy/failed-attempt receipts, run commands/output, summaries and complete hashes.
Full game/source hashes and synthetic EEPROM identity stay local. The receipt
separates native command success from failed game acceptance.

Observation receipt SHA-256: A2
`8f7311bb7c501cadde1d66c7dde119656f67b1764d9d7522b068addcbebd0d65`, B1
`9d075b7318ba59e84877268b9a7619b9beca1e9e96dab60a852ae5d1a747a9f1`.
All three native termination receipts hash to
`9dfd2c0bd250dd7006f233ed21926e85e38ee7db9709c0cde666fb7b66e31d57`.

Workspace checks: **21 passed, three gated native/storage tests ignored**;
the existing raw-view integration test additionally passed explicitly, now covering
the desktop JSON filename form and read-only behavior. Clippy with warnings denied,
format and diff checks passed. A new synthetic boundary test proves refusal to
reuse an existing profile, preserving all predecessor files/epoch and avoiding
native entry. No destructive/storage test used owner data.

Before/after hashes verify **38/38 retained artifacts unchanged**: all previously
inventoried firmware/maps/receipts, fixture outputs, native core, Android selectors/
selected assets, the d87 image and common synthetic EEPROM template. Both phone
originals and existing phone profiles remain preserved; only the Pixel source
was read. Owner private evidence was not reopened for game-code analysis.
The identical before/after manifest SHA-256 is
`d50f67293e9ec874f5ec6da58116586c46636047333f8b9320e0626f876b168b`.

**No demonstrated working route exists for Pixel 4 continuation.** No Android
build, install or game trial was performed. The owner confirmed no reference
BIOS/MCPX bundle is available and reiterated the product requirement: a
distributable, setup-free environment beyond supplying a game, as intended for
the sister product. C was an optional diagnostic, not a proposed product
dependency. No proprietary acquisition is requested. These results leave an
unattributed public-platform compatibility defect; the next concrete scoped
kernel/core investigation needs Master's decision rather than an unsupported
patch or broad campaign. Private game-code/disassembly remains paused. Gameplay, audio, ordinary
saving, full resume, shortcuts, performance and owner acceptance remain unqualified.
