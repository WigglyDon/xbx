# Architecture contracts

Status: proposed initial design constraints for Master evaluation. They are not
implemented capabilities. Change the design when evidence warrants, preserving
product intent and data safety.

## Owners

A small shared Rust player owns game identities, imports, library metadata,
launch requests, explicit pause, input, session policy and durable commit records.
Thin Android/iOS adapters own actual file grants, lifecycle events, shortcuts,
window/surface access, audio devices and platform notifications. The selected
emulation implementation owns the represented Xbox, CPU/device execution and
engine-internal threads. Actual boundaries follow its supported contracts.

Do not impose PSX's libretro interface or thread-local callback assumptions on
xemu/QEMU. Inspect lifecycle, event loops, vCPU threads, global initialization,
renderer command ownership, audio callbacks and shutdown. A narrow maintained
C ABI is allowed, as are justified modifications or Rust replacements. A desktop
subprocess launcher is useful diagnostic machinery but is not the final mobile
player. QEMU's own threads are not additional user-visible game sessions.

Potential responsibilities, not mandatory crates: `xbx-player`,
`xbx-engine-<selected>`, `xbx-android`, and Rust `xtask`. Create only the boundaries
needed by a demonstrated executable slice. Reuse upstream CMake/Meson/NDK tools;
Rust-first does not mean rewriting compilers or third-party build systems.

## Identity

- `GameId`: opaque local entry identity, stable through display-name edits and
  verified file relinking. Content fingerprints validate media, not replace IDs.
- `VirtualConsoleProfileId`: durable identity for firmware choice, EEPROM and
  associated persistent virtual storage. Default to game-isolated writable
  profiles; intentional shared-console behavior requires a separate decision.
- `SavedSession`: GameId + profile + selected media + compatible checkpoint,
  durable storage generation and explicit pause intent.
- `LiveRuntime`: ephemeral executor association, engine build and runtime state.
  Runtime/thread/process/task IDs and GPU handles never enter permanent shortcuts
  or durable identity.

One validated `OpenGame(GameId)` operation serves collection, cold launch,
Android intents and eventual iOS entrypoints. Same active game reuses its runtime;
new game requests are serialized. Validate the target before disturbing a current
session. Coalesce repeated requests; reject unknown IDs without selecting a
substitute. Use typed messages, not shell interpolation or arbitrary core paths.

## Persistent disk and snapshot consistency: the critical Xbox difference

xemu documents snapshots inside its HDD image and warns that restoring an old
snapshot can hide newer in-game saves [S3 in SOURCES.md]. Do not label that rollback
as successful safe resume.

Prefer one immutable, qualified empty/base disk template with separate per-game
writable overlays/profile data. Audit whether the selected engine's block layer
actually supports the proposed format and overlay/snapshot operations. "Copy on
write" is a design option, not permission to use an unverified filesystem trick.
Use bounded sparse allocation where supported, record logical versus physically
allocated size, and keep base-image identity stable while overlays depend on it.
Do not allocate a full nominal multi-gigabyte disk for every game by default.

A coherent checkpoint identifies, at minimum:

1. Schema, exact modified engine build and state ABI; firmware/boot-route digest.
2. Game/media identity and the selected disc/executable; controller topology.
3. The corresponding writable-disk snapshot/generation and applicable EEPROM,
   virtual-console identity and persistent peripheral state.
4. Complete CPU/device state, pause intent and validated bounded payload metadata.

Quiesce through the engine's actual synchronization mechanism. Complete guest
block writes, capture a matching machine/disk boundary, durably store all required
parts, then atomically publish the checkpoint manifest. Do not describe an
atomic manifest as sufficient if its referenced files are not durable. Retain
the previous committed checkpoint until the replacement has been verified.
Interrupted writes must leave either the prior checkpoint or an explicitly
recoverable incomplete candidate, never a fabricated current state.

One OS-owned writer lease covers every writable resource in the console profile,
including disk overlays, EEPROM and checkpoint publication. Hold it through all
engine threads' final I/O and clean teardown. Process death releases the lease;
a new runtime validates actual state before acquiring write ownership. If a
shared-console profile is later added, its lease is profile-wide, not just GameId.

A checkpoint that names an older disk generation cannot be resumed over newer
accepted saves automatically. Preserve it and the current disk, explain the
mismatch, and allow explicit cold boot of current durable progress. Any deliberate
rollback is a separate informed operation on retained data. Never restore RAM
and arbitrarily splice a newer disk under it: guest caches and filesystem state
can disagree. This differs materially from a PS1 card byte-array restore.

EEPROM identity/key material can affect storage and save portability. Keep it
private and stable, and bind it to the virtual-console profile. Do not regenerate
it on launch, assume cross-firmware save compatibility, or publish it as a fixture.
Export/recovery must describe which disk/profile/firmware components are required;
exporting just an opaque RAM state is not a complete backup.

## Engine lifetime and presentation

Initially keep one live emulated machine. Prove start, pause, snapshot, stop and
start-another-game in the same process if the donor supports that lifetime. If
upstream global state prevents reliable reinitialization, a bounded single-engine
worker-process lifecycle is an allowed integration option. That is distinct from
running or retaining multiple games concurrently and must be measured, not
forbidden because PSX currently has one activity thread.

One runtime owns engine graphics resources and coordinates with the frontend
surface owner. Borrowed vCPU/render threads retain documented synchronization.
Surface loss, folding and rotation are not requests to erase the machine. A
persistent context/surfaceless path is acceptable only when supported; otherwise
checkpoint/recreate safely. Vulkan, EGL/GLES and Metal are different integrations,
not interchangeable names. Preserve a no-readback/low-copy path where possible.

Audio callbacks consume bounded PCM; they do not mutate emulated time, snapshot
state or perform file I/O. Measure rate, buffer behavior and underruns. Pause and
focus loss release input and stop output without relying on an undocumented
Android service exemption. Use actual guest timing and distinguish internal game
FPS, emulation speed and presentation FPS.

## Optional future residency

Many saved sessions are a baseline. Multiple paused resident runtimes are an
optional Android optimization. Simultaneous executing games are separate future
scope. A bounded pool of declared app-process components may eventually host
independent engines; task cards and launcher shortcuts are not processes.
Android may reclaim any cached runtime, so restoration must remain correct with
zero resident workers. No false foreground service to prevent eviction.

Native globals may make one emulator per worker process the correct boundary;
never instantiate two global-state cores in one address space without proof.
Resident profiles must not share writable overlays. IPC authentication, launch
validation, crash handling and budgets are mandatory before a second live worker.
Do not implement a general process pool during initial qualification.

## Separate sister, deliberate code reuse

The proposed app ID is `com.machinerhythm.xbx`; the launch scheme is
`machinerhythm-xbx://play/<GameId>`. Both are xbx-owned proposals until verified in
source/builds. They must not collide with PSX. No PSX app-data access, signing-key
copy, manifest alias or cross-app data migration is implicit.

Pin selected public PSX Rust source, retain notices, and record paths/revision in
THIRD_PARTY.md when copying. Keep console-specific disk policy, input and renderer
adapters here. Do not extract a shared studio framework just to share a few files.
