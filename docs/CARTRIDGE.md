# Integrated cartridge/session implementation

Master [6400021880](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6400021880)
supersedes diagnostic-first ordering. This is implementation in the existing
checkout, without rewriting history or adopting the sixteen paused paths.

The Rust cartridge path imports a local XDVDFS image once, assigns stable game
and profile identities, supports exact-media relinking and generates isolated
FATX/qcow2 storage. It selects the packaged open runtime automatically; private
reference firmware cannot pass its ordinary resource manifest. The legacy
diagnostic profiles remain intact. There is one saved game session, including
mutable kernel state, rather than separate firmware snapshots or a chooser.

`cartridge/player.rs` owns serialized launch/pause/departure/return policy.
`desktop.rs` is its QMP transport. Native owners stop/drain/capture/load the
complete machine; Rust does not serialize a second machine representation.
The Android transport now uses that same policy. Its private APK is built and
verified; actual Android lifecycle qualification remains pending. A compiled
shared policy is not a device result.

## Resources and state safety

The private desktop package contains the source-built open image, retained
native core and its slirp dependency, a disk helper and resource manifest.
It uses normal host libraries. Fresh application data plus that package and an
original game input boot without referring to a developer's `.local` directory.
This is a private integration package, not a portable/public binary release.
The native resource set is pinned; frontend/disk-helper versions do not themselves
define the machine state ABI. Cross-core restoration is not promised.

Writable HDD/EEPROM generations advance durably before native writes or resumed
execution. Snapshot-container metadata alone is not a new guest-save generation.
Logical disk contents and EEPROM identity are checked as well as the generation.
Quiesced native `savevm` captures machine and writable-disk state. Rust syncs,
copies, hashes and verifies the complete candidate before atomically publishing
the session reference, retaining the preceding accepted checkpoint until then.
Successful previous checkpoint retirement is bounded; failed/interrupted or
rejected candidates remain recovery evidence.

Restore validates resource/media/profile/state bindings and current saves before
native entry, rechecks immediately before loading and before accepting the
attempt, and loads only a private disk/EEPROM copy. Failed native mutation or
teardown therefore cannot flush into authoritative current saves. A newer epoch
or changed ordinary save refuses automatic stale resume, preserving both sides;
explicit restart uses current saves. Newer disk contents are never spliced into
older RAM. Explicit user Pause differs from interruption and is checkpointed.

The current desktop logical-content check uses a sparse read-only raw conversion
and hashes canonical nonzero logical blocks. It does not hash qcow2 metadata as
ordinary progress. Android uses a separate packaged Rust executable and a narrow read-only adapter
to the retained native qcow2 backend, hashing the same logical blocks. Its device
execution and timing remain unqualified. Host reflinks and ephemeral `-snapshot`
are not used.

## Actual original-program proof

Original fixtures A/B use independently built tiny XBE/XISO images. An ordinary
bidirectional COM1 terminal supplies their input, and a reserved synthetic-disk
sector holds a separate durable marker. RAM contains a counter and arithmetic
integrity cookie. These are original-program mechanism checks, not Xbox
controller, commercial in-game save or Fable acceptance.

The tools-v15 product path completed this sequence on corrected native-v3
(the earlier tools-v9 result remains retained), using the packaged open runtime
and a fresh application-data directory:

1. A started at RAM0/save0. Input advanced RAM1, wrote save1, then advanced RAM2
   without saving it; cookie57F090E4. Pause/capture committed the session.
2. The first player/native processes retired normally. A genuinely fresh player
   and native process restored RAM2/save1/cookie57F090E4, then an action produced
   RAM3/save1/cookie6B482CC3 and committed the next generation.
3. Another fresh player/native restored RAM3/save1/cookie6B482CC3. A -> B -> A
   gave B independent RAM0/save0 and then RAM1, while A returned to RAM3/save1
   with its cookie unchanged. The original guest emitted its finite completion.
   Final retirement was normal. All ordinary successful returns have native
   reap/storage sync receipts; the capacity phase completed and released.

The core ran with64MiB, native FP, Intel OpenGL, offline networking and dummy
audio. `-display none` supplies no presentation surface; the native GPU still
retains its required offscreen context. Snapshotting did not wait for a frame.
Two fixtures have SHA-256
`317e10827bc88ae6e5dcb768c242867d7bc7d6dde963ae9e567a9340179c591a`
and `85a173c73aa642fd82f64146149c6a37c38720ac8a020eb03d0c4f34de6c3255`.

Measured corrected-runtime checkpoint operations were5.760s and8.450s; fresh
restores took24.232s and19.401s, including resource/storage validation. The A/B
switch back took11.443s. Sampled native high-water values reached403068KiB and
the Rust player3500KiB. Current original checkpoints occupy approximately5.6MiB
each (qcow2 lengths10.6--10.7MiB plus256-byte EEPROM and750-byte binding record).
These are operation-time process samples, not simultaneous aggregate peaks or
device results. They do not qualify low-friction latency. Earlier tools-v9
checkpoint operations78.468s/53.033s and restores37.114s/39.648s remain evidence;
uncontrolled shared-host conditions prevent attributing a speedup to a patch.

## Supported native correction and focused checks

The first original attempt lacked UART activation; the next exposed an overlong
Unix socket path. Both were fixed in fixture/ordinary launch code. A subsequent
capture succeeded but shutdown failed. Host-only stacks of this original program
showed the desktop main thread waiting for `display_init_sem`: `-display none`
never invokes that frontend's display-init callback. No commercial instructions,
title-memory observer or paused diagnostic chain was used.

`xemu-headless-session.patch` adds the missing no-presentation init/retirement
handshake. Only `ui/xemu.c` was recompiled and existing objects relinked, with
the accepted base/RAMHT changes preserved. Earlier native builds are untouched.
The corrected core SHA-256 is
`554f6164155b673bd97be067d435f98bebe54c73dd0fa556e7480de9035e543f`;
patch SHA-256 is
`0d0e7c5abf05ad3ed268fdef2d7a3467d331960f761d5c77fd61eb6c2ae1e72a`.
The open resource is the accepted serial-off required-read image
`c29e4f5edb36807ef497e0ad8a63a984bea26daa1c075285f496815501f079b8`.

The20s control bound also failed during a later original capture; the finite90s
candidate completed the sequence above. Earlier timeout/forced-return evidence
remains failed clean-return evidence. No timeout is relabelled a native success.

The isolated tools-v11 shared-policy refactor compiled. Six focused original
storage-policy tests passed, then Clippy with warnings denied and formatting
passed. They cover newer-write epoch refusal, metadata-only snapshot changes,
interrupted reference publication, damaged/incompatible state, failed load-copy
containment and pause intent. The native partial-load test subsequently passed
against a private copy of the qualified original checkpoint: QEMU rejected the
missing saved UART section after beginning restoration. Current disk, EEPROM,
accepted checkpoint, pointer and generation were unchanged afterward. Native
retirement was clean. This was an intentionally incompatible original-only
topology; no game/platform defect is attributed to that induced failure.

The tools-v16 exact source compiles; seven focused storage cases, Clippy with
warnings denied and formatting pass. The additional case protects legacy and
failed storage while retiring only a known managed successful predecessor after
durable replacement. A byte-slice Clippy failure on tools-v15 was corrected;
its seven passing cases remain preserved separately.

Actual tools-v16 shared-policy follow-up passed on a private copy of the
qualified original app. Explicit Pause captured automatically; a fresh process
used the persisted last-game target, remained paused and emitted no pending
terminal response until Resume. It recovered RAM3/save1, then reached RAM4 and
committed. A subsequent ordinary guest save wrote4 before explicit Stop. An
injected older checkpoint reference was refused before native restore; current
disk/EEPROM, rejected checkpoint, pointer and generation were byte-for-byte
unchanged by refusal. Explicit Restart cold-loaded RAM0/save4, then new input
produced RAM1/save4. Every normal native return was clean. The original completed
proof and rejected checkpoint remained intact.

## Paused capture preparation correction

Public-source audit found a preparation gap in the prototype's HMP route:
`save_snapshot` calls `vm_stop(SAVE_VM)` while already paused, but `do_vm_stop`
notifies device owners only from RUNNING/SUSPENDED. NV2A SAVE_VM preparation can
therefore be skipped although serialization and post-save callbacks run. The
earlier RAM/disk result alone did not establish those preparations or sound.

`xemu-paused-savevm.patch` applies the retained Android wrapper's narrow
run/pause transition under BQL, restoring explicit Pause after success or error.
The original Rust/C host fixture links the exact public HMP function with
controlled preparation/error dependencies. Its first host link failed for
missing PIC and is retained. The corrected exact-function regression executed:
before, running control passed but both paused cases failed to prepare devices;
after, paused success, paused error preservation and running control all passed.
This host fixture controls dependencies; it is not a full native serializer,
i386 ABI or guest concurrency test.

Native-v3 reused the headless object and recompiled only the HMP unit, relinking
retained base/RAMHT objects. Core SHA-256 is
`b302cafae4f049e6ec61294538acdded43346302322bc092deb8dbe69008ead6`;
paused patch SHA-256 is
`2e2c0938a26487168b8ebebe80442296d7cccd35ee14e3cbc9980bfe5e7aa556`.
Its exact runtime identity gates reuse of original proof checkpoints. The full
fresh-process/A-B-A proof and actual policy follow-up above passed on this
package. A new actual native partial-load fault also passed: restoration failed
on the deliberately omitted saved UART section, current disk/EEPROM/checkpoint/
pointer/generation stayed unchanged and native retirement was clean. Prior
checkpoints are not silently migrated. Sound output and GPU/APU component-value
coverage remain distinct from these original CPU/RAM/disk and owner-route checks.

## Boundaries and preservation

The fixed assignment baseline is11,514,675,200 external allocated bytes, maximum
17,957,126,144bytes (additional6GiB), with the existing shared12GiB floor. No
attempt resets that baseline or spends an earlier allowance. Exact private
commands/source snapshots/build identities, output and failure receipts remain
in the owned `cartridge-v1` workspace/evidence. No game media was duplicated.

Earlier capacity refusals started no dependent test/build or reservation. Later
phases admitted through the same locks and unchanged budget, completed and
released normally. At21:14:49UTC the v16 admission measured12,634,599,424 allocated
bytes,92,976,795,648 bytes available and7,503,441 inodes. This timestamped reading
does not grant a later window or reset the baseline. No peer pause or replacement
scheduler was created.

The single corrected-logging Fable observation still showed only the open-kernel
splash and ended on the existing post-handoff access violation, with clean native
retirement. Both original startup-lifetime cases passed on reference and open;
no termination repair is supported. [Exact results and next evidence boundary](OPEN_STARTUP.md).
The Android candidate described below is built; device lifecycle, normal
controller play, sound and ordinary save/cold reload remain unqualified. No
PSX/sibling mutation, project pause, cross-agent scheduler or routine broadcast
was introduced. All prior media/builds/profiles/evidence
and sixteen paused paths remain preserved.


## Android cartridge candidate, original-only qualification

The Android entry now selects the packaged open runtime, generates isolated
storage and uses the shared `Player` for import/relink, automatic departure,
last-game return, explicit Pause/Resume and recovery. The scoped document URI is
an identifier for a retained read-only FD; it is never reopened as a host path.
Duplicate exact-image imports retain the existing game/profile identities.
Actual host capability proof passed: pathname removal did not invalidate an open
capability, writable FDs and incorrect relinks were refused, and correct relinking
preserved both identities.

The native QMP transport is shared with desktop. Actual desktop policy regression
on tools-v28 passed after extraction: fresh-process explicit Pause/Resume, newer
guest marker4 refusing the old checkpoint without changing either side, and
explicit cold restart to RAM0/save4 then new input. Eight focused storage cases,
Clippy with warnings denied and formatting passed. The eighth case retains
interrupted administrative control files on an explicit recovery before allowing
a new write epoch. This is not automatic disposal of incomplete states.

Android neutralizes input on lost focus/surface and coalesces departure requests.
Restoration starts paused; healthy foreground, focus and surface gates permit
execution unless user Pause persists. One native machine is initialized per OS
process. The alternate private Activity is only a serial fresh-process handoff;
both app/profile flock descriptors remain held until the previous OS process
exits. A joined native thread is recorded separately from a reaped OS process.
Failed native restoration keeps attempted storage private; explicit recovery is
deferred to a fresh process when the old process still holds its writer lease.
The previous qualification namespace and profiles are not migrated or deleted.

The retained Android pin is `4967efd3ee594cc236869a45ee4cb396e25af1ac`.
Only HMP paused capture, the accepted RAMHT repair and the mechanical image-reader
unit were recompiled into a copy of the retained archive. Six missing retained
platform/bridge objects were recompiled for relinking; the final ABI4 bridge was
then rebuilt alone. No SDK download or full-core rebuild occurred. Missing public
headers were restored from retained Git objects in the separate owned source
copy (3,720 headers,34,630,065 logical bytes). Old source/builds remain untouched.
The read-only block adapter remains GPL-derived; substantive hashing, bounds,
package assembly and lifecycle/storage policy are Rust.

Private candidate identities:

- Native core: `9c8b5e1dcf9654d1ec907bc6c70ba97421183a99efa5e2dc8681fd9097cc8c7a`.
- Rust library, tools-v27 source: `587203ea0a7d5536e25498d187fc9d82878549665c20d8d31b4262011f5fa05a`.
- Read-only image helper: `9f77fed23f0b04eb858c544661262fbdd8f31af159aeebda333b2fc2cfca0369`.
- Generated empty qcow2: `04270e68f3a3aea1d3ea6170d40cad3616311f3a4d560914223ae33bf84e6e48`.
- Signed APK: `67fda516c52ee22f00a99b0e54d415d0c1ca6a1b4b7b6ba62891f78f8802ea5f`.

The APK retains `com.machinerhythm.xbx` and its existing signing identity,
arm64-v8a, minimum API29 and target API36. Java/DEX/resource assembly, alignment,
APK signature verification and package/ELF inspection passed. Open resource
`c29e4f5e…079b8` is supplied internally; no firmware picker/private input exists
in the ordinary cartridge entry. The executable helper uses its packaged native
library directory with `$ORIGIN`, not a developer filesystem path. Build/link,
Rust helper, Java lambda and signing-argument failures remain preserved beside
the completed stages. These are build results, not phone execution acceptance.

Master permits only the bounded original-fixture Pixel4 lifecycle check at this
stage. Prepare rollback, verify current activated stock policy/identity and acquire
the existing device transaction before installation. A first admission attempt
was refused by an already-owned physical-device lock, before any app/UI mutation;
its short host phase returned. No peer pause or reservation was requested. Fable
phone trials still require the independent desktop commercial-game gate.


First admitted Pixel4 original-only window: stock identity/build matched, existing
own APK retained and signer matched, cartridge APK updated without clearing data,
and a uniquely named128KiB original image transferred with exact hash readback.
The actual new collection was captured and reviewed. Document search found the
original input, but the private operator refused an ambiguous filename shared by
the search field and result row. The operator targeting rule was corrected;
this was not a native boot or compatibility failure. No guest was launched and no
complete-session/device acceptance is claimed. The app was retired, own processes
verified absent, physical screen OFF and transaction released with no pending
restore. Prior APK/data, new namespace, original input and private evidence remain.


The subsequent admitted continuation reached real scoped import: the collection
published a new original-game identity, generated its game-isolated qcow2 profile
and published the last-game target before native entry. No guest output or native
boot was verified before the app left foreground. User interaction and a picker
keyboard overlap affected the earlier selections, so these attempts do not
establish a runtime compatibility failure. No native repair is justified yet.
The current attempt and legacy data remain preserved. Retrieval must target only
this cartridge entry's new root diagnostic files before attributing the first
launch boundary; the initial operator had targeted the legacy diagnostic folder,
whose retained bytes were not used to resume an old investigation.

Both subsequent windows returned with own app processes absent, physical screen
OFF, the device transaction released and no pending restore. The private candidate
remains installed with app data preserved and the previous APK retained. The
owner's continuation was bounded to the existing30-minute lab ceiling; no Fable
phone trial occurred. Android unsaved-state restoration, Pause/Home behavior,
helper execution and native shutdown remain unqualified. These small original
inputs and app-created storage have not been deleted.
