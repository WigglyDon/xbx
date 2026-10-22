# Original-Xbox feasibility brief

Research observations: 2026-10-21. See exact observed commits in
`upstream-candidates.json` and [SOURCES.md](SOURCES.md). These are candidate pins,
not accepted implementation dependencies or complete recursive build locks.

## First engine lineage to evaluate

**xemu is the reference lineage; X1 BOX is an existing Android-port candidate.**
xemu is an original-Xbox full-system emulator derived from QEMU. X1 BOX's author
reports Android ARM64, Vulkan, touch/physical controllers and a recommended 8 GB
or more of RAM. That gives us a concrete port to inspect rather than assuming we
must create Android execution from scratch. It does not qualify performance on
this Fold, the source completeness of a distributed binary or fitness of its UI
for xbx. [S1, S2]

Inspect the exact Android-native changes and build/dependency graph against
upstream. Keep or adapt its successful execution/renderer work; replace application
policy and UI with our Rust player. A Java/Kotlin frontend in a donor does not
force xbx to adopt it. Do not delete donor code before understanding native-window,
thread, filesystem and audio contracts. No prebuilt-only blob becomes the core.

Current upstream and fork can differ materially. Compare the observed fork with
a current upstream source view, select a reproducible candidate and freeze it
for the qualification pass. Recheck repository state before work; do not churn
pins automatically on every nightly or readme change.

**Cxbx-Reloaded is a reference/alternative, not the assumed mobile route.** Its
published host requirements are Windows/x86-64 Linux with Wine and Direct3D.
That differs from Android ARM64 and demands its own port analysis. No mandatory
tour of every Xbox emulator is required. [S7]

## Specific technical questions to settle

- CPU: verify the exact x86-guest-to-ARM64 execution path and executable memory
  protections. Host ARM64 support elsewhere is not an Android benchmark.
- GPU: inspect the native Vulkan path, synchronization, shader compilation,
  supported extensions, surface recreation and driver-specific behavior. Do not
  assume the Fold uses an Adreno GPU or that another handheld's fix applies.
- ABI/lifetime: identify actual native entrypoints; libretro is not promised.
  Prove clean shutdown and re-entry, thread/callback lifetime, pause and errors.
- Disk/checkpoints: test a coherent machine/disk save boundary before importing
  the PSX autosave UX. A running frame and a state blob are not durable progress.
- Content: inspect XDVDFS/XISO headers and container layout; preserve originals
  during any necessary conversion. No in-place truncation of owner disc images.
- Resources: estimate checkout, dependency, build and retained-evidence peaks
  before downloads. Share authorized read-only tools, not mutable caches blindly.
- Portability: distinguish package alignment from actual 4 KB/16 KB mappings,
  JIT protection and runtime tests. iOS non-JIT feasibility remains a separate
  unresolved question, not a blocker for making valid Android progress.

## Complete-install direction

The conventional xemu setup expects boot/flash and HDD inputs [S4]. However,
**do not make proprietary firmware import an architectural law**. There are now
source-visible replacement candidates to assess rather than dismissing the goal.

Roswell (`XboxDev/roswell`) describes a ReactOS-based open Xbox kernel, a generated
flash image and a no-bootrom xemu route. Its README calls it work in progress and
uses a 128 MB emulated-machine configuration. Evaluate this exact configuration
and compatibility rather than silently changing a retail profile. [S8]

OXOS (`kooscode/oxos`) claims an open BIOS/kernel route without an MCPX dump and
reports selected game runs. These are author claims, not xbx results or legal
certification. Its documentation distinguishes an open image from an optional
comparison build incorporating a user's Microsoft BIOS. Only the independently
redistributable route could be considered for shipping. Audit provenance, fetched
inputs and output contents; do not import its specification corpus wholesale
merely because the repository uses an open license. [S9]

Cromwell explicitly says it boots Linux rather than original Xbox games; it is
not by itself our retail-game BIOS replacement. [S10] A generated empty disk and
open dashboard may solve packaging of storage without solving boot/kernel
compatibility. Treat those as distinct components. [S4]

Existing replacements are options, not an obligation to accumulate dependencies.
A targeted first-party Rust boot/kernel/HLE implementation or core correction is
allowed when its bounded scope and tests are justified. Choose the smallest
coherent solution. Do not promise a complete kernel in a small integration pass.

## First decisive experiment

Source-build a pinned candidate off-device under the host's actual capacity
policy. Prepare one owner-selected, legitimately available original-Xbox game
and a licensed homebrew instrument where useful. No game has been selected for
xbx by this seed. Do not infer selection from a PS2 game with an Xbox edition.

In an authorized Fold window, establish real gameplay, functioning controls,
sound, a durable in-game save and return after process restart. Capture usable
emulation-speed, frame-time, memory and thermal observations. A title screen,
menu, installation or synthetic fixture is a separate result. Establish a same-
source donor baseline only where it helps attribute an observed problem.

Reuse the same candidate while implementing the smallest Rust vertical slice.
If a core boundary is inadequate, patch it; do not build a framework around the
inadequacy. If the result reveals a material new architecture campaign, retain
useful work and ask Master for that decision with the exact blocker and evidence.
