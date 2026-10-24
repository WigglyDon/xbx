# Ordinary open-dashboard qualification

Master [6380344397](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6380344397)
replaces the unfinished allocator investigation with ordinary application and
configuration qualification. Existing checkout HEAD was
`5c3dd767a55ebe2a0ec7fdf5a8c85ed50082dc68`; the completed classification result
in STARTUP_TOTALS.md was reconciled without rerunning it. Actual host process/cwd
and lease checks found one xbx Codex, no active xbx native/build job. No worker
was spawned. Sixteen unfinished files plus their diff were copied privately;
all 79 predecessor artifact identities matched. Prior investigations stay paused.

## Inputs and runner

Exact [dashboard release v20260516-0955](https://github.com/xemu-project/xemu-dashboard/releases/tag/v20260516-0955),
source tag `8cb5e525a8d2c980a4daa6b2c2106b53661b5b20`:

- Disk 1,638,400 bytes, SHA-256
  `00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd`.
- Complete bundled LICENSE 60,240 bytes, SHA-256
  `3c4d8c95fdcd47f61e106f1e597d8c9db37a57193d0239346edcbc9f4f2334f6`.
  Component/font notices are retained verbatim under LICENSES.
- Settled desktop core 51178df74f0b26316c799ff8e8c1a65b1b841b97858cd07dd31903d52ebdcd0d,
  fork 4967efd3ee594cc236869a45ee4cb396e25af1ac/ABI3.
- Same classification firmware 5ac9faccb346a9996ec40227249562b81b46d4417a1fecac6abc1502785f8ce1,
  OXOS d87/Cromwell 86f/full patch 80cf64f0bacf4c700be055fd7722017001a3f580d313f48cae7a883598cbccff.
- 64 MiB, TCG/native floating point, OpenGL/llvmpipe, X11, dummy audio,
  generated EEPROM template, port1 keyboard-bound Controller S.

Original Rust runner now exposes
`run-dashboard ENGINE NEW_SCRATCH FLASH HDD EEPROM SECONDS [MCPX]`. The optional
MCPX uses the same isolated profile-copy path as `run-title`; omitting it retains
the open-firmware boot route. It does not select or supply a firmware default.
The observation receipt records whether a boot ROM was supplied. No disc is opened,
inspected or passed as an FD; dvd_path is empty. No-disc qualification has
`-nic none` and automatic update checks disabled. Title observations retain
the settled core network default to match the preceding classification run.
Independent profile copies become mode 0600 even when a pristine seed is 0400.
The pristine disk stays read-only. Existing profile refusal, profile lease,
durable dirty epoch, actual native reap and storage sync remain intact.

A separate small build source contains settled main plus these ordinary runner
changes, excluding preserved unfinished allocator code. No core/firmware rebuild,
new instrumentation, title-code inspection, controller framework or SDK is used.
Preflight had about 26 GiB available, metadata 65.76% used and global reserve unused;
the isolated Rust build target occupied 145 MiB. No cleanup was performed.

## Visible dashboard result

Fresh `.local/dashboard-no-disc-v1` booted the disk's dashboard, visibly showing
Main Menu, System Info selected, Tray State: No Media and FTP Server: No Link.
Clock changed from 06:28:05 to 06:28:21 and beyond; background geometry animated.
These are dashboard frames, not the kernel splash. Normal keyboard mappings A
and B opened System Information and returned to Main Menu. Owned-window images
at 06:28:52 and 06:29:00 establish that round trip. The core's ordinary USB inventory
independently confirms Controller S attachment.

The private operator identifies exactly the native PID and its unique X11 window;
only that window is captured. Initial XTest/focus attempts did not activate the
menu. Direct ordinary X11 key press/release events to the owned window succeeded
through the existing SDL keyboard mapping. No remote desktop, whole-desktop
capture, dashboard settings change, installer, updater or network transfer was
used. System Info contains generated console identity; captures stay private.

The exercised dashboard starts, renders, continues activity and accepts input.
Its 300-second observation completed at 300.173 seconds with acknowledged QMP
stop/quit, native exit 0, forced=false and storage_synced=true. Its profile and
build leases released before the Fable process was started.
Displayed disk-utilization figures were not independently qualified. Dummy audio
does not qualify audible output. This dashboard is an internal compatibility
control; the intended product remains direct-to-game.

The executed dashboard supervisor hashes to
`8cf6e0c9e7627b4dc3df3d6a1595ca9b1b11d4d625b6fc394a1c5dfbe9073ada`.
Before the title observation, the network override was narrowed to no-disc runs
so title networking matches the preceding classification control. No-disc
runtime configuration is unchanged; the final supervisor hashes to
`2ec736f61c0eb56fc743e15a64510d48f45d4efa976eff1073b3dcf0f459c97e`.
Both executable/source variants remain private.

## Unchanged Fable result

Exactly one run used fresh `.local/fable-dashboard-seed-v1`, independently copied
from the same pristine dashboard disk. Core, classification firmware, RAM,
generated EEPROM template, Controller S, renderer, floating-point policy and
read-only disc view matched the completed classification control. The changed
environment is the dashboard disk instead of the generated empty FATX seed.
Title networking retained the core default, as in that control; the no-disc
dashboard alone was forced offline.

The disc was present at cold boot. The existing launcher selected the disc,
loaded its default executable and dispatched entry directly. This was not a
dashboard-to-disc launch. The owned-window images at approximately 17 and 52 seconds
show only the open-kernel splash. No Fable menu, game frame or gameplay was
observed. Existing ordinary runtime status subsequently reports an access
violation and bugcheck 0x1e; the runner stopped at 118.778 seconds with
reason=guest_terminal_startup_failure. A later capture correctly refused because
the native process had already exited. No post-exit window or unrelated desktop
was captured.

QMP stop/quit acknowledged, native exit 0, forced=false, capture_failure=None,
storage_synced=true. No diagnostic map/capture was requested. Adding the ordinary
dashboard environment did not produce a playable Fable route. The working
dashboard demonstrates its own startup/rendering/input only; this run does not
establish a cause of Fable's failure. Prior low-level investigations remain
paused, and no additional game run was made.

Both profiles and all captures/logs are retained. Gameplay, audible sound,
in-game saving, checkpoint resume, performance and owner acceptance remain
unqualified. The desktop gate for a phone trial remains unmet.

## Reproduction and preservation

```sh
XBX_DESKTOP_VIDEO_DRIVER=x11 G_DEBUG=fatal-criticals \
  "$HOST" run-dashboard "$CORE" "$NEW_DASHBOARD_ROOT" \
  "$CLASSIFICATION_FLASH" "$PRISTINE_DASHBOARD_DISK" "$EEPROM_TEMPLATE" 300
```

Then, only after successful navigation and actual native termination, use a
different nonexistent profile root for the single unchanged-title observation:

```sh
XBX_DESKTOP_VIDEO_DRIVER=x11 G_DEBUG=fatal-criticals \
  "$HOST" run-title "$CORE" "$NEW_FABLE_ROOT" \
  "$CLASSIFICATION_FLASH" "$PRISTINE_DASHBOARD_DISK" "$EEPROM_TEMPLATE" \
  "$AUTHORIZED_MEDIA" 300
```

Neither command requests a diagnostic capture or loads a machine checkpoint.
Final workspace checks on the isolated source: 32 passed, 3 gated tests ignored;
Clippy with warnings denied, format and diff checks passed. The existing
profile-preservation regression now covers no-disc launch as well as title launch.
The complete imported notice retains its original mixed line endings; its
scoped .gitattributes entry preserves exact bytes and the existing patch rules.
The retained 79 input/build/selector identities remain unchanged, and full
read-only media hashes match the preceding receipt before and after this slice.
All 14 unaffected unfinished source files remain byte-identical; the other two
retain their original paused edits alongside the ordinary runner changes.
Both native processes ended and both profile leases plus the build lease were
successfully reacquired for verification and immediately released.

Observation receipt SHA-256: dashboard
`3be71d34d949079e14fc6fcf05cd1d8f0d42d347ed1d6a80261611e1ce7405e5`, Fable
`ef0276856a159fc801ca59bdc4fee8fe376a023a56092f12a528c8d5905ea4ca`.
Both termination receipts hash to
`a17595a965ea00f659d9af0e147e277f189d0339612eb6fe234d55c73d9c9399`.
Private evidence is under `.local/evidence/dashboard-v1`; original source,
builds, generated empty seed, dashboard seed, profiles, selectors and media are
retained. PSX and all phones remain untouched. No binary release or shipping
default is selected. Desktop Fable progression and fresh custody still gate any
phone trial.
