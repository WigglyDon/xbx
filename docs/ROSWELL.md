# Roswell release and desktop qualification

This is the accepted 128 MiB slice at `8712333`. Master's
[64 MiB amendment](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6386375869)
later superseded its conditional 64 MiB gate and expressly authorized ordinary
controller input with unchanged read-only game media. See
[the separate 64 MiB result](ROSWELL64.md); the historical observations and
earlier approval denial below remain unchanged.

The later [owner-directed retention pass](RETENTION.md) condensed the clean
Roswell checkout while preserving its complete pinned source in offline Git
objects and retaining the frozen flash/build. Restore the checkout under fresh
capacity admission before another source build; no firmware was rebuilt.

Master [6380776059](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6380776059)
authorizes one separate Roswell candidate. Existing checkout was
`a323404c768723bef0dca55a39ad903ae3a803d2`; the completed OXOS/reference and
dashboard milestones were reconciled, not repeated. Actual host process/cwd and
lease checks found one xbx worker and no owned native/build job. All 16 paused
unfinished source paths were copied privately, with 91 prior artifact identities
verified. Earlier allocator and private title-code work remains paused.

The release build and 128 MiB dashboard navigation passed. The single unchanged
Fable 128 MiB run completed normally at 300.242 seconds with only black frames
observed. No visible Fable menu, gameplay or input-response acceptance was
earned. The conditional 64 MiB comparison was not run.

## Source, tools and build

[Roswell 1569e2e89fb47cc72b9c704a8884f98200432bd4](https://github.com/XboxDev/roswell/tree/1569e2e89fb47cc72b9c704a8884f98200432bd4)
was fetched with depth 1 into its own directory. The complete upstream tree
inventory contains 28,577 blobs/441,666,580 uncompressed bytes and no submodules.
The retained source stayed tracked-clean; no Roswell patch was needed.

The normal upstream CMake/Ninja `flash` target built with explicit Release,
DBG=0, ARCH=i386, SARCH=xbox, one compiler job and one-job nested host-tool
builds. HOST_BUILD_TYPE=Release keeps the needed host tools separate from the
target kernel. Only the required host tools, kernel, folded drivers/HAL and open
nxldr packaging were built; no ReactOS-wide build or container was used.

The normal host PATH lacked MinGW/Bison/Flex/M4. Nine exact Fedora RPMs and their
dependency closure were downloaded into the new task directory; every RPM
passed digest/signature verification before extraction. Nothing was installed
into the host. A mirror had recoverable TLS failures during download; the
successful archives and signatures are retained. Exact NEVRA, package SHA-256,
tool versions and dependency-manifest identity are in
[roswell.lock.json](../roswell.lock.json). The portable checksum list
[roswell-dependencies.sha256](../roswell-dependencies.sha256) restores the exact
`dependencies.identity` input; all entries name public tool/package files.

Main tools: MinGW GCC/G++ 16.2.1, target binutils 2.46.0, host GCC 16.2.1,
CMake 4.3.0, Ninja 1.13.2, Bison 3.8.2, Flex 2.6.4, M4 1.4.21; existing host Python/xz
and upstream packaging scripts are retained. Bison data/M4 paths select only
task-local tools. No large toolchain rebuild, floating SDK or privileged change.

Build completed all 729 outer Ninja actions and required host-tool dependencies.
Actual monitored build duration 228.772s, native build exit 0. Output:

- `flash.bin`: 262,144 bytes, SHA-256
  `f18517d1d06c50bf29bd1e47796c7092e0b8e170a99b0985e1af92a0a2ad422e`.
- Kernel 420,352 bytes compressed to 193,380 bytes; upstream packer reports 54,492
  bytes free in its 247,872-byte payload slot.
- Build identity SHA-256
  `49a75875ca01e685a80ccbdf925df1e865ffc1ed6e3ae4d96aaf9366b7283e43`.
- Executed Rust builder SHA-256
  `31aa1d684b407678ec5b638f49a191cbac0b3b1df8e1c0329e1d54c53ec119dc`.

Upstream compiler/linker warnings include ignored alignment attributes,
LTO/stub declaration mismatches and PE section-flag notices. The original log
is retained; no kernel repair is inferred from a warning. Upstream's final
initialization-reference check passed. The flash was frozen read-only after
build. This is a private source-built qualification artifact, not a public
binary release or shipping-default selection.

## Rust policy and resource admission

Original MIT Rust in `xtask/src/roswell.rs` owns source/dependency binding,
Release/DBG selection, serial build invocation, writer ownership and capacity
monitoring. Existing upstream CMake/Ninja/Python/xz remain build mechanisms.
Configure/build refuse existing or changed attempt identities; incomplete
outputs/logs remain. The executed builder/source snapshot is retained separately
from the later observation-wrapper addition.

Master's hard ceiling is 8 GiB additional physical allocation and 10 GiB free.
The existing local policy preserves 12 GiB, so the stricter floor applies.
Preflight observed 22,366,658,560bytes available, Btrfs metadata 67.54% used,
1 MiB device-unallocated, emergency reserve unused, about 5.8 GiB available RAM.
The conservative source/dependency/build/profile/evidence estimate was 4 GiB.

The Rust guard measures allocated blocks in both new task roots and checks
free space plus Btrfs metadata/reserve about every two seconds. It stops at
7 GiB allocation or below 13 GiB free, leaving 1 GiB headroom inside the hard/local
limits. Build descendants inherit the same writer lease; a guard failure retires
only the owned process group. The same monitor wraps each ordinary observation.
No peer job, old build, cache, backup, media or recovery evidence was cleaned.

The ordinary desktop supervisor now accepts only
`XBX_DESKTOP_MEMORY_MIB=64|128`; absence retains 64. Unsupported values refuse
before creating a profile or entering native code. The selected memory is
written to xemu.toml and the observation receipt. Fresh-profile inputs also
bind exact flash/core/host hashes, disc presence and networking. Existing
profile ownership, durable dirty epoch, native reap and storage sync remain.

Execution uses a separate reviewed Rust source copy from a323404 plus only this
slice's changes. Paused unfinished code is excluded. An initial patch-context
collision during source-copy preparation was corrected without changing paused
files; its preliminary binaries/log remain. The new memory test initially used
a socket path longer than Unix permits in that deep copy. Its fixture now uses
a short synthetic /tmp path; the production path guard stays intact.

Changed Rust checks: 35 passed, 3 gated tests ignored; Clippy with warnings denied,
format and diff checks pass. Tests exercise invalid-memory refusal before native
entry/profile creation, retained 64 default, explicit 64/128 profile binding,
private seed preservation/lease release, and allocation/free-space guard limits.

## Ordinary 128 MiB dashboard

Retained source-built upstream xemu 03b988c9d76553b33ed712ba00822ffdf49f0e7d,
core SHA-256 31e1c0f5f3c165d25e9c6315ecaf3191b972f57ae22616d03de6ee8d6243be3d
and its already documented mechanical integration patch are unchanged; see
[REFERENCE_CORE.md](REFERENCE_CORE.md). The ordinary host used here hashes to
`842d177c722a847db9821447363dd4fa6709b098e3a99b8a5c091cdffaa36a10`.
The observation capacity wrapper hashes to
`63cab1bce8319b60130d538fdd8b6c9911e7173d1140f1ffb900e46167a28bb1`.

New `.local/roswell-v1/profile-dash128-v1` independently copies the pristine
dashboard seed 00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd,
the generated EEPROM template and the new flash. No OXOS owner profile is
reopened under Roswell. EEPROM/flash/disk paths and XDG data/config/cache stay
inside the new profile; no empty EEPROM path or shared host-global fallback.
No disc, no MCPX, guest networking disabled, update checks disabled,
Controller S/keyboard on port1, TCG/native FP, X11/OpenGL/llvmpipe/dummy audio.

Reviewed owned-window frames show the actual dashboard Main Menu, No Media and
No Link. The clock advances 07:29:17→07:29:26→07:29:36 and background geometry
changes. Normal A opens System Information (kernel version 1.00.5530.00), and B
returns to Main Menu. Normal bounded completion at 300.385 seconds acknowledged
QMP stop/quit, native exit 0, forced=false and storage_synced=true. The wrapper
reacquired the profile lease before publishing its bound qualification inputs. Controller attachment is separately confirmed by the
existing USB inventory. No updater, installer, settings or file-transfer action
was invoked. Only the exact owned local emulator window was captured/controlled.
Generated identity in System Info stays private. Displayed disk-use/clock values
are not independent filesystem/time accuracy acceptance.

## Fable 128 MiB outcome and preservation

The 128 MiB dashboard completed at 300.385 seconds with acknowledged QMP stop/quit,
native exit 0, forced=false and storage_synced=true. The wrapper reacquired its
profile lease and durably wrote the firmware/memory input receipt before the
separate game process started.

Fable used new `.local/roswell-v1/profile-fable128-v1`, the same exact release
flash/reference core/pristine seed, another independent EEPROM/disk copy, 128 MiB,
and the retained unchanged read-only disc view. Disc was configured at cold boot;
no dashboard-to-disc action was used. The final normal launch-parameter log
confirms `-m 128`. Fable uses no `-nic` override or explicit network settings;
the recorded core defaults apply (its configured default backend is NAT).
No port forwarding or network feature was selected or qualified.

Reviewed frames at about 9.5/56.6/141.1/251.6 seconds are black. There is no
evidence establishing a game menu, gameplay or executable dispatch; none is
claimed. The release serial file stayed 0 bytes. Its final native log is 1533 bytes,
including normal shutdown/launch-parameter output. Quiet logs cannot establish
absence of a guest fault. The observation budget expired at 300.242 seconds;
there was no confirmed terminal guest error. QMP stop/quit acknowledged,
native exit 0, forced=false, storage_synced=true. A later owned-window capture
safely refused after native exit.

One attempted Start/Return input was rejected by automatic approval review as
changing the unchanged observation. It did not execute; no workaround or
alternate input route was used. Passive observation continued. The rejection
does not diagnose Roswell or Fable, and game input response remains unqualified.
The successful dashboard A/B navigation is a separate observed result.

This proves alternative-environment startup/render/input at 128 MiB, while the
chosen game remains unavailable in this exact setup and 300-second budget.
It does not prove universal incompatibility or attribute the lack of game
progress to a specific kernel/core/media component. No ordinary integration
defect was demonstrated that would justify repeating this game gate here.

The 64 MiB follow-through condition was not met, so no 64 MiB run or additional
game trial occurred. A next bounded choice for Master would be an explicit
same-release 64 MiB control despite this negative 128 MiB game result, since the
pinned run helper documents both configurations. It would isolate that memory
variable without a speculative kernel patch. That is a proposed new scope,
not work started by this handback. Prior investigations remain paused.

Both native processes ended; both profile leases and the global build lease
were reacquired and released after termination. All 91 preceding artifact
identities and the complete read-only media hash match after the slice. All 16
paused source paths remain preserved: 14 untouched files are byte-identical;
the two touched files retain their original paused edits outside the commit.
The Roswell source remains at the pin and tracked-clean; the frozen release,
reference core, seed and all previous generations are retained.

Final task-root physical allocation 1,503,981,568 bytes; sampled peak 1,503,936,512
bytes. Minimum sampled free space 19,874,324,480 bytes; minimum sampled usable
Btrfs metadata outside reserve 1071 MiB. No resource guard refused, and global
reserve remained unused. Tiny synthetic /tmp fixtures and reviewed source edits
are retained as well; all growth remains far below the 8 GiB ceiling. No cleanup
was performed to achieve these values.

Observation receipt SHA-256: dashboard
`c9491b8c59d06fdff353e2d7dc5966b00c096827f0aa20d1aedc963f0390cbbe`, Fable
`d2d96328273a6733b91f7a82e387b06c20b4dbf4bcf45cf4fd440ef1d079e6e5`.
Both native termination receipts hash to
`a17595a965ea00f659d9af0e147e277f189d0339612eb6fe234d55c73d9c9399`.
Detailed commands, source/tool/patch identities, failures, captures and manifests
are retained privately under `.local/evidence/roswell-v1` and the separate
`.local/roswell-v1` build/profile tree.

## Reproduction

Initial build commands are below. Completed attempts are protected; another
source build needs a new simple attempt label and matching clean pinned source
directory. Restore the exact task-local packages and copy
`roswell-dependencies.sha256` to `.local/roswell-v1/dependencies.identity` first:

```sh
cargo xtask roswell-configure v1 /absolute/xbx
cargo xtask roswell-build v1 /absolute/xbx
```

The successful build is retained. Ordinary observations use new labels:

```sh
cargo xtask roswell-observe /absolute/xbx v1 dash128-v1 128
# Only after actual dashboard/input acceptance and clean native termination:
cargo xtask roswell-observe /absolute/xbx v1 fable128-v1 128 "$AUTHORIZED_MEDIA"
# Original slice's condition; superseded by Master6386375869 (see ROSWELL64.md):
cargo xtask roswell-observe /absolute/xbx v1 fable64-v1 64 "$AUTHORIZED_MEDIA"
```

Each command is bounded to 300 seconds with ordinary early terminal-failure
handling. Dashboard is offline; title networking uses the recorded core default.
No snapshot, extra tracing, debugger or private game-code inspection is enabled.
The 128 MiB alternative changes firmware and represented-console configuration;
any later 64 MiB comparison tests this release's configuration requirement only.
Neither outcome identifies the cause of OXOS's earlier result.

All previous evidence, media, profiles, builds and source lineage remain
preserved. No phone, PSX change, data migration or public binary release.
Gameplay, audible audio, in-game saves, complete checkpoint resume, Android
compatibility/performance and owner acceptance require their own observations.
