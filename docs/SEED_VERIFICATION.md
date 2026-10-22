# Seed verification

This checks the **foundation and local bootstrap**, not any Xbox implementation.
No live GitHub repository was created from this chat; no Codex worker, emulator,
firmware build, APK or device test ran as part of the following checks.

## Executed checks

Bash syntax checks passed for the archive bootstrap and both repository scripts.
The payload checksum manifest was verified. Repository-local Markdown links and
JSON/TOML metadata were checked.

The startup harness used a mocked GitHub CLI, an inert Codex launcher and actual
local bare Git repositories. All nine cases passed:

1. fresh creation/push/4 issues/receipt/launch; identical retry has no duplicate issues or commit.
2. failure after accepted issue creation; retry inspects and reuses it.
3. wrong account refuses without checkout or remote creation.
4. unrelated nonempty destination retained and refused.
5. non-404 lookup failure does not trigger repo creation.
6. existing private repository is not exposed or converted.
7. unrelated remote main refuses with no force push or issues.
8. dirty retry preserves local work and does not publish it.
9. altered seed payload is rejected before checkout/publication.

The harness caught an origin-inspection issue caused by Git URL rewriting; the
final script validates the stored origin and additionally verifies the published
commit through GitHub's API. The corrected harness passed. This is simulated
provider verification, not a claim that GitHub accepted these writes in this chat.

Actual publication still depends on the owner's installed GitHub CLI, normal
authentication/permissions and network. The scripts preserve work and stop on
unresolved errors. Read the generated bootstrap receipt after a successful run.

No third-party source, proprietary input or credentials are included in this seed.
