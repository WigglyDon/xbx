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


## Attributed Android socket-driver correction

Under Master6407613531, the retained first-launch diagnostics were retrieved
through fresh stock Pixel4 admission before another launch or native change.
The bounded read matched the existing original entry/profile, collected only
its root diagnostic files and minimal storage controls, and preserved all bytes.
Two selected logs were empty; the attributable nonempty log recorded QEMU launch
arguments but no initialization outcome. That absence alone identified no cause.
The read-only window returned with no own process, screen OFF and lease released.

One subsequent unchanged-candidate launch presented the expected recovery menu:
the interrupted session had no healthy checkpoint. The operator selected the
actual **Restart using current ordinary saves** action for this original fixture.
Process-specific output then established native display preinitialization,
followed by QMP argument rejection: **'socket' is not a valid char driver name**.
The process exited before machine initialization. This is an ordinary Android
link defect, not an open-kernel/game compatibility result. No Fable trial or
checkpoint/restore acceptance follows from it. The failed attempt and current
ordinary storage remain retained; the phone returned screen OFF, own processes
absent, helper exit0 and lease released.

The retained archive already contains `char-socket.c.o`; its socket type is
registered only by a constructor, so an ordinary static archive link discarded
the unreferenced member. The new Rust build action
`cartridge-android-socket` extracts that one unambiguous member from the accepted
archive and supplies it directly before the archive in a new link. It preserves
the prior core and all source/objects, compiles no native unit and does not
force-load the whole core. The resulting library contains the checked socket
open/class/connection functions. Native SHA-256:
`0a79043449af4dae36c56b2ab255f176098e96e31b2f582eec2cb473aa86ec6d`;
retained member SHA-256:
`108a22ab9be161354c44c9d05474b9f47b554e0a1c4ed93ee4e8d4b27694d547`.
The separate `cartridge-android-package-socket` action packages that changed
core while reusing the existing read-only image helper and generated seed.
Actual Android guest execution and continuity remain to be qualified.

The changed private APK is built and signature/package/alignment verified with
the same signing identity, package, minimum/target APIs and Java source. APK
SHA-256 `11e3c689a04d7398b6755d9ac557b5d1ded3d8b8ae61706f07a482a8d83a1ad4`;
Rust library SHA-256
`018621488f83365c08fc5b8d0457ee23f6dcf1490453c0bf9261bad236a8e532`.
The image helper and generated seed retain their previous hashes. Exact isolated
tools-v29 source passed eight storage-policy tests, warnings-denied workspace
Clippy and formatting. An initial custody review found an active unreturned
transaction; after its verified return, normal fresh admission became available.
No peer pause, queue or operating window was manufactured from a free host
resource.

The changed APK was then installed on stock Pixel4 through the same-signer
data-preserving update route. The prior APK remained as rollback; the existing
library and last-game records compared byte-for-byte unchanged, and the installed
new hash was verified. Two preceding host signer-output parser refusals changed
no package and returned the phone cleanly. The corrected exact invocation was
tested off-device against both APKs before the successful update.

The existing original entry again required explicit restart using current saves.
That restart passed the former socket rejection but faulted during QMP monitor
initialization, before guest execution. The attributed native stack resolves to
`aio_bh_schedule_oneshot_full` with a null context from `monitor_init_qmp`.
The real `event-loop-base.c.o` is another constructor-only member discarded by
the Android static link. The donor's fallback base lacks the UserCreatable
completion interface that invokes the IOThread initializer; its thread object
therefore has no AioContext. The socket candidate lacks the real completion
callback, while the retained archive contains it. The failed app process exited;
the operator verified own processes absent, physical screen OFF and released
the transaction with helper exit0. No clean guest shutdown is claimed.

The separate `cartridge-android-monitor` action retains that one real base object
in addition to the already-retained socket backend, preserving normal upstream
registration and completion rather than adding a substitute thread mechanism.
It reuses the same archive, compiles no native unit and leaves prior builds
untouched. `cartridge-android-package-monitor` produces a separate private APK
with unchanged helper, seed, signing, Java and application policy. Tools-v30
passed all eight storage-policy tests, warnings-denied Clippy and formatting;
the final library contains both real completion and socket functions. APK
signature/package, ZIP16KiB alignment and all four ELF LOAD alignments passed.

- Native core: `3e77e3f5c2b4ee156ad413db94b5001e05abc50c4dfc4456413f8799aa2e97ba`.
- APK: `61825de67f5f502d68bc4625f25eec6c3fcc2f5c6f58449053413f08f4224a84`.
- Rust library: `735a670d8bccac37d6aa109a4a255bf8a39bf33d9168b1d8d215f4d735b5d4c2`.
- Retained event-loop member: `4f91ca70686024ca70916c8eac1acccd8de481362aab4ff3bc0f77f8c2e408d5`.

Fresh device admission for this second correction reached host/output admission,
then refused because another transaction owned the physical phone. The helper's
normal identity/profile preflight precedes its lock attempt; no XBX operator,
app/profile access, package update or guest trial followed the refusal. The
installed Pixel4 candidate remains
`11e3c689…a1ad4`; Fold remains on its earlier collection-only candidate. The next
necessary result is booting the existing original entry with the monitor
correction, then the assigned unsaved-state/fresh-process/Pause/Home sequence
only after actual guest execution. All prior data and paused source remain;
Fable is still unqualified. No retry loop, peer pause or scheduler was created.

## Original Android guest execution and Home checkpoint

After a later verified return, the monitor-corrected candidate was installed
without clearing data; the existing library and last-game controls remained
byte-for-byte unchanged. Native machine/display initialization and Vulkan
renderer readiness completed. The core reported its touch controller connected
to port1, but the reused HMP readiness probe timed out before guest execution.
The pinned Android donor supplies an empty `hmp_cmds` table and a comparator
that always rejects names. This explains the failed probe and also prevents the
snapshot commands from being reached; it is not a missing controller diagnosis.
The failed attempt returned with own processes absent, screen OFF and lease
released. No guest execution was claimed for that candidate.

The Android transport now checks the pin's acknowledged `x-query-usb` result,
retains the older HMP response for attribution and still requires an actual
Xbox controller before restoring or starting a guest. On timeout it retains the
rejected topology. The new native patch connects only `savevm`, `loadvm` and
`info snapshots` to their existing upstream implementations and restores the
upstream command-name comparator. It introduces no state format or serializer.
The isolated build recompiles only `monitor_stub.c`, reuses pinned Android
configuration headers read-only and retains the socket/event-loop/paused-capture
repairs. The first missing-header build is preserved separately.

The actual-source host routing contract fails against the original stub
(exit31) and passes against the compiled patched source (exit0), including exact
and alias matching, rejected prefixes/unknown names and distinct save/load/list
callbacks. Its controlled host declarations do not qualify Android ABI execution
or snapshot restoration. The initial fixture compile failure is retained.
Eight unchanged storage-policy cases, workspace Clippy and formatting passed in
tools-v31; final builder/contract source passed Clippy and formatting in v35.
Native compilation/linking confirms the populated command table and real
handlers. APK signature/package, ZIP16KiB and all four ELF LOAD alignments pass.

- Installed APK: `bbc4cab9300c432a3003a1e9c60ec7d07729af29cf4b5929fdc73585bad18477`.
- Native core: `a264a31ff20a66f2d09e7220c267e87a574df7ce789c58e40d3ab9267ac26e18`.
- Rust library, tools-v33 source: `8ef5efe7a2ce15fe2eaa32361fd5a5162c113c831b220beeefbf80affa31b4b5`.
- Native patch: `c61d4e30c8b2ff551d2fcbafd26ee60a68dee60d33c8c0a2df16c73983eb9a4d`.

In the next admitted stock Pixel4 window, the same-signer update retained its
predecessor and preserved the existing entry/profile/storage. The operator chose
the presented explicit restart using current ordinary saves. The actual guest
then produced **CART A state=00000000 save=00000000 cookie=31415926**, visible in
the app's status presentation and a captured/opened screen. The ordinary product
path used its internally supplied open runtime and generated storage. The native
USB query independently reported a Microsoft Xbox Controller at port1.3.1; the
legacy HMP probe returned an unknown-command response. This qualifies original
guest execution and controller presence, not Xbox-controller input or Fable.

Android Back was delivered but did not open the product menu. The retained SDL
surface consumes key events before `Activity.onBackPressed()`. Without the menu,
the operator could not advance the original counter/write its disk marker or
choose explicit Pause. No noninitial unsaved-state proof was manufactured.
Home did trigger the shared automatic checkpoint path: its manifest was observed
at generation6 with `paused=false`, bound to the current native runtime and
ordinary disk/EEPROM hashes. The initial-state checkpoint remains retained.
No fresh-process restore or successful Home/return is claimed yet.

Timing is bounded by observations: guest output was observed within24.31s after
the restart action was acknowledged. The checkpoint pointer was absent27.65s
after Home and present by157.27s. These include operator sampling; exact boot,
capture/helper/publication durations are not separately instrumented and no
low-friction latency is qualified. After preserving the checkpoint, bounded
controlled process retirement verified own processes absent, physical screen OFF,
helper exit0 and lease released. This is not a clean native-quit result.

A mechanical Back-key forwarding correction and packaging that reuses the
unchanged native/Rust libraries were initially preserved unbuilt when normal
bootstrap admission refused an occupied shared capacity lease. The completed
runtime/checkpoint work was published separately from that unfinished follow-up.
Next: qualify the Back route, then earn noninitial state plus a
distinct disk marker, explicit Pause, a genuinely fresh process restoring the
same state, a new action/new generation and Home/return. All previous data,
builds, failure evidence and paused investigations remain; Fable is unqualified.

### Back-event adapter candidate

After the host capacity lease returned, tools-v36 built and passed warnings-denied
workspace Clippy and formatting. The Activity now consumes the platform Back key
and forwards its uncancelled release to the existing Rust menu command before
SDL can consume it. Menu contents, game commands, pause intent and session policy
remain Rust-owned. The existing onBackPressed callback still supports the
platform callback route; gamepad Back is a separate input.

The new `cartridge-android-package-back` action compiles the changed Java/DEX
adapter and explicitly reuses the prior package's verified Rust library. It
checks the retained core/helper/seed bindings and Rust-library digest and records
the actual prior Rust source separately from the new build-owner source. No
unchanged native core, Rust runtime, firmware or image helper was rebuilt.
Signature/package and ZIP16KiB alignment checks passed. APK SHA-256:
`cacf0d1d9057238b2fd876810888986a94ef7be50d95e22fcb356f4763956f4a`.
Native core `a264a31f…6e18`, Rust library `8ef5efe7…b4b5`, helper, generated seed
and open-system hashes match the installed session candidate exactly.

Fresh host/output admission for this prepared candidate succeeded, then the
physical-device transaction refused another live owner. The normal helper's
identity preflight precedes that lock attempt; no XBX operator, installation or
app/profile access followed. The installed APK remains `bbc4cab9…8477`, with
its initial-state Home checkpoint intact. Back-menu operation, noninitial-state
capture/restoration, explicit Pause and Home/return remain pending device proof.
No peer pause, reservation or polling loop was created.

### Android original-cartridge continuity qualification

On October 26, the existing-entry sequence under Master6407613531 completed
on the standing stock Pixel4 through the installed finite-recipe custodian.
The previously built Back adapter `cacf0d1d…56f4a` was installed by the normal
same-signer update, retaining `bbc4cab9…8477` as rollback. Existing library and
last-game records compared byte-for-byte unchanged. No rebuild, reimport,
data clear, firmware selection or commercial launch occurred. The native core,
Rust runtime, helper and generated seed remain the exact preceding artifacts.

Android Back actually opened the Rust menu. Its presented original-only
commands produced these guest responses, captured and opened for visual review:

| Stage | RAM counter | Synthetic disk marker | Integrity cookie |
| --- | --- | --- | --- |
| Initial query | 0 | 0 | `31415926` |
| Advance | 1 | 0 | `4498F505` |
| Write marker | 1 | 1 | `4498F505` |
| Advance before Pause | 2 | 1 | `57F090E4` |
| Fresh process, after Resume and query | 2 | 1 | `57F090E4` |
| New action after restoration | 3 | 1 | `6B482CC3` |
| Query after Home/return, without selecting Resume | 3 | 1 | `6B482CC3` |
| New action after Home/return | 4 | 1 | `7E9FC8A2` |

Explicit Pause durably published generation7 with `paused=true`, and the app
visibly showed Paused. Leave game acknowledged native quit, joined the machine
thread, returned native status0 and synced storage. The OS process was independently
observed absent before reopening the same entry. Its replacement had a different
PID and `/proc/PID/stat` start value, restored visible Pause, then returned the
same noninitial RAM/cookie and distinct disk marker after ordinary Resume.
No recovery flag, checkpoint pointer or storage record was edited by the operator.

Home from the running restored session exercised the actual lifecycle path.
Return was requested while capture was still completing; generation8 was
observed afterward with `paused=false`. The same live OS process responded to
the query and next action without an explicit Resume. Final Leave game
published generation9 with `paused=false`; its native record again showed
acknowledged quit, joined machine thread, status0 and storage sync. Independent
OS observation confirmed process death. Native return records conservatively
retain `process_reaped=false`; the separate OS observations supply that proof.
Writer leases remain owned until actual process exit.

These are sampling bounds, measured from action acknowledgement, rather than
instrumented engine/helper durations:

| Boundary | Actual observation |
| --- | --- |
| Explicit Pause publication | Prior checkpoint at4.46s; generation7 by31.62s |
| Fresh launch restoration | Visible Paused by26.06s after start acknowledgement |
| Home publication | Prior checkpoint at38.78s; generation8 by80.19s |
| Final departure | Process present at3.61s; absent by44.78s; generation9 observed by49.10s |

Input-query and image-review delays are included. Logical-disk validation,
helper, native capture and durable publication times and phone allocation growth
were not separately measured. Low-friction latency and mobile performance remain
unqualified. Some UART status updates present a partial line because the current
bridge displays the latest received chunk; the full counter/marker/cookie values
above were visible. This does not qualify commercial graphics, audio, Xbox-pad
input or every native device's snapshot behavior. The retained desktop newer-save
rejection proof is unchanged; this Android run adds no destructive rollback test.

The first private recipe failed during Python imports because its filename
shadowed the standard `operator` module, before any phone command. Its cleanup
failed at that same import. The installed named cleanup-only recovery verified
no XBX process and physical return while preserving both failure records. An
isolated Python invocation then completed the product sequence. No shared helper,
custody record, lock or sibling resource was manually repaired or removed.

The final device recipe completed with27 UI actions and13 captured/opened
screens, own processes absent, physical display OFF, no setting/tunnel changes
and verified return under its held lease. Pixel4 remains booted and wired;
screen-off is not a security-lock claim. Only after device return, a separate
installed workspace recipe preserved and hash-verified the private operator,
receipts, selected diagnostics and captures. Its existing project/capacity guards
and fixed17,957,126,144byte allocation ceiling/shared12GiB reserve stayed intact.
No storage reservation was held while waiting for the phone. All previous media,
profiles, checkpoints, builds, rollback artifacts and sixteen paused paths remain.
Fable's completed observations were not repeated; its compatibility is still
unqualified. This earns the scoped original-cartridge Android continuity milestone.

The remaining Fable boundary is the retained corrected-logging observation:
selection/handoff returns, then a terminal access violation outside the public
kernel map. The two original startup-thread lifetime cases already pass on both
reference and open implementations. Under Master6399143636's pass branch,
no termination repair, repeated splash/fault observation or speculative matrix
follows. Further repair needs an independently identified concrete public startup
contract and original reproducer, or a scoped additional-evidence direction.
The Android result supplies neither a Fable cause nor authority to reopen the
paused title-code, exception-tracing or allocator investigations. Phone availability
and user-managed firmware are not the remaining dependencies.

## Owner-authorized Fold setup

After Master6407613531, the owner explicitly authorized setup on the Fold and
provided fresh connected/unlocked windows. This adds the same private candidate
on that phone; it does not replace the pending Pixel4 first-launch evidence or
authorize a repeated Fable campaign. Earlier transient USB disconnects occurred
before app access and made no package/data changes. The completed window verified
the bound stock user0/API37/arm64 platform with4096-byte pages and updated the
existing `com.machinerhythm.xbx` package through normal same-signer install-r.
The installed APK hash matches `67fda516…02ea5f`; no binary was rebuilt.

The exact prior APK was already retained and hash/signer verified, so it was
reused as rollback without making another duplicate. No app data was cleared or
migrated. The new collection and touch-control layout were captured and actually
reviewed on the active outer display in landscape2342x1080. No game input was
imported or launched during this setup: native guest boot, real controller input,
audio and complete-session restoration on the Fold remain unqualified.

Stock screencap prepended a multi-display warning before its PNG stream. The raw
capture remains intact; only its exact unchanged PNG bytes were extracted for
inspection. An inactive Fold panel can report OFF while the active panel is ON,
so the operator's initial any-OFF result was insufficient as return proof. Final
bounded read-only verification used the physical Display Devices section:
**both internal panels OFF**, keyguard showing and XBX processes absent. That
verification used only tiny local receipts, with no shared-volume access/write,
application launch or setting change. Both the setup and verification helpers
exited0 and released their physical leases before the owner was told the Fold
was released. Prior apps/media/saves, the rollback candidate and private evidence
remain preserved; no security, Bluetooth or connectivity preference was changed.
