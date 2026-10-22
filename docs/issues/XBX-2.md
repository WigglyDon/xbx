# XBX-2: Game-first Rust player and durable Xbox sessions

## Status

Prepared successor to XBX-1. The same worker may advance this scope once a real
engine integration is viable; inspect live Master amendments. No duplicate worker,
engine rewrite or shared studio platform is implied.

## Outcome

The existing game-first PSX idea in a separate xbx application: local collection,
comfortable play surface, direct-game Home Screen entry, safe switching and
reliable per-game return across process death and display transitions.

## Scope

- Implement stable GameId, SavedSession, VirtualConsoleProfile and ephemeral
  LiveRuntime separation in actual Rust code. Route collection, import, cold/warm
  launch and pinned shortcuts through one validated OpenGame operation.
- Detect supported Xbox content structurally. Support scoped file/folder access,
  explicit imports and content-verified relinking without changing game identity.
  Do not modify owner source media in place or scan without granted access.
- Provide aspect-correct, camera-safe outer/inner portrait/landscape layouts with
  non-overlapping controls by default. Add usable twin sticks, analog triggers,
  Xbox digital/pressure channels and Black/White semantics. Preserve captured
  simultaneous touches and explicit pause through interruptions.
- Use the engine's actual pacing/graphics/audio ownership. Keep technical choices
  out of the ordinary UI. No fake 60-FPS target or buffered sound replay.
- Implement durable Xbox HDD/EEPROM profile ownership and coherent machine/disk
  checkpoints as specified in ARCHITECTURE.md. Default to independent game storage
  rather than rolling back a shared disk from another game's checkpoint.
- Keep immutable base storage and bounded writable deltas only where verified by
  the engine. Protect disk/EEPROM/state identities and retain recoverable prior
  checkpoints. Older snapshots must not silently erase newer in-game saves.
- Preserve explicit restart with current durable storage, snapshot incompatibility
  recovery, missing-media recovery, process-death lock recovery and update safety.
- Add Android pinned game icons with real launcher confirmation. Cold and warm
  entry, repeated delivery and another active game must preserve progress without
  duplicate runtime owners. Retain a portable iOS launch contract, not a fake port.
- Design local progress export/import around actual profile/storage dependencies;
  implement a safe minimal route if needed for testing/recovery. No cloud required.

One foreground runtime initially. More than one saved game is not more than one
resident process. Do not add a misleading multiple-games toggle or silent keepalive.
A bounded worker process needed for safe engine reinitialization is allowed and
is not a resident-session feature.

## Acceptance

Read docs/ACCEPTANCE.md. At minimum: actual gameplay/sound/input; ordinary HDD
save loaded after a fresh process; separate full-checkpoint resume; game A -> B
-> A with each progress retained; folds/rotations/background/focus loss; cold/warm
shortcut taps; update/relink retention; isolated stale-state/storage-failure and
killed-writer recovery. Record real emulator speed, pacing and resource use.

Generated fixtures do not establish gameplay. An in-game save is not full-session
resume. A whole-machine snapshot is not safe if it silently rolls back another
storage owner. No source-only or boot-only completion claim.

Retain source/build provenance and public-source updates. Phone work still requires
fresh custody. Keep xbx and psx identities, private data and signing separate.
Master accepts the complete experience and records exact remaining release gates.
