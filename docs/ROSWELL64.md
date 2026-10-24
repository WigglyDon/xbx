# Roswell 64 MiB qualification

Master's [acceptance and assignment](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6386375869)
accepts `87123331470789f6dc24746c51b8d78caff07b1b` and supersedes the former
128 MiB game-progression prerequisite for a 64 MiB check. The accepted release,
128 MiB dashboard/navigation and completed 128 MiB Fable observation were not
rebuilt or repeated. See [ROSWELL.md](ROSWELL.md) for those actual results.

**The 64 MiB dashboard gate remains unqualified.** Two fresh dashboard attempts
were interrupted by the existing host Btrfs metadata guard. The second showed
only black frames before and after an approved ordinary A input. Fable at
64 MiB was not launched because its dashboard baseline did not pass. Neither
interruption proves a Roswell guest failure or a completed timeout result.

## Reconciliation and unchanged inputs

One implementation session in the existing checkout; no second worker or owned
native/build job at admission. All 16 paused paths were copied privately before
work. The 101 preceding artifact identities matched, including the accepted
release/core/supervisors and 128 MiB receipts. No paused allocator or private
title investigation resumed. PSX and phones were untouched.

Both new profiles independently copied the same pristine disk and generated
EEPROM template; no previous writable profile was reopened. Inputs:

- Roswell source `1569e2e89fb47cc72b9c704a8884f98200432bd4`, retained tracked-clean;
  Release/DBG=0 flash SHA-256
  `f18517d1d06c50bf29bd1e47796c7092e0b8e170a99b0985e1af92a0a2ad422e`.
- Retained upstream xemu `03b988c9d76553b33ed712ba00822ffdf49f0e7d`, core SHA-256
  `31e1c0f5f3c165d25e9c6315ecaf3191b972f57ae22616d03de6ee8d6243be3d`.
- Pristine dashboard seed SHA-256
  `00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd`.
- Explicit 64 MiB, no disc, no MCPX, `-nic none`, updates disabled,
  Controller S/keyboard port 1, TCG/native FP, X11/OpenGL/llvmpipe, dummy audio.
  No audio qualification. Final native configuration and USB attachment records
  are retained; a connected controller is not evidence of guest input response.

## Ordinary supervisor changes

The ordinary Rust supervisor accepts a new profile-local `observation.complete`
request. It requests the existing QMP stop/quit, reap and storage-sync path;
it does not mark visual acceptance. A confirmed terminal failure takes precedence.
Existing profiles, including stale completion requests, still refuse reuse.
The Roswell wrapper can explicitly select a new supervisor through
`XBX_ROSWELL_OBSERVATION_HOST` and records its hash. This preserves the accepted
executable instead of rebuilding over it. No guest/core or capacity-policy edit.

Only these ordinary changes were built from an isolated source copy of
`8712333`; paused edits were excluded. Host SHA-256
`6275e583494d1c758d1d2d4cec0a3b48c1855a98f8d968acb2ac2ce3d3db432e`;
wrapper SHA-256
`d92e07321fb2fd6cbc0f5fc161fd0b9f516a734487ba5c744b503bfef7301e0c`.
Offline serial Rust build passed, 35 tests passed / 3 gated checks ignored,
Clippy with warnings denied passed. The existing-profile preservation test now
includes a stale completion request. Both native attempts were guard-retired
before normal completion, so the new early-completion path did not receive a
successful native integration observation in this slice.

Commands, once per new label `dash64-v1` and `dash64-v2`:

```sh
XBX_ROSWELL_OBSERVATION_HOST=/home/don/xbx/.local/roswell-v1/rust-target64-v1/debug/xbx-host \
  .local/roswell-v1/rust-target64-v1/debug/xtask \
  roswell-observe /home/don/xbx v1 dash64-v1 64
```

The first fresh attempt was interrupted after about 4.5 seconds. Its only
reviewed frame was black with the host keyboard-attachment overlay. Its dirty
epoch, disk, EEPROM, logs and image were retained. After metadata accounting
recovered above the unchanged admission floor, only this interrupted dashboard
gate was retried in another new profile. No completed game trial was repeated.

## Second attempt: passive and interactive observation

`profile-dash64-v2` was started on 2026-10-24 at about 16:20:39 UTC. Owned-window
captures at approximately 1.940 and 105.527 seconds were black; only the early
host keyboard-attachment overlay was visible. No dashboard menu or activity was
established during this passive interval.

One ordinary A input used the established exact-PID/window controller route
through the normal approval process. It executed under the new express scope
clarification; the older denied Start action remains unexecuted history. The
helper recorded completion at about 137.522 seconds after its ordinary
two-second press/release sequence. Reviewed frames at 139.430 and 185.854 seconds
remained black. This establishes no visible response, menu or navigation;
delivered input alone is not a pass. No B, Start, directional, settings, updater
or installer action followed.

At 199.811 seconds after the recorded command, the wrapper wrote
`growth refused: insufficient usable Btrfs metadata outside emergency reserve`
and retired its owned process group. This timing is from file timestamps,
not a normal supervisor observation receipt. There was no normal termination
receipt; the release serial file remained empty. No confirmed guest terminal
error or executable dispatch was established. The kernel/core cause of the black
frames remains unknown. This resource-interrupted 64 MiB dashboard observation
cannot isolate the cause of the prior 128 MiB Fable timeout or OXOS faults.

## Termination, storage and capacity

Both guard interruptions forcibly retired the owned supervisor/native process
groups. Afterward, no owned native/build job remained; both profile writer
leases and the global lease were reacquired and released. Existing regular
profile files and directories were fsynced under exclusive ownership after
retirement. **This is not clean native shutdown, coherent-checkpoint acceptance
or a normal native storage-sync receipt.** Both dirty profiles remain preserved
and are not reused. No repair, rollback, cleanup or deletion was performed.

Initial free space was 18,131,189,760 bytes, with about 1026 MiB usable metadata
outside the unused reserve. The unchanged local floor is 1 GiB usable metadata
when device-unallocated space is below 1 GiB; device-unallocated was only 1 MiB.
Passing samples had almost no margin, and two later checks fell below that floor.
Data-space and task-allocation ceilings were not the refusals. A final host-side
read showed device-missing 0, 17,675,505,664 bytes available, metadata 70.00% used
and emergency reserve still unused. Task/evidence physical allocation at that
read totaled 1,589,923,840 bytes, well below the assignment's 8 GiB ceiling.
Capacity changes on this shared host do not establish xbx as their sole cause.
No third attempt follows automatically from transient recovery above the floor.

All 101 preceding artifact identities and the unchanged read-only media are
verified again at handback. Fifteen paused paths are byte-identical; the touched
desktop supervisor retains its earlier paused hook outside this commit. Its
complete before-image and patch remain private. All source/build generations,
128 MiB profiles, failed 64 MiB profiles and evidence are retained under
`.local/roswell-v1` and `.local/evidence/roswell-v1/retail64-v1`.

The remaining immediate prerequisite is stable host metadata headroom sufficient
to complete the unchanged gate under existing margins. There is no qualified
64 MiB dashboard or Fable result to take to a phone. The accepted 128 MiB
dashboard remains the demonstrated Roswell baseline; Fable compatibility remains
an xbx deliverable with no common OXOS/Roswell cause established. Master's
[ownership direction](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6386463128)
is reconciled in PRODUCT.md and ARCHITECTURE.md. A substantive new service or
diagnostic campaign still requires Master's evidence-selected scope.
