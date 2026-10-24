# Local work, resources and safety

## One product worker

Use one xbx Codex in its own checkout. Inspect local changes, processes and issue
acknowledgments before starting another. No interference with PSX, FN64, Kitten,
Genie or studio checkouts. A published issue is not evidence of a running worker.
Master owns product/architecture decisions; Codex owns routine implementation,
tests and reviewed commits/pushes. Keep durable decisions in these small documents
and work progress in the actual issue, not an expanding parallel status registry.

## Workstation storage

The owner's development host is shared and has recently required storage repair.
That history is a warning, not a current capacity measurement. Before cloning
large donor histories, downloading SDKs, expanding disks or building, inspect
current capacity and jobs. On Btrfs, free bytes alone are insufficient: inspect
metadata allocation, global-reserve use and device-unallocated space through
existing approved read-only tooling. Inability to read required accounting is
not permission to assume capacity.

Estimate peak extra allocation, including source/dependencies, compilation,
virtual disks, temporary conversions, source bundles and evidence. Reserve the
current custodian's margin and serialize competing heavy work. Reuse existing
authorized SDK/NDK tools without purging or mutating another project's caches.
No privileged filesystem balance, repair, package removal, blanket `git clean`,
`cargo clean`, snapshot expiry or unrequested deletion.

Use sparse/COW game-disk storage only with actual format and filesystem evidence.
Record apparent size and allocated blocks separately. Do not copy multi-gigabyte
game media into every backup. Preserve complete current progress before updates;
a previous APK alone is not recovery of migrated app data.

The seed/bootstrap itself is small and performs no emulator build or SDK install.
It refuses unrelated existing destination contents and does not delete work.
`.local/` remains private/protected. It contains receipts and later owner data,
not an automatically disposable cache.

An owner-qualified independent shared workspace may admit a bounded phase even
while internal Btrfs growth remains refused. Resolve its current private binding
through `resource-discovery`; never copy a remembered mount path into product
defaults. Acquire the existing whole-volume capacity lease and the xbx project
lease, then verify the exact mounted UUID, filesystem, target and private owner
permissions before writing. Refuse a missing or wrong volume without creating
anything under the uncovered mountpoint. Initially serialize heavy phases and
retain the one shared 12 GiB free-space floor plus outstanding growth.

The current xbx reference-preparation envelope is 2 GiB of total new input,
profile, cache, temporary and evidence allocation on that volume. Read retained
internal artifacts in place; do not migrate old profiles or recreate full game
copies. All new writable runtime paths belong in the admitted xbx directory.
Ext4 admission uses its own available blocks and inodes; it does not relax the
internal Btrfs guards or certify the internal allocation problem. Stop/reap owned
runtimes, sync storage and release all writer handles before shared safe removal.
New durable profiles require the same volume and lineage to return.

Owner-authorized xbx retention work may retire identified rebuildable objects,
successful packaging stages and verified duplicates after recording their exact
scope and retained recovery source. Keep canonical source/pins, unfinished edits,
final qualified/rollback artifacts, unique failure evidence and all saved data.
Do not retain multiple expanded copies merely to preserve a source identity:
complete verified Git objects or pinned package archives can retain clean source
offline. Restore an expansion only after budgeting its data and metadata peak.
Record logical/allocated estimates separately from measured shared-filesystem
changes. [RETENTION.md](RETENTION.md) records the owner-directed October 24 pass.

## Device operation

The owner's personal Fold is available only in a fresh authorized operating
window under the existing stock-device procedure. Verify exact identity, one
custodian and current grant; never guess serials or assume an unlocked device is
free. Build and package before holding it. Avoid PSX data and all unrelated apps.
No root/unlock/wipe, package disabling, global graphics/debug changes or hardware
firmware flashing. Ask the owner for required interaction instead of bypassing
lock screens or permission prompts.

Install xbx only with its own package identity. Before replacing a tested xbx
build, preserve actual xbx data, signing identity and recovery instructions.
Respect the authorized window and stop new work on expiry or connection loss.
Checkpoint, restore agreed settings, stop only owned processes and verify the
agreed return state. Report inability to verify rather than claiming a handback.

## Private inputs

Ask for one original-Xbox game the owner actually wants to test, then have them
identify it locally. A filename in a conversation is not rights evidence or file
availability. Do not acquire games/firmware from unofficial download sites.
Only inspect owner-authorized directories; do not scan the whole workstation to
look for firmware or consolidate unrelated libraries.

Generated fixtures and lawful homebrew are useful for mechanisms but do not
replace selected-game acceptance. Games, proprietary BIOS/MCPX, EEPROM/keys,
saves, signing material, raw device logs/captures and private paths stay local.
Use sanitized identities in public issues. Inspect source bundles before publish.

## Keeping evidence useful

Keep one named output per experiment with source pin, host/device class,
commands, expected milestone, actual result and remaining boundary. Do not
publish repetitive raw dumps or call a synthetic test a performance measurement.
Retain last-working artifacts and unique failure evidence. Normal reviewed source
integration is authorized; public binary release, paid resources and store
submission require their own gate. Reconcile rather than force-pushing history.
