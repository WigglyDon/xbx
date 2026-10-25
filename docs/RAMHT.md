# Retained-core RAMHT compatibility boundary

The private reference at `5de6e35` aborted in the retained upstream core's
RAMHT lookup before dashboard navigation. No game ran. The owner's continuation
starts one bounded public-source correction at this boundary; the earlier kernel
and game investigations remain paused.

## Public contract and minimal candidate

The public [Nouveau RAMHT implementation at Linux v6.18](https://github.com/torvalds/linux/blob/v6.18/drivers/gpu/drm/nouveau/nvkm/core/ramht.c)
uses eight-byte entries and derives hash width from their count. Its
[FIFO register setup](https://github.com/torvalds/linux/blob/v6.18/drivers/gpu/drm/nouveau/nvkm/engine/fifo/nv04.c)
encodes the table-size field as hash bits minus nine. For the four ordinary
register encodings, this gives table bytes `2^(12 + size)` and entry-index
width `9 + size`.

The retained `03b988c9` xemu function instead used `11 + size`. With size zero
and synthetic handle `0x200` on channel zero it returns index512, whose
eight-byte offset4096 escapes a4096-byte table. Correct entry-width folding gives
index1. The candidate changes that width by two bits. It retains all lookup
assertions, handle validation and object processing; no final-index mask,
discarded GPU command or proprietary-image change is involved.

`patches/xemu-reference-ramht.patch` is an additive LGPL-2.0-or-later modification
to the retained PFIFO source. `ramht-core.lock.json` binds it separately from the
accepted reference patch and executable. The baseline build, source checkout
and crash profile are preserved. This candidate is not the selected Android
core or a shipping firmware route.

## Original regression

Rust xtask extracts the exact pinned C function and register header, compiles a
mechanical test ABI, then applies the candidate and repeats against that actual
function. Original Rust fixtures check fixed high-handle vectors for all four
table sizes, table bounds and independent bit-plane parity on channels0..15.
They make no new claim about additional channel semantics or complete hardware
accuracy. The test contains no proprietary firmware, game or guest instructions.

The original function failed the selected bounds check and the corrected function
passed all three tests. The initial shim compilation refused a redundant macro;
that complete attempt is preserved. The corrected shim uses the actual upstream
register-header macro. These are arithmetic/interface results, not dashboard,
navigation or game acceptance.

## Separate build and qualification

`cargo xtask ramht-check ROOT LABEL`, `ramht-configure`, `ramht-reconfigure`, `ramht-build` and
`ramht-observe` use the existing shared-volume binding supplied privately by the
operator. They acquire the actual whole-volume and xbx project leases, refuse a
wrong/unmounted/nonprivate directory before creating outputs, and retain the
shared12GiB floor. The candidate has one cumulative6GiB growth envelope beyond
the measured existing xbx allocation. Remaining internal-host guards are intact.

Source exports, all dependencies, compiler targets, temporary files, caches,
logs and new profiles stay in the admitted xbx directory. Exports use pinned
retained Git objects; the two non-Git xxHash wrap archives are checked against
their exact source-pin hashes. Configuration disables downloads and native build
uses one compiler job. The retained DSP library is reused, not rebuilt.
No existing source expansion, executable, profile, media or saved data is moved.

Real wrong-UUID, missing-binding, absent-workspace and occupied-capacity-lease
checks refused before creating fixture output or altering the phase receipt.
The isolated Rust tool passes Clippy with warnings denied and formatting checks;
all paused source is excluded from its build. A redundant-borrow warning was
corrected before that pass. The guard fixture's first absent-volume expected
message was corrected; the actual tool had already refused safely.

The first offline configuration reached dependency/compiler checks but exceeded
its five-minute controller deadline. Owned children were retired and that entire
partial build remains. The explicit reconfiguration route reuses the tested
source exports in a fresh `LABEL-retry1` build directory and uses a finite
fifteen-minute configuration allowance. Storage guards, package pins, source
flags and one-compiler policy are unchanged; the sources are not exported again.

That configuration subsequently reached a concrete layout error: clean Git
dependency exports omit the Meson wrapper files that xemu normally overlays from
its own `subprojects/packagefiles`. The ordinary exporter now supplies exactly
`meson.build` and `meson_options.txt` for Berkeley SoftFloat and TestFloat from
the same pinned xemu commit. It verifies the source bytes, preserves matching
files and refuses to overwrite a differing destination. A wrapper receipt and
the tested PFIFO hash are bound into configured/native identities. The next
explicit retry uses the existing source export and another fresh build directory;
both earlier configuration directories remain. This is build integration, not
a new core/firmware failure or a change to the RAMHT candidate.

Native qualification remains separate: one fresh64MiB offline no-disc profile,
the same private flash/MCPX, retained pristine open-dashboard and EEPROM seeds,
keyboard ControllerS, native FP, OpenGL/llvmpipe and dummy audio. Observe the
owned window and actual navigation, preserve exact build/profile identities,
and distinguish abort from acknowledged shutdown and storage synchronization.
Only a dashboard/navigation pass reopens the assigned unchanged-game checks.
Dummy audio and successful host exit alone do not establish game acceptance.

## Native build and visible dashboard result

The final separate retry built successfully with `ninja -j1` and downloads
disabled. The executable SHA-256 is
`eaefdb4f61d01133557374585383e531438e0341d11579c11e8575283a5ccb9b`.
Its PFIFO source SHA-256 is
`d328116c30bbc211c803471044440dd2caf7898aeb8c5b9fbdb8bf0f2aeb2307`.
All13 retained dependency commits and the DSP library matched their accepted
identities. The four pinned Meson wrapper files are recorded separately. A full
comparison of Meson build options found only `wrap_mode=nodownload` differing
from the original build; assertions remain active. The two earlier incomplete
configuration directories and original reference executable are retained.

One fresh64MiB offline no-disc observation used this executable with the same
private flash/MCPX, pristine dashboard/EEPROM seeds, keyboard ControllerS,
OpenGL/llvmpipe, native FP and dummy audio. Controller attachment was observed.
Six owned-window frames were retained at approximately26,76,116,146,181 and241
seconds. Reviewed frames show the dashboard Main Menu before input, System
Information after normal A, and Main Menu again after B. A was delivered at135.333
seconds and B at170.122seconds. This establishes visible dashboard operation and
the observed navigation sequence. The former RAMHT assertion did not recur.

The300-second observation ended after307.443seconds with a QMP `WouldBlock`
timeout and supervisor-forced SIGKILL. The native child was reaped and storage
synced. **Clean native shutdown and a coherent checkpoint are unqualified.**
The dirty profile is retained without reuse. Immutable executable, private inputs
and pristine seeds matched again. The shared, project and profile writer leases
were subsequently verified released, and the recorded native PID was absent.
No extra native run was made merely to change a receipt or review a capture.

The ordinary Rust task now also supports `ramht-title`, using a different fresh
profile and the same unchanged settings. An optional operator driver receives
the real shared/project lease descriptors, keeping its captures and normal input
under the owning operation's custody. The exact isolated tool snapshot passed
Clippy with warnings denied and formatting; paused source was excluded.

The next reference Fable check requires access to the retained phone-backed
media. Its first admission attempt found the existing Pixel4 transaction
occupied and refused without acquiring custody. The helper's exact-target
identity/profile selection precedes that lock; the task's media/state script
did not run. No media read/transfer, app access, USB-mode change or Fold query
was performed. Earlier failed MTP restoration and later verified custodian
returns remain preserved; no stale undo was applied. No game or matched
open-firmware comparison ran, and no game-compatibility conclusion follows.

After the existing custodian recorded a new verified return, a second admission
acquired the ordinary Pixel4 transaction. The qualified stock identity and exact
retained Fable size of7,825,162,240bytes matched. Current USB mode was ADB only;
host interface metadata independently confirmed no MTP interface. No USB mode
change or transfer was attempted. The current screen-off/nonsecure return was
verified, the transaction released, and its new receipt retained alongside the
historical failed return. No phone application or deployment was involved.

An exact owner amendment is pending for one verified copy to the shared external
workspace: an additional8GiB media envelope, one stock transfer capped at ten
minutes, unchanged phone copies and the existing shared12GiB reserve. This
exceeds the routine device output/command ceilings. The prior6GiB candidate-build
scope does not silently become a game-staging grant. Partial transfers would be
preserved; no overwrite, permanent internal-host copy, new streaming framework,
USB-mode workaround or emulator launch from unverified media is assigned.
The concrete next prerequisite is that bounded staging amendment or an already
supported read-only media view with qualified custody/return.

Don subsequently authorized that exact copy, then a bounded continuation after
its ten-minute transfer deadline. The ordinary Rust runner now accepts only the
explicit additional8GiB envelope, supplied separately by the admitted operator;
its default6GiB candidate bound is unchanged. The combined envelope is14GiB above
the original allocation baseline, with the same shared12GiB reserve, identity,
inode, aggregate accounting and existing shared/project locks. The actual Android
lease is inherited through the transfer subprocess. No USB mode change is needed.

The first stock `adb pull` reached its600-second deadline and all owned children
were retired. Its process completed after610.081seconds, including retirement.
The retained partial settled at5,644,288,000bytes, leaving2,180,874,240bytes.
It is unverified and cannot be launched. A subsequent flush under the existing
storage leases completed; partial and directory durability were recorded. No
recent USB/block/filesystem error messages were found in the bounded kernel-log
check. This does not qualify sustained write performance or hardware endurance.
Pixel4's unchanged USB configuration and screen-off/nonsecure return were
verified and its transaction released; no phone file/app/settings write occurred.
The post-copy source metadata check was skipped on the failed transfer.

The failed attempt exposed a durability gap in the ordinary copy task. The
runner now flushes a retained partial and directory before releasing its storage
leases on failure or accepting its size/hash on success. The continuation uses
stock `tail` into an append-only file descriptor, preserving the retained prefix;
an inherited OS file-size limit bounds the output to the selected image size.
Remote filenames are shell-quoted. A synthetic binary fixture tests actual
stock-tail execution with spaces and an apostrophe, exact suffix length and
prefix preservation. A separate boundary test refuses arbitrary budget values.
Both focused tests, Clippy with warnings denied and formatting passed in an
isolated snapshot excluding every paused path. The initial descriptor-vector
compile failure and all intervening source/build attempts are retained.

Continuation is gated on its explicit1200-second amendment, unchanged input
size, exact retained partial length, fresh qualified phone custody and successful
stock-command availability check. No existing completed image is overwritten.
Full expected SHA-256, exact complete size, storage sync and read-only sealing
remain mandatory before native use. The approved continuation has not yet run;
the latest phone custodian's return must first be reconciled. The deadline is
not extended silently, and a free lock alone does not prove a verified return.

The subsequent custodian recovery/return was verified before a fresh Pixel4
transaction. The approved continuation appended only the remaining bytes with
stock `tail`; the original prefix was retained. It completed after461.059seconds,
including flush and full-image verification. Final length is7,825,162,240bytes;
the full SHA-256 matches the original private media manifest. The file and
directory were synced and the image sealed mode0400. No full internal-host copy
was recreated. Neither phone copy was deleted or written.

Source size/mtime and unchanged USB configuration were checked after this
successful transfer. Screen-off/nonsecure return was observed, the device
transaction exited0, and its lease was released. The new attributable receipt
does not overwrite the historical failed MTP return. Shared/project leases were
also verified released. The next existing peer received the requested bounded
shared-workspace slot before the prepared Fable observation; this coordination
does not claim that peer's execution, capacity or acceptance.

## First unchanged Fable observation

After the peer's actual writer retirement, fresh admission observed no USB I/O
in flight, zero host writeback, ample memory, more than101GB externally available
and released shared/project locks. One fresh64MiB Fable profile used the same
RAMHT candidate, private flash/MCPX, pristine dashboard/EEPROM seeds, native FP,
keyboard ControllerS, OpenGL/llvmpipe and dummy audio. The structural inspector
reported the same7,825,162,240-byte image, partition offset405,798,912, title
`4d53000d`, region5 and version3. The title receipt retained `network=core_default`;
no network-feature qualification was attempted. This differs from the dashboard's
explicit offline command and is recorded as an ordinary configuration gap.

All seven retained PNGs were reviewed. They show the Xbox/Microsoft boot logo,
an advancing armored-character introduction, then the Lionhead Studios logo
with smoke. The introduction was visible before input. Start, D-pad Down and
D-pad Up were delivered later; their images show continued animation, without
a demonstrated menu/control response. The planned A did not execute. **This is
actual title startup/video progress, not menu, gameplay or responsive-control
acceptance, and not a completed compatibility failure.**

The240-second capture helper exceeded its18-second controller limit. A valid
late PNG was preserved and reviewed, but the operator's exception requested
ordinary early retirement. The native observation ended after258.349seconds:
exit0, `forced=false`, QMP acknowledged stop/quit and storage synced. The outer
operator/task exited1 because of the capture failure. Native status and operator
failure are distinct; neither is a300-second game timeout. The native PID and
all shared/project/profile writer leases were subsequently verified released.
The complete profile, seven images, input timings and original receipts remain.

The first-party launcher had forced software graphics. A bounded existing-host
GLX capability check observed accelerated Intel OpenGL4.6 on the same desktop;
this is capability evidence, not a native game result. An ordinary correction
is being prepared to select that host path explicitly, make title networking
explicitly offline, defer capture durability to retirement under retained custody
and keep an optional capture timeout from ending the native observation. The
original executable and interrupted profile will remain unchanged. No matched
open-firmware comparison, second-title observation or phone game trial has run.

The ordinary Rust correction is now built and checked in an isolated snapshot.
`XBX_DESKTOP_GRAPHICS=host` explicitly requests the existing X11 hardware OpenGL
path; software remains the default, and unsupported selections refuse before
profile creation. The ordinary console profile and native arguments now disable
networking explicitly for both dashboard and title observations. Receipts record
the requested graphics path; the native GL renderer log remains the actual
backend evidence. A separate fresh host-graphics title profile preserves the
software trial. No core or firmware was rebuilt for this correction.

The focused supervisor suite passed17tests with3native/tool-dependent tests
still gated; Clippy with warnings denied and formatting passed. The real
synthetic child records its inherited graphics environment and offline NIC
arguments, alongside the profile configuration. Earlier validation attempts
exposed an overlong synthetic Unix socket path and cross-fixture inherited-FD
contention. Short fixture names plus a shorter private temporary directory on
the same admitted volume resolve the path bound; serial execution isolates the
process-lease checks. Both failed attempts and their outputs remain. No runtime
ownership or socket guard was weakened.

The capture operator now measures its interval after input hashing, defers image
durability to retained-custody retirement and records an optional capture timeout
without ending the native observation. The conditional matched-open route uses
the retained exact official OXOSv0.3.3 image in separate fresh storage with no
MCPX. It requires the operator's actual reviewed reference-menu/control success;
setting its attestation is not itself evidence or automatic compatibility
acceptance. The new hardware reference and any conditional open result remain
separate native qualifications.

## Corrected hardware observation and decision packet

The corrected ordinary supervisor SHA-256 is
`8d76f7c7077bc54ec4f6b290906ea3540dde27362dd389afc4bb16c65e2c683b`,
from the isolated source matching `48d9b09`. It reused the same retained native
core and unchanged verified game, in another fresh64MiB profile. Actual native
logs identify Intel UHD620 OpenGL4.6; the receipt records explicit offline
networking, native FP, keyboard ControllerS, private MCPX present and dummy audio.
A separately observed peer compiler means this is functional qualification,
not an idle-host performance measurement.

All nine retained frames were reviewed. They show Xbox/Microsoft boot, the
armored-character introduction, Big Blue Box studio animation/logo, a later
cinematic/scene transition, dim colored imagery and a black final frame.
Start, Down, Up and A were recorded through the owned window. One extra ordinary
Start was delivered near the deadline; it occurred after the last capture, so
there is no follow-up image proving its response. **No visible menu, responsive
game controls, gameplay, audio or save/reload is qualified.** The late imagery
does not establish a guest crash or a cause shared with either open kernel.

This observation completed its full bounded interval:302.537seconds, native
exit0, `forced=false`, QMP stop/quit acknowledged and storage synced. The
operator exited0, retained images/input/control directories were synced, the
core/firmware/seed hashes matched and the verified read-only media metadata was
unchanged. No optional capture timed out in this corrected run. The native PID
was absent and shared/project/profile leases were verified released. Actual
handoff notification was delayed during visual review; the delay was not an
active runtime or writer. The next peer received the verified return. Both
reference profiles remain without reuse and no machine checkpoint was published.

| Boundary | Actual achieved stage | Remaining qualification |
| --- | --- | --- |
| Retained public RAMHT correction | Original regression, separate native build, dashboard navigation and game introduction | Full hardware conformance is not claimed |
| Unchanged Fable/private reference | Advancing title content; corrected hardware run completed with normal retirement | Menu/control progression and then actual play/audio/save |
| Selected second-title candidate | Retained archive identified by label; earlier container inventory preserved | Fresh authorized access, full input verification, Xbox certificate and ordinary reference run |
| Matched open firmware | Exact retained OXOS release and guarded ordinary route prepared | A demonstrated successful corresponding reference control |

Keep the current xemu lineage, working public correction and Rust supervision.
Do not select private firmware for shipping, blame an open-kernel service from
these observations, resume paused investigations or start another speculative
diagnostic matrix. The required positive reference is still missing; the matched
open route was never entered and its success attestation was not set. Historical
OXOS/Roswell results and the resource-interrupted Roswell64 attempts are unchanged.

The concrete next input is the assigned second-title archive, whose only retained
copy is on the owner's Fold. A fresh safe personal-device window is required;
no Fold query or transfer has begun. The archive is2,288,825,769bytes and its
inventory reports a4,112,318,464-byte ISO, about6.40GB combined before small
profile/evidence overhead. An additional8GiB external staging envelope has been
requested separately, retaining the shared12GiB reserve and every original.
No duplicate download, phone expansion, app/save access or phone game trial is
part of that request. After actual admission, verify the existing archive, expand
only its selected image off-device, identify the Xbox structure/certificate and
make the assigned single normal reference check. Only actual menu/control success
opens its corresponding matched open-environment comparison. If that control
also remains unqualified, return the common input/configuration/host uncertainty
for a scoped decision; no private-game/kernel diagnostic campaign follows here.
