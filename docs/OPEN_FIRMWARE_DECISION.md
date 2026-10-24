# Official OXOS reference and firmware decision packet

Master [6380501920](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6380501920)
accepts the dashboard milestone at
`198dd3449d5ec4e28482fa8f81070ddfccd6e7d3` and assigns one ordinary Fable
reference. [DASHBOARD.md](DASHBOARD.md) remains the completed dashboard receipt;
none of its runs or the earlier diagnostic campaigns was repeated.

**Result:** one official-release/upstream-reference run completed at 300.378s
without an observed Fable menu or gameplay. Reviewed frames show the OXOS splash.
Normal QMP shutdown, native exit 0, no forced termination and storage sync were
verified. The conditional retained-xbx-core comparison was not run.

## Exact reference inputs

- Official, unmodified [OXOS v0.3.3 release](https://github.com/kooscode/oxos/releases/tag/v0.3.3),
  `bios_v0.3.3.bin`, 1,048,576 bytes, SHA-256
  `8bf516fcb453f18925ad0aad75ff923bd5f225420be6b0175d0fe818f0653314`.
  Annotated tag object `2beec03409c36d04f037e6f10e1a5447e01ff8e7` resolves to
  source commit `9d8cd58ca19f49f5ecb4b7b55bfa194f4e4a5a96`.
- Retained source-built upstream xemu
  `03b988c9d76553b33ed712ba00822ffdf49f0e7d`, executable SHA-256
  `31e1c0f5f3c165d25e9c6315ecaf3191b972f57ae22616d03de6ee8d6243be3d`.
  This is the [previously qualified reference build](REFERENCE_CORE.md), with
  full integration patch
  `4e317283ddb368c1c80243804aa7fcfdc1e5228ac6d7ddec9ca84cb5cdd138e1`.
  Its watch ownership, process cleanup, Epoxy dependency and source-built DSP
  integration are recorded there. It is not a pristine upstream binary, and is
  not xbx's retained Android fork. No new patch or rebuild was applied.
- Accepted pristine xemu-dashboard seed SHA-256
  `00d7df7a2bc235f8801764f00b7f40e194d1e392f7a9619d6b2396c89770f6dd`,
  independently copied into a new profile. Complete component/font notices stay
  in LICENSES; the pristine seed and official BIOS remain mode 0400.
- Existing ordinary Rust supervisor SHA-256
  `2ec736f61c0eb56fc743e15a64510d48f45d4efa976eff1073b3dcf0f459c97e`,
  tested source at 198dd34, excluding all unfinished allocator edits.
- Same unchanged read-only Fable ISO, structurally validated partition offset
  405,798,912, shared through the retained read-only FD view. Full media identity
  matched the preceding receipt. No extraction, repacking or game modification.
- 64 MiB, TCG/native floating point, X11/OpenGL/llvmpipe, dummy audio,
  generated EEPROM template independently copied, Controller S on port1.
  MCPX path empty, automatic update checks disabled, core network default.

The official BIOS is an experimental input. Upstream identifies GPL-2.0 and
Cromwell 86f as its source lineage; the downloaded license is retained privately
alongside the asset. This exact-artifact experiment neither reproduces that
release build nor selects a shipping firmware. Upstream provenance statements
are not independent distribution clearance.

## Reproduction and preservation boundary

```sh
XBX_DESKTOP_VIDEO_DRIVER=x11 G_DEBUG=fatal-criticals \
  "$ACCEPTED_HOST" run-title "$RETAINED_UPSTREAM_CORE" "$NEW_PROFILE" \
  "$OFFICIAL_OXOS_V033" "$PRISTINE_DASHBOARD_DISK" "$EEPROM_TEMPLATE" \
  "$AUTHORIZED_UNCHANGED_FABLE" 300
```

The existing Rust runner creates a fresh mode 0700 root, holds its profile writer
lease, advances the dirty epoch before native writes, and copies writable
storage independently. It retains the read-only disc capability and uses ordinary
QMP stop/quit, actual reap and storage sync. The private operator captures only
the exact owned X11 window.
No diagnostic map, guest-memory read, observer flag, game-instruction analysis
or debugger is requested. No machine checkpoint is loaded.

Preflight found one xbx Codex, no owned native/build job or held xbx lease, about
19 GiB free, Btrfs metadata 67.43% used and global reserve unused. All 16 unfinished
paths were copied privately and 86 retained artifact identities matched before
entry. Extra writes are the 1 MiB public BIOS, small source references, a fresh
sparse disk/profile and bounded evidence. No SDK/core build or cleanup is needed.

Private inputs, exact commands, source-reference copies, owned-window captures,
native logs and preservation manifests remain under
`.local/evidence/official-oxos-reference-v1` and
`.local/fable-official-reference-v1`. The previous dashboard profiles, generated
empty seed, selectors, game and source/build generations remain retained.

## Actual ordinary observation

The disc was configured and present at cold boot in the new profile. Images
taken at approximately 3, 24, 47, 130, 240 and 284 seconds show the OXOS splash after
its initial fade. None establishes a visible Fable menu or gameplay. Controller S
attachment was confirmed by the existing normal USB inventory. No dashboard-to-
disc action was taken. There is no evidence establishing executable dispatch on
this release, so none is claimed.

The unmodified release's serial file stayed 0 bytes. The 589-byte native log
identifies the expected xemu commit and OpenGL/llvmpipe host. Quiet output does
not establish guest progress, absence of a guest fault, or a particular cause.
The observation expired normally at 300.378seconds; it did not report a terminal
guest error. QMP stop/quit acknowledged, native exit 0, forced=false,
capture_failure=None, storage_synced=true. The native process ended and the
profile/build leases were successfully reacquired and immediately released.

This is an upstream-reference compatibility limit for this exact BIOS/core,
media view, generated profile, 64 MiB and 300-second host observation. It is not a
universal OXOS/Fable incompatibility claim, nor proof that a particular component
caused the lack of progression. Both firmware and core differ from the preceding
xbx comparison; this baseline was intended to qualify the documented open stack,
not attribute a defect to one changed component. No alternative core comparison,
Fable repeat, private title analysis or diagnostic campaign followed.

All 16 paused unfinished source files remain byte-identical. The 86 preceding
artifact identities and full read-only media identity are checked again after
the run. All source/build/profile generations and the official asset/license
are retained. No runtime implementation changed, no build or new test suite was
needed, and the accepted 32 Rust test passes remain the prior verified result.
Only receipts/state/provenance documentation changes in this assignment.

The new profile allocated 28,360,704 bytes after the run, including its retained
images/cache; no full game copy was created. Observation receipt SHA-256:
`a7f7e1487d8e1512922ef156271bc30b64dfb2f78fcc3a123afa244b7e9dc446`.
Termination receipt SHA-256:
`a17595a965ea00f659d9af0e147e277f189d0339612eb6fe234d55c73d9c9399`.
No phone, PSX, public binary release or active-source reset occurred.

## Paths toward the setup-free product

These are engineering sizing judgments and proposed future gates, not measured
delivery estimates or newly activated work. No public option has earned Fable
acceptance on xbx. Dashboard operation demonstrates only the exercised dashboard.

| Path | Evidence and product fit | Cost and uncertainty | Evidence that would distinguish it |
| --- | --- | --- | --- |
| Maintain OXOS with a defined compatibility scope | Existing xbx builds and synthetic regressions reduce setup cost. Its public gameplay list covers other titles; Fable is absent. The v0.3.3 notes address quick reboot and exFAT, which do not establish a Fable fix. | Lowest tooling cost, unknown compatibility effort. Existing incomplete API behavior cannot be treated as a demonstrated cause here. No observer variant or speculative fix follows from this run. | A newly scoped, original public-contract reproducer must demonstrate a concrete defect and its repair before another game validation. Master would need to authorize that separate campaign. |
| Qualify Roswell as an independent open kernel | ReactOS-derived implementation, own loader, no boot ROM required; documented configuration is 128 MiB. Public status is work in progress and unspecified software boots. No Fable result is claimed. | Separate source/dependency/license audit and build, unknown build peak and gameplay coverage. 128 MiB changes machine configuration as well as firmware. Save and EEPROM compatibility need new isolated profiles. | First a source-faithful pinned build, original/homebrew smoke and visible dashboard at the documented 128 MiB; only if those pass, one separately scoped unchanged-Fable observation. This tests alternative viability, not kernel-only causality. |
| Implement bounded first-party Rust platform services | Fits Rust ownership while retaining the existing emulation core. Public interfaces and source-authored fixtures can define individual services. | Highest scope risk. No small missing service has been established by this experiment; implementing a new whole Xbox kernel would exceed this assignment. | A Master-approved service boundary, public behavioral specification, original failing conformance fixture, and an explicit integration/replacement plan must precede implementation. |

OXOS facts above are upstream claims from the
[pinned README](https://github.com/kooscode/oxos/blob/9d8cd58ca19f49f5ecb4b7b55bfa194f4e4a5a96/README.md)
and [release notes](https://github.com/kooscode/oxos/releases/tag/v0.3.3).
Its documented different save-signature keys make firmware/profile identity a
durable compatibility boundary; the owner's existing data must not be moved
between kernels implicitly.

Roswell's [pinned README](https://github.com/XboxDev/roswell/blob/1569e2e89fb47cc72b9c704a8884f98200432bd4/README.md)
documents 128 MiB and GPL 2 as a combined work with per-file terms. Its
[contribution policy](https://github.com/XboxDev/roswell/blob/1569e2e89fb47cc72b9c704a8884f98200432bd4/CONTRIBUTING.md)
distinguishes public interface facts from implementation provenance. These
sources support considering it; they do not establish game compatibility or
Android readiness.

[Cromwell alone](https://github.com/XboxDev/cromwell/blob/86f547387184d1001d377cce97d3756c8acf91cc/README.md)
boots Linux and explicitly cannot launch original Xbox games. It supplies useful
boot components rather than another retail-game kernel option. The qualified
open dashboard supplies an application and disk environment, not a replacement
kernel. Proprietary BIOS acquisition does not meet this product's complete,
distributable installation requirement and remains outside the assignment.

## Proposed next decision

The official reference did not earn Fable acceptance. Recommend one bounded
Roswell source/build/desktop qualification as the next materially
different control. Keep the existing xbx core/player and OXOS candidate preserved;
do not select a shipping default from a source description. Use the already
inventoried Roswell pin initially, or explicitly justify a new pin before build.

The proposed scope is admission/dependency audit, one isolated source build,
original smoke, visible dashboard, then one conditional Fable observation.
Stop at a failed gate and return the evidence; no broad kernel rewrite,
private title analysis, phone trial or automatic restoration of paused branches.
Resource peak and an explicit allocation cap must be established before that
future build; the current19 GiB free measurement grants no large build by itself.

If Master instead prioritizes OXOS compatibility engineering, choose a concrete
public service with a new finite specification and acceptance boundary. Repeating
Fable while changing diagnostic builds would not distinguish that choice.
Neither path is started by this decision packet.
