# xbx development

## Boot

Read `docs/state.toml`, `docs/github-state.toml`, this file, `docs/PRODUCT.md`,
`docs/ARCHITECTURE.md`, and the active issue plus amendments. Use
`docs/XBOX_FEASIBILITY.md`, `docs/DISTRIBUTION.md` and `docs/OPERATIONS.md` for
relevant decisions. `handoff/MASTER_SEED.md` is the complete initial context.
Live issues own assignments; documents own durable contracts. Check actual
local work and worker acknowledgments before creating another worker.

## Product and authority

The owner requests a public, free-source Xbox sister to PSX named `xbx`, with
separate branding and app identity. Original Xbox is the initial interpretation;
Android ARM64 on the owner's stock Fold was the first desired playable target.
The owner's October 22 continuation prioritizes the accessible stock Pixel 4
for compatibility work; Fold qualification remains in scope.
Xbox 360/later systems, a full iOS port and multiple live games are later work.
Preserve the game-first, direct-launch, durable-return experience in PRODUCT.md.

All substantive new first-party application, runtime, UI, input, save and build
policy is Rust. Necessary mechanical platform/C-ABI shims and targeted edits to
retained C/C++ are permitted. Existing upstream build tools may remain; do not
invent a Python/Kotlin/Java/C++ application layer. The Bash seed/bootstrap is
one-time repository setup, not a competing product build owner.

Borrow mature emulation for a reason, but own the product's engineering.
Targeted core repairs, interface changes, performance corrections, original
boot/HLE work and justified Rust subsystem replacements are allowed when they
solve an observed product need. No mandatory donor tour, untouchable vendor
boundary, whole-core rewrite or gratuitous language translation. A material
new subsystem/research campaign needs Master's scoped decision, not permission
for every ordinary patch. Preserve upstream source lineage and applicable terms.

Start XBX-1 locally. Inspect and qualify the xemu lineage and its Android path;
retain freedom to choose a better evidenced solution. No libretro API, renderer,
core restartability, BIOS-free launch, page size or performance is assumed.
Build one real slice before broad abstractions. A desktop result is not a Fold
result. Continue small in-scope implementation autonomously; do not stop at a
planning memo when source/build progress is available.

## Continuity and separation

GameId, SavedSession, VirtualConsoleProfile and LiveRuntime are distinct.
One live emulated machine initially; multiple durable game sessions. Core-owned
threads stay core-owned. One writer owns each writable virtual disk, EEPROM and
checkpoint group. An atomic rename does not itself provide exclusive ownership.
Xbox snapshots may include disk history: never silently roll back newer in-game
saves or restore RAM against a different disk generation. Validate coherent
checkpoints; preserve failed/incompatible states and offer explicit cold boot
against current durable storage. No PID/task/native handle in permanent IDs.

Use separate `com.machinerhythm.xbx` identity (proposed until verified), signing,
local data and shortcuts. Do not change or install over PSX. Read only a pinned
public PSX revision for reuse; record copied files and notices. No shared SDK or
cross-product runtime framework is required. Do not modify FN64, PSX, Kitten,
studio policy, server facilities or their in-progress checkouts under this task.

## Resources and publication

Prepare off-device. A connected/unlocked phone is not custody. Acquire the
existing authorized stock-device operating window before any device operation.
No root, bootloader changes, system-package changes, hardware firmware flashing,
unrelated app access or weakened security. No spending or paid infrastructure.
Check actual free space, Btrfs metadata/allocation and ongoing jobs before large
writes. Follow the current host custodian; never clean another project's caches,
backups, media or recovery evidence. `.local` is protected, not disposable.

Public source/normal reviewed commits and pushes are authorized for xbx only.
Publish no games, proprietary firmware, EEPROM/key dumps, private saves, device
identifiers, credentials, signing keys or raw owner logs. Original code is MIT;
GPL-derived translations and modifications keep their actual obligations.
No store submission or public binary release before its own release checks.

## Evidence and handback

Record pins, commands, actual outputs and failures. Distinguish source review,
build, installation, boot, gameplay, audio, saves, snapshot resume, performance
and owner acceptance. Focus tests on changed boundaries and protect the last
working candidate. Use synthetic fixtures for destructive/storage fault tests.
Report useful progress on the current issue and a full source/build/data-return
handback. Ask only for a missing private input, device window, material scope
change, unresolved technical/rights question or data-loss risk. Never claim a
worker started merely because an issue was posted.
