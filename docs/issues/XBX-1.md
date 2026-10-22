# XBX-1: Source qualification and original Xbox vertical slice

## Assignment

Authorized initial work for one local xbx Codex. The owner requests a separate,
public, Rust-first Xbox sister to PSX with the same game-first experience.
Original Xbox is the initial target; the stock Android ARM64 Fold is the first
desired playable device. No xbx implementation or gameplay result exists at seed.
Read AGENTS.md, the Master seed, current state, architecture, feasibility,
distribution and operations documents before source work.

A fresh product Master evaluates the initial choices and material boundaries.
Routine inspection/build/implementation should proceed within this assignment,
not wait for another permission message or generate only a planning report.
No second worker or change to an active PSX checkout is authorized.

## First concrete work

1. Bind the actual local/remote xbx revision and inspect uncommitted work. Read
   live issue comments before assuming no other worker. Verify owner/visibility
   and source/bootstrap receipt. Record an acknowledgment with the first action.
2. Inspect host tools and current storage/allocation/reserve without mutating
   shared caches or starting a large build. Estimate a bounded source-build peak.
3. Evaluate the xemu lineage and X1 BOX's existing Android-native port from the
   observed pins or a justified newer pin. Lock recursive inputs, record original
   licenses and trace native renderer, CPU, pause, threading, file and state APIs.
4. Identify the smallest Rust-owned Android slice that can run this core. Do not
   require libretro, a desktop frontend, a particular host language or an immutable
   upstream boundary. Targeted patches and narrow C-ABI glue are allowed.
5. Source-build the candidate when capacity permits; retain a reference and exact
   patch/build identity. No prebuilt APK/library substitutes for reproducibility.
6. Implement the minimum executable Rust host to import one supported local game,
   run actual graphics/audio/input and stop safely. Prove actual execution rather
   than scaffold a generic backend trait with no working engine.

Inspect the disk/snapshot boundary early. Native runtime code must not write
outside its selected private virtual-console profile. A one-game diagnostic may
use scratch storage before a full library exists. Do not risk owner saves just
to prove an engine boots.

## Firmware and completeness

Use legitimate locally supplied required inputs if the selected route needs
them. Also evaluate the current open-kernel candidates in the feasibility brief
for a distributable default. Audit output provenance and actual gameplay rather
than taking "open" or "clean-room" labels as proof. A targeted original Rust
implementation is permitted when bounded and better justified than extra layers.
Do not turn the initial pass into an unbounded replacement-kernel campaign.
The ordinary product should eventually supply all distributable machinery.

No Xbox game is selected by this seed. Ask the owner to select an original-Xbox
title and identify its authorized local input; do not substitute a PS2 disc or
invent availability. Continue independent source/host work while an input or
phone window is missing. Generated homebrew can prove mechanisms, not commercial
compatibility.

## Device and evidence

Acquire a fresh authorized stock-device window through the existing procedure;
prepare off-device first. Keep PSX and all unrelated apps/data untouched. Use the
separate proposed `com.machinerhythm.xbx` application identity and its own signing.
No root, system-package changes, security weakening or console flashing.

Observe installation, startup, actual game frames, audible sound, usable analog/
digital touch input and intended emulation speed separately. Record memory and
available thermal observations in a bounded run. State what failed or was unrun.
A donor desktop result is not mobile acceptance; a title screen is not gameplay.

## Integration and completion

Record a source-backed architecture decision and the smallest working candidate,
then continue the prepared XBX-2 work when this path is viable and in scope.
Formal closure of this issue is not a ceremonial block to ordinary progress.
Keep both issue results distinct; Master accepts completion based on evidence.

A material change of platform, whole-core/kernel campaign, unresolved rights,
unsafe persistence model, spending or unavailable resource requires Master's
specific decision. Modifying donor code by itself is not a hard stop.

Hand back exact commits/builds, reproduction, source/license inventory, actual
mobile results and resource/device return. No public binary/store release is
implied by a prototype. Keep private inputs and raw owner evidence out of GitHub.
