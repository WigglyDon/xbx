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
The Android transport/lifecycle connection is still unfinished. A compiled
shared policy is not an Android lifecycle qualification.

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
ordinary progress. This helper and the private copy approach require Android
integration/measurement; host reflinks and ephemeral `-snapshot` are not used.

## Actual original-program proof

Original fixtures A/B use independently built tiny XBE/XISO images. An ordinary
bidirectional COM1 terminal supplies their input, and a reserved synthetic-disk
sector holds a separate durable marker. RAM contains a counter and arithmetic
integrity cookie. These are original-program mechanism checks, not Xbox
controller, commercial in-game save or Fable acceptance.

The tools-v9 product path completed this sequence with the packaged open runtime
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

Measured checkpoint operations were78.468s and53.033s; fresh restores took
37.114s and39.648s, including host resource/storage validation. The A/B switch
back took81.675s. These shared-workspace values do not qualify low-friction
latency. A bounded120s sample of the first player/native pair observed VmHWM
3312KiB/411148KiB; that is not a whole-proof aggregate peak or device result.

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

The later tools-v14 candidate compiles, including explicit Stop, protected
retirement of known successful storage generations, and focused follow-up
drivers. The seven-case retention/safety checks and actual shared-policy
Pause/newer-ordinary-save follow-up have not run on that source. The older six
passing tests remain attributed to tools-v11. Later automatic-return edits
remain unfinished separately.

## Paused capture preparation gap and pending correction

Further public-source audit found a preparation gap in the prototype's HMP
route: `save_snapshot` calls `vm_stop(SAVE_VM)` while already paused, but
`do_vm_stop` notifies device owners only from RUNNING/SUSPENDED. NV2A's SAVE_VM
preparation can therefore be skipped even though serialization and its post-save
callbacks run. The earned original RAM/disk restoration does not establish
complete GPU/APU preparation or sound-state correctness.

`xemu-paused-savevm.patch` prepares a narrow correction using the retained Android
wrapper's run/pause transition under BQL, restoring explicit Pause after success
or error. The original Rust/C host fixture links the exact public HMP function
with controlled preparation/error dependencies. Its first host link failed for
missing PIC; the corrected `-fPIC` build task compiles, but the fixture itself
has not executed. No before/after test success or corrected native image is
claimed. The native follow-through reuses the headless object and recompiles
only the HMP unit; it is gated on the original regression.

Next admitted work is the prepared paused-capture regression, its single-unit
native repair, fresh integrated original continuity/retention checks, and the
actual shared-policy Pause/newer-save check. Clean and failed profiles remain
preserved; no checkpoint is silently migrated to another core identity.

## Boundaries and preservation

The fixed assignment baseline is11,514,675,200 external allocated bytes, maximum
17,957,126,144bytes (additional6GiB), with the existing shared12GiB floor. No
attempt resets that baseline or spends an earlier allowance. Exact private
commands/source snapshots/build identities, output and failure receipts remain
in the owned `cartridge-v1` workspace/evidence. No game media was duplicated.

At18:58:46UTC, the actual capacity lease was busy, with the xbx project lease and
all three qualified app/profile leases reacquirable. No owned runtime/build job
remained. Read-only accounting measured12,287,623,168 allocated bytes, an increase
of772,947,968bytes;93,440,376,832 bytes and7,504,252 inodes were available. Those
readings do not grant capacity. Repeated admission refusals started no dependent
test/build and created no reservation, peer pause or replacement scheduler.

Fable's accepted open result remains splash-only; no new Fable result is claimed
here. The corrected-logging observation and scoped startup-lifetime comparison
are deferred until the first complete storage-safety slice, then continue under
Master's bounded public-platform authority. Android lifecycle integration and a
private candidate, normal controller play, sound and ordinary save/cold reload
remain unfinished. No phone, PSX/sibling mutation, project pause, cross-agent
scheduler or routine broadcast occurred. All prior media/builds/profiles/evidence
and sixteen paused paths remain preserved.
