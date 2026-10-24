# Retained-core RAMHT compatibility boundary

The private reference at `5de6e35` aborted in the retained upstream core's
RAMHT lookup before dashboard navigation. No game ran. The owner's continuation
starts one bounded public-source correction at this boundary; the earlier kernel
and game investigations remain paused.

## Public contract and minimal candidate

The public [Nouveau RAMHT implementation at Linux v6.18](https://github.com/torvalds/linux/blob/v6.18/drivers/gpu/drm/nouveau/nvkm/core/ramht.c)
uses eight-byte entries and derives hash width from their count. Its
[FIFO register setup](https://github.com/torvalds/linux/blob/v6.18/drivers/gpu/drm/nouveau/nvkm/engine/fifo/nv04.c)
encodes the table-size field as hash bits minus nine. For the four ordinary
register encodings, this gives table bytes `2^(12 + size)` and entry-index
width `9 + size`.

The retained `03b988c9` xemu function instead used `11 + size`. With size zero
and synthetic handle `0x200` on channel zero it returns index512, whose
eight-byte offset4096 escapes a4096-byte table. Correct entry-width folding gives
index1. The candidate changes that width by two bits. It retains all lookup
assertions, handle validation and object processing; no final-index mask,
discarded GPU command or proprietary-image change is involved.

`patches/xemu-reference-ramht.patch` is an additive LGPL-2.0-or-later modification
to the retained PFIFO source. `ramht-core.lock.json` binds it separately from the
accepted reference patch and executable. The baseline build, source checkout
and crash profile are preserved. This candidate is not the selected Android
core or a shipping firmware route.

## Original regression

Rust xtask extracts the exact pinned C function and register header, compiles a
mechanical test ABI, then applies the candidate and repeats against that actual
function. Original Rust fixtures check fixed high-handle vectors for all four
table sizes, table bounds and independent bit-plane parity on channels0..15.
They make no new claim about additional channel semantics or complete hardware
accuracy. The test contains no proprietary firmware, game or guest instructions.

The original function failed the selected bounds check and the corrected function
passed all three tests. The initial shim compilation refused a redundant macro;
that complete attempt is preserved. The corrected shim uses the actual upstream
register-header macro. These are arithmetic/interface results, not dashboard,
navigation or game acceptance.

## Separate build and qualification

`cargo xtask ramht-check ROOT LABEL`, `ramht-configure`, `ramht-reconfigure`, `ramht-build` and
`ramht-observe` use the existing shared-volume binding supplied privately by the
operator. They acquire the actual whole-volume and xbx project leases, refuse a
wrong/unmounted/nonprivate directory before creating outputs, and retain the
shared12GiB floor. The candidate has one cumulative6GiB growth envelope beyond
the measured existing xbx allocation. Remaining internal-host guards are intact.

Source exports, all dependencies, compiler targets, temporary files, caches,
logs and new profiles stay in the admitted xbx directory. Exports use pinned
retained Git objects; the two non-Git xxHash wrap archives are checked against
their exact source-pin hashes. Configuration disables downloads and native build
uses one compiler job. The retained DSP library is reused, not rebuilt.
No existing source expansion, executable, profile, media or saved data is moved.

Real wrong-UUID, missing-binding, absent-workspace and occupied-capacity-lease
checks refused before creating fixture output or altering the phase receipt.
The isolated Rust tool passes Clippy with warnings denied and formatting checks;
all paused source is excluded from its build. A redundant-borrow warning was
corrected before that pass. The guard fixture's first absent-volume expected
message was corrected; the actual tool had already refused safely.

The first offline configuration reached dependency/compiler checks but exceeded
its five-minute controller deadline. Owned children were retired and that entire
partial build remains. The explicit reconfiguration route reuses the tested
source exports in a fresh `LABEL-retry1` build directory and uses a finite
fifteen-minute configuration allowance. Storage guards, package pins, source
flags and one-compiler policy are unchanged; the sources are not exported again.

Native qualification remains separate: one fresh64MiB offline no-disc profile,
the same private flash/MCPX, retained pristine open-dashboard and EEPROM seeds,
keyboard ControllerS, native FP, OpenGL/llvmpipe and dummy audio. Observe the
owned window and actual navigation, preserve exact build/profile identities,
and distinguish abort from acknowledged shutdown and storage synchronization.
Only a dashboard/navigation pass reopens the assigned unchanged-game checks.
Dummy audio and successful host exit alone do not establish game acceptance.
