# Completeness, rights and distribution

Goal: a public, free-source player with a complete ordinary installation, and
an eventual Google Play and Apple App Store route. None of those stores has
approved xbx. This seed neither publishes a binary nor provides legal clearance.

The ordinary product must be distributable and setup-free beyond the user
providing a game. Supply the qualified engine, open boot/kernel route and
generated storage environment in that installation. Current compatibility and
release qualification remain unearned; this requirement is not a shipping claim.

## What can be included

Package original Rust software, correctly licensed engine/runtime components,
necessary notices, and independently distributable boot/kernel and generated
storage components that have passed qualification. Users should not assemble a
developer toolchain, download native cores separately or select obscure modes.
The boot/kernel is an immutable internal runtime resource selected by xbx, not
a user-managed input or separately saved firmware session. Private reference
inputs are excluded from the ordinary resource manifest and launch path.

An open firmware candidate's license is not an audit of every source and binary.
Reproduce from pinned source, inspect build-time downloads, distinguish open
outputs from comparison builds containing proprietary material, retain all terms,
and test gameplay, saving and checkpoints. No original Microsoft BIOS, MCPX ROM,
retail dashboard, game, owner EEPROM/key dump or game-derived save belongs in
public source or release artifacts without the relevant rights.

Conventional low-level boot has proprietary input requirements; it is not the
ordinary product's setup contract. Master's
[private reference assignment](https://github.com/WigglyDon/xbx/issues/1#issuecomment-6386851912)
supersedes the earlier closure of BIOS/MCPX comparison for the owner-supplied
candidate. Such an input requires resource admission, actual variant/MCPX
identification and observed compatibility before it can be a working reference.
It remains private, unverified until qualified, and separate from shipping assets
and clean implementation inputs. GitHub availability or a computed hash does
not establish redistribution rights or original authenticity. No proprietary
implementation copying, BIOS patching or disassembly is assigned.

Complete and qualify the distributable public boot/kernel environment rather
than making proprietary firmware acquisition an ordinary user prerequisite.
Existing public-source corrections and bounded original Rust replacements remain
available under the architecture/scope rules. Do not remove safety checks or
counterfeit a known firmware identity to make an unqualified replacement load.
See [feasibility](XBOX_FEASIBILITY.md) and the
[reference-baseline inventory](REFERENCE_BASELINE.md).

Do not promise "all you need is the game" until the selected default demonstrates
that claim for its supported scope. Report limitations without turning the
ordinary UI into an emulator-configuration catalog.

## Licensing and first-party engineering

Original xbx work is MIT. xemu/QEMU's root LICENSE and individual source notices
must be reviewed for the actual build [S11]. Inherited GPL code is modifiable,
but applicable distribution/source obligations remain. Keep a patch series or
maintained fork, the original notices, complete build inputs and corresponding
source paired with each public binary. C ABI separation and translating code
into Rust are not license-reset mechanisms [S12].

Verify dependency compatibility rather than importing the PSX GPLv3 bundle into
an Xbox build with potentially GPLv2-only components. Selective reuse of public
MIT Rust host code is different from reusing the entire PSX application.

Trademark assets and copyrighted startup artwork are not granted by a software
license. Use original xbx identity; no Microsoft/Xbox logos, boot animation,
commercial box-art bundle or implied endorsement. The working name is not a
formal clearance opinion. Obtain appropriate professional review for unresolved
rights/distribution questions before public release; do not disguise unknowns
as a clean-room or worldwide-lawful guarantee.

## Android

Ship signed code through normal app updates. Use scoped document/folder grants
and honest coverage for library import. Do not treat a file picker as automatic
permission for all-files access. No native-core downloader, root, special system
package changes or sideload-only setup as the normal Play product.

Verify then-current target SDK, App Bundle delivery, privacy declaration,
accessibility and content ratings at release. The seed deliberately does not
freeze a transient minimum target API from a sibling project. Android requires
both packaging and real native-runtime work for 16 KB pages [S13]; validate JIT
and memory protection boundaries as well as ELF/ZIP alignment.

Foreground services are not a general exception for silent paused games. Initial
background behavior is pause/checkpoint/stop audio. Any future service requires
an actually eligible user-visible function and its own policy review.

## Apple

Apple's review guidelines recognize retro-emulator software, subject to their
other rules [S14]. This is not a grant of game or firmware rights. Stock-iOS
runtime code-generation restrictions remain separate from whether the app
category is allowed [S15]. An ARM64 macOS build is not an iPhone result.

Keep shared Rust launch/input/session semantics and a thin native adapter.
Bundled code, public APIs, file grants, correct graphics backend and stock-device
performance all need evidence. No jailbreak, debugger-assisted JIT or exceptional
entitlement is the ordinary App Store plan. If the desired engine cannot meet
that route yet, retain the goal and state the unresolved work plainly; do not
silently turn xbx into a web shortcut or remote-streaming product.

The future user-driven Home Screen route can use a game-specific Shortcut/App
Intent into `OpenGame(GameId)`. That does not mean an iOS app can silently create
new installed apps or arbitrary persistent per-game processes.

## Release evidence

Before any public APK/AAB/IPA: identify the exact source and licenses, binary
hash/signature, complete required inputs, actual supported games/devices,
known limitations, safe data-update/recovery route and corresponding-source
availability. Check that exported source and diagnostics omit private material.
An upstream store listing or another fork's release is not xbx acceptance.
