# Source provenance

## Seed inventory

This foundation contains original project documentation and Bash repository
bootstrap scripts. **It contains no emulator, firmware, game, font, logo or PSX
implementation code.** Its source links and candidate observations do not
incorporate those upstream projects or certify their provenance.

Original contributions use the root MIT license. That grant does not cover
software added from elsewhere. A future combined distribution must satisfy the
exact licenses of its core, dependencies, assets, firmware and copied source.

## Candidate lineage, not incorporated code

- xemu derives from QEMU. Its LICENSE describes GPL version 2 overall, with
  other terms in individual components. Do not assume GPL-2.0-or-later for every
  file merely because a standard GPL text contains example wording.
- X1 BOX (`izzy2lost/xemu`) is an Android fork of xemu; its README retains the
  xemu/QEMU licensing lineage. Trace Android-specific changes and dependencies
  separately rather than treating an APK as corresponding source.
- Roswell (`XboxDev/roswell`) is a ReactOS-derived open Xbox kernel. Its README
  describes a GPL-2.0 combined work with per-file terms.
- OXOS (`kooscode/oxos`) advertises GPL-2.0 and an original boot/kernel path. Its
  source/provenance and comparison-image paths require review before use.
- PSX is a related product, not an xbx dependency. Any selected public MIT host
  source may be reused with its original copyright notice. Do not copy its
  GPL cores, fonts or firmware and label the resulting tree MIT-only.

Observed revisions and source URLs are in `upstream-candidates.json` and
[docs/SOURCES.md](docs/SOURCES.md). No candidate is selected by this inventory.

## Before incorporation

Record upstream URL, exact commit, recursive dependency pins, original notices,
file-level license exceptions, local modifications, generated binaries and
reproduction commands. Maintain a patch series or deliberate fork with a known
reference build. Recheck no proprietary inputs entered generated artifacts.

Changes inside a GPL core remain governed by the applicable license. A Rust
translation is not a licensing reset, nor does a C ABI automatically remove
combined-work obligations. Keep first-party and borrowed attribution honest.
No additional marketplace permission or worldwide legal clearance is claimed.
