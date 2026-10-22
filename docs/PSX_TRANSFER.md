# PSX lessons: reuse the product, not console assumptions

The inspected public sibling source is `WigglyDon/psx` at
`1e4ba86fa0a5fc1470837b6bcc06bd20b6642c11` (observation 2026-10-21).
That binding is a reference, not an instruction to reset an active PSX checkout.

The owner reports a strong PS1 experience: ordinary play and individual-game
resume across app/process/display changes. That is user-reported acceptance,
not proof that xbx, another engine or another platform already supports it.

## Preserve these decisions

Shared Rust product logic, one validated game-launch operation, durable IDs,
explicit pause intent, pointer capture/cancellation, compact game controls,
separate editable layouts, aspect-correct picture and recoverable saves are
useful design precedents. Use automatic technical defaults instead of a public
core-selector/settings catalog. Preserve provenance while fixing the real owner
of a defect, including inside an upstream engine.

The most recent PSX engineering direction permits core changes, boot/HLE work
and justified Rust replacements. Do not resurrect the superseded instruction
that donated internals must remain unchanged or require another engine whenever
a small core repair is more coherent.

## Candidate source to inspect, not automatically copy

At the pinned revision, inspect public `crates/player`, `crates/android`, Rust
`xtask`, `docs/architecture.md`, `docs/formats.md`, `docs/storage.md` and their
licenses. Reuse only verified applicable files with the original copyright and
license. Record the exact paths/revision and local changes at incorporation.
No PSX source is embedded in this seed, and no private source/history is needed.

Input semantics, game detection, state formats, emulated storage, rendering,
threads, audio rates and native-core lifecycle must be qualified for Xbox.
PS1 fixed-size cards, synchronous libretro calls, GLES 3.0 and a restore-then-card
procedure are not an Xbox specification. PSX's disk-space numbers are an observed
starting reference, not authority to reserve the same resources for a new project.

## Independent product boundaries

Use its own repository, app identity, signing material, local files, Home Screen
scheme, game IDs, checkpoints and acceptance. Do not inspect or modify PSX's
private app data, invent an xbx migration into its sandbox, or use a known signing
key merely because both apps have the same owner. Do not interrupt its Codex.

A one-time copy of a few licensed pure-Rust modules can be appropriate. It does
not require a studio-wide shared runtime, a new monorepo or synchronized releases.
A shared package should follow a demonstrated maintenance need and deliberate
ownership/versioning decision, not precede the first Xbox game.

Source: [PSX instructions](https://github.com/WigglyDon/psx/blob/1e4ba86fa0a5fc1470837b6bcc06bd20b6642c11/AGENTS.md),
[storage](https://github.com/WigglyDon/psx/blob/1e4ba86fa0a5fc1470837b6bcc06bd20b6642c11/docs/storage.md),
and [engineering correction](https://github.com/WigglyDon/psx/issues/2#issuecomment-6277288148).
