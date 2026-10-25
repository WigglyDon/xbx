# Product: Xbox games as durable applications

## Intent

`xbx` is a separate application in the same product family as PSX. The owner's
successful PS1 experience is the model: a game can be left, another opened, the
phone folded or the process restarted, and the right game resumes safely. Xbox
needs its own implementation, not a rebrand of PSX or a merger of console cores.

The product is **choose a local game -> play -> leave -> return**. Master's
[cartridge assignment](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6400021880)
defines game images as cartridges conceptually: import once, without mandatory
repackaging or duplication. The integrated runtime supplies its own pinned open
system implementation and generates game-isolated writable storage. Firmware
selection and separate firmware snapshots are not ordinary product concepts.
A Home Screen
shortcut is an entrance to an identified game, not a menu of emulation cores.
The ordinary player should not choose JIT modes, graphics backends, BIOS regions
or obscure compatibility switches. Use safe technical defaults with compact,
actionable explanations only where an input or capability is genuinely missing.

## Scope

First generation: original Xbox. This is the seed's initial engineering
interpretation of "Xbox" and is explicit for Master review. Future Xbox 360 or
later systems need separate engine, resource, rights and performance decisions.
Do not ship dead sections or imply that an original-Xbox core runs those games.

The owner's initial desired device was the stock Android ARM64 Fold. The owner's
October 22 continuation makes the more accessible stock Pixel 4 the higher
priority compatibility target; retain the Fold as a supported target to qualify.
Measure the actual OS, GPU, host page size and available memory in an authorized window; do not
infer them from a product nickname or from a different phone. Host diagnostics
on Linux/macOS may establish causes, not mobile acceptance. Keep an iPhone/store
route as a product goal without pretending a normal stock-iOS build can use a
debugger-enabled or specially entitled JIT.

## Ordinary experience

Open xbx to a small local collection and a clear Add games action. Detect supported
content structurally, preserve stable entries through relinking, and remember
selections without rescanning storage every launch. Scoped file/folder picking
must be a complete route. An `.iso` suffix does not identify a valid Xbox disc.
No game download service, account, mandatory network, cover scraping or telemetry.

Each entry supports direct play/resume and a small actions sheet: add shortcut,
locate moved content, explicit restart, and later safe progress export. Invalid
launch targets lead to that entry's recovery, never a different game or arbitrary
path execution. Removing a shortcut does not delete content or saves.

The picture is aspect-correct and clear of controls by default. Touch layout
adapts to actual usable window size, cutouts, inner/outer displays and rotations.
Phone play defaults to sensor landscape, with controls in side columns and a
clear 4:3 picture using the public PSX layout geometry. Face buttons retain
original Xbox A green, B red, X blue and Y yellow; Black and White have distinct
caps and text, alongside Back, Start, both triggers and stick clicks.
Retain multi-touch capture, cancellation, useful analog dead zones and editable
per-display profiles. Original Xbox input includes two sticks, stick clicks,
analog triggers, D-pad, A/B/X/Y, Start/Back and Black/White. Preserve pressure
channels where the selected engine/game uses them; touch may use documented
fixed pressure for a pressed digital face button. Do not silently map this to
a modern Xbox controller that has different shoulder/control semantics.

Physical controllers feed the same logical model; disconnect neutralizes input
and reveals usable touch controls. This is not a reason to require a controller
for the first touch-player milestone.

Pause feedback is immediate. User pause survives focus, display and lifecycle
changes. An interruption preserves explicit user Pause separately; healthy return
automatically resumes unless the user chose Pause. Explicit Stop/Restart uses
current ordinary saves. An incompatible checkpoint requires recovery, never an
unannounced cold boot. Background behavior defaults to paused, silent and checkpointed. Audio
focus belongs to the platform. No silent-audio keepalive service or background
emulation assumption. Audio queues and input must not replay stale activity when
returning. Adopt reported cadence rather than manufacturing a 60-FPS requirement
for a title that internally targets another rate.

## Continuity is the product contract

A game has durable identity, progress and a resumable session independent of its
current process. Distinguish guest in-game save, complete emulator checkpoint,
last-presented frame and evidence of successful storage durability.

Original Xbox typically writes progress into an emulated hard disk. A snapshot
can include that disk's history, unlike our PS1 card boundary. Therefore safe
return is defined by [architecture](ARCHITECTURE.md), not by copying PSX's state
bytes or restoring memory and then replacing arbitrary files.

Normal game switching should preserve the departing game's accepted checkpoint
before destroying its only live state. Storage failure must be visible and
recoverable. Incompatible checkpoints are retained; explicit restart uses current
durable progress. Automatic fallback must not silently erase the session the
user expected to resume.

## Fullness without concealed dependencies

The ordinary installation must provide a distributable, setup-free environment
beyond supplying a game: player, qualified engine/runtime, open boot/kernel
route and generated storage environment. Users must not obtain a proprietary
BIOS/MCPX bundle or assemble firmware/storage as ordinary setup. Completing the
public Xbox platform is part of xbx's engineering responsibility. This is a
binding requirement, not a claim that the current candidate achieves it.
No copied console startup art, dashboards or proprietary images bundled under
an open-source label.

Master's [ownership amendment](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6386463128)
makes demonstrated compatibility of that distributable boot/kernel environment
an xbx deliverable. A missing compatible service is engineering work for xbx,
not another private firmware prerequisite for the owner. Keep traceable inputs,
reproducible specified builds and repeatable game compatibility as distinct
obligations. A successful source build or dashboard does not satisfy the first
game milestone: Fable reaching play with controls, then sound and an ordinary
save/cold-reload check. Later titles need independent qualification.

Open-kernel compatibility is unverified until tested. A default that removes a
setup prompt but breaks saving or the selected game is not an improvement. Read
[DISTRIBUTION.md](DISTRIBUTION.md) before selecting or building firmware.

## Deliberate non-goals for the first milestone

Do not start a new whole Xbox core, broad emulator framework, cloud service,
online multiplayer, achievements network, native-code downloader, full second
platform port or concurrent-running-game feature. Do not rewrite working donor
internals just to make the repository look pure Rust. Small justified changes
and first-party Rust replacements are expressly permitted.

The architecture must allow future bounded Android resident sessions without
requiring them now. One foreground runtime and many saved game sessions deliver
the initial user benefit. A later "Keep another game ready" feature must survive
OS eviction safely and demonstrate a measured benefit.
