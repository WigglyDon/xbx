# Owner-directed xbx retention and media relocation

The [continuation amendment](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6387537066)
and Don's later live instructions authorize xbx media relocation and project
cleanup. This pass continues `579ca7f`; no worker, completed firmware trial or
paused low-level investigation was restarted. No public binary release.

## Media and device return

The selected Fable image was fully size/hash-verified against the retained
identity on both phones. Their existing copies were reused, not transferred
again. After both current verification/return receipts and a no-active-host-reader
check, the exact laptop image was retired. Private source/destination mappings
preserve its identity and the historical path instead of pretending that path
still exists. Writable console disks, EEPROM, saves and checkpoints were not
media-cleanup targets.

The second title's archive was copied directly to Fold, synced and fully
hash-verified there. Pixel 4 refused admission for insufficient space before any
archive copy. Owner-authorized stock cache trimming recovered about 235 MB,
which was insufficient. PCX/GCX media and durable data were left untouched.
The archive label still does not qualify its Xbox structure or edition.

After that capacity refusal, Don explicitly directed immediate removal of the
PC copies and took Fold back. The exact laptop archive was retired under that
later instruction after checking its verified Fold receipt and absence of a
host reader. **The earlier two-phone gate was not met for this archive: Fold
holds its only verified copy, and no Pixel 4 copy is claimed.** The exception
and retained mapping are explicit in the private retirement receipt. No game
was downloaded again or silently copied back to the workstation.

Fold's successful transfer transaction ended locked/screen-off before the owner
revoked its window. No subsequent Fold shell or MTP operation occurred. Pixel 4
was returned screen-off after the media verification, storage inventory and
cache cleanup. A later separate input-access probe has the return limitation
described below; the earlier clean returns do not erase that limitation.

## Reclaimed project working copies

Retirement used exact manifests, hashes, current ownership checks and retained
recovery witnesses. No blanket `.local` purge, source reset, other-project cache
cleanup, filesystem balance/repair, reboot or weakened resource guard.

| Retired material | Retained recovery / evidence |
| --- | --- |
| 1,260 generated APK intermediates, about 2.09 GB logical | Exact payloads in retained APKs or class JARs; per-file SHA-256 and mappings |
| 15 additional unsigned aligned APK stage copies, about 341 MB logical | Matching complete payloads in retained signed APKs; updated witness mappings |
| 5,437 generated native/Cargo compiler objects, about 1.28 GB logical | Final executable cores and shared/static libraries, source/pins/build recipes and hashed retirement inventory |
| Three Cargo incremental caches, about 349 MB allocated estimates | Source, final binaries and check receipts; no profile/checkpoint data |
| Clean Roswell source expansion, about 515 MB allocated reduction | Complete local Git object database at the same pin, verified connectivity and clean status |
| 60 project-owned download repository-metadata files, about 221 MB allocated estimates | Signed RPM archives, dependency identity, extracted tools and download/build evidence |

The known game-media source lengths total about 10.11 GB. These figures are
logical lengths or allocated estimates as labelled, not additive physical
reclaim. Ordinary shared-filesystem free space improved by roughly 11 GB during
the pass. Reflink sharing, allocation timing and peer activity prevent attributing
every byte of the net change to xbx. Exact data/metadata/reserve samples remain
private; no capacity grant is inferred from the increase.

All 184 protected artifact identities still matched after retirement, including
the frozen release/reference core and earlier dirty-profile files. The sixteen
paused paths remain outside this change; their earlier edits are preserved.
All unique evidence, final firmware/core generations, signed/rollback artifacts,
source changes, package archives, signing material and virtual-console state
remain. Unclassified or modified source/cache material was retained.

Roswell's complete source remains offline at
`1569e2e89fb47cc72b9c704a8884f98200432bd4`; only its clean working expansion was
condensed using Git sparse checkout. Before a future source build, budget the
restored data/file-count peak and restore it:

```sh
git -C .local/roswell-v1/source-v1 sparse-checkout disable
```

The later host follow-through applied the same source-representation principle
to the two xbx-owned xemu checkouts. Complete, independently verified Git object
databases remain at Android pin `4967efd3ee594cc236869a45ee4cb396e25af1ac`
and reference pin `03b988c9d76553b33ed712ba00822ffdf49f0e7d`. Git cone sparse
checkout removed 22,283 hash-verified clean working files while retaining every
modified file, ignored dependency tree, Git submodule path, runtime/resource
directory and root file. Full object inventories, all five reference edits and
all fifteen Android edits match their pre-operation identities. No object was
pruned. A synthetic fixture checked clean-source removal, dirty/ignored-file
preservation and exact offline restoration.

The two checkout block totals decreased by 343,289,856 bytes (about 327 MiB).
This is a directory accounting difference, not a claim of exclusive physical
reclaim on the shared filesystem or a qualified capacity window. The retained
reference executable, dashboard seed and Roswell flash still match their pins;
no core/firmware rebuild or native observation ran. Final builds, profiles,
media locations, evidence and the sixteen paused paths remain preserved.

Restore a full source expansion only after budgeting the recorded file/data
peak under fresh host admission:

```sh
git -C .local/src/xemu-android sparse-checkout disable
git -C .local/src/xemu-reference-03b988c9d76553b33ed712ba00822ffdf49f0e7d sparse-checkout disable
```

The flash was not rebuilt or changed. The retained xemu executable and Android
library still provide hardware emulation beneath the open boot/kernel work.
Retiring their rebuildable intermediates does not replace that hardware core
or resume the paused investigations. A later rebuild must recreate those
intermediates under fresh admission.

## Preventing new packaging accumulation

A further owner-requested clutter pass retired 173 generated DSP Rust
object/library-metadata cache files and 3,376 exact unmodified entries from the
task-local Roswell build-tool extraction. All nine original RPMs remain and
match `roswell.lock.json`; their package/header/payload digests were verified.
Regular-file hashes, modes and symlink targets independently matched the retained
packages before removal. Unmatched entries and all final DSP static archives
were preserved and reverified. No system package was uninstalled.

Those three task-local directory block totals decreased by 666,910,720 bytes
(about 636 MiB). This is not exclusive physical reclaim: the net shared free-space
change during the pass was smaller. The retained reference core, dashboard seed
and Roswell flash still match their pins, and the core's dynamic libraries still
resolve. No emulator, firmware build, phone operation or other-project cleanup
was used to verify the pass. The host growth guard remained unchanged and still
refused admission at the final reading.

The extracted Roswell compiler/tool sysroot must be rehydrated from those exact
retained RPMs before any future source rebuild. Budget the extraction under fresh
admission, use a new empty private destination, verify the original payload
identities, and preserve unmatched original entries before adopting tool paths.
The dependency/build identities are historical bindings and were not rewritten.
The accepted flash and ordinary retained-reference observations do not require
that compiler expansion. A no-extraction payload recovery check reproduced the
original M4 bytes; synthetic deletion checks covered changed files, final static
archives, symlinks and parent traversal. Exact manifests and recovery mappings
remain private with the earlier retention receipts.

Rust now exclusively creates a marked APK staging directory and retires only
that owned stage after signing, signature verification and package inspection
all succeed. The final signed APK and signing keys are outside staging. A
missing/empty output, mismatched stage ownership, or symlink stage/output refuses
retirement. Failed builds retain their complete stage for investigation.

An isolated source snapshot excluded all paused edits. **38 Rust tests passed,
3 gated tests ignored; Clippy with warnings denied passed.** New destructive
boundary tests exercise completed-stage retirement, preserved failure/foreign
data, missing/empty output, missing ownership and symlink refusal. No core,
firmware or APK rebuild/installation was performed to test this change.

## Device-backed desktop input qualification

The normal desktop media path requires a regular seekable read-only capability;
an ADB stdout stream does not provide that boundary. No Pixel 4 filesystem was
already mounted. The installed standard GVFS/MTP backend was reviewed at
[1.60.3](https://github.com/GNOME/gvfs/blob/1.60.3/daemon/gvfsbackendmtp.c): it
supports seekable partial-object reads when the device provides them. This is
source review, not a performance or game result.

Under Pixel 4's existing exclusive custody, a bounded probe temporarily enabled
the stock MTP interface and used its exact canonical udev mount identity. The
first probe's legacy USB-property check did not establish physical-interface
state; the corrected check observed the actual MTP interface. An already-mounted
same-target result was then reconciled using the backend's documented canonical
URI. No new streaming framework or full host staging was introduced.

Fable opened as a regular, seekable, read-only file. Bounded reads included an
offset above 4 GB, and the unchanged Rust disc inspector succeeded: image length
7,825,162,240; partition 405,798,912; certificate title Fable / `4d53000d`, region
mask 5, version 3. These are input-access/structural results, not executable
dispatch, gameplay, speed, audio or save acceptance. No emulator was launched.

The owned MTP mount was removed, but ADB was unavailable during the bounded
teardown. A normal recovery transaction refused before action because its exact
USB target was unavailable. The owner subsequently reported that Pixel 4 was
temporarily assigned to another task. **The temporary USB-mode and screen-off
return is not verified, and xbx has no current device custody.** No connection
cause or phone defect is inferred. The current owner assignment takes precedence:
there is no queued xbx operation and no stale USB undo may override that task.
The private custody receipt and project evidence preserve the unverified return.
No fallback phone, shared ADB-server reset, privilege change or Fold operation
was used. Reconcile the current operator/return before later xbx device work.

## Remaining integration prerequisites

The existing private host coordination request was updated in place with the
measured retention effect; no duplicate request/worker. The host custodian's
subsequent serialization direction assigns its allocation/packaging step first;
xbx's next growth slot is not granted. The existing margins remain in effect.
No private-reference firmware asset was downloaded, identified or executed here.

After a fresh device window and admitted capacity, the demonstrated stock media route
can be used under custody without permanent game copies on the laptop. The
private reference still needs actual flash/MCPX identity/suitability and normal
dashboard qualification before its game observations. The second-title archive
additionally needs a fresh Fold window or another admitted input arrangement;
its image remains unexpanded/unqualified. Only a demonstrated positive reference
justifies the assigned matched open comparison and further evidenced integration.
No common OXOS/Roswell cause, shipping firmware, or completed game qualification
is claimed by this retention/input work.

Exact commands, mappings, failures, source identities and disposal manifests stay
private under `.local/evidence/media-relocation-v1` and
`.local/evidence/project-retention-v1`. Historical receipts remain intact.
