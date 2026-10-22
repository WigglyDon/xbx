# xbx

**Your Xbox games, ready where you left them.**

xbx is Machine Rhythm's Xbox sister project to [psx](https://github.com/WigglyDon/psx):
a separate, public, Rust-first native player with the same game-first product goal.
Choose a local game, play, leave, and return safely. A game's optional Home Screen
icon should open that game directly rather than an emulator configuration screen.

## Status

**Project foundation, not a working Xbox emulator or an APK.** No xbx core has
been incorporated, no game has been run by this seed, and no mobile/store
compatibility has been earned. See [current state](docs/state.toml).

Initial scope is the **original Xbox**, with the owner's stock Android Fold as
the first desired playable device. Xbox 360 and later generations are future
independent engine decisions, not present support. iPhone/App Store and Google
Play are distribution goals with explicit technical and licensing gates.

## Product commitments

- A small local library, direct-game launch, comfortable touch controls and
  physical-controller support through one input model; no accounts, advertising,
  analytics, uploads or online artwork service required to play.
- Durable per-game progress and automatic session resume across process death,
  app return and display changes. Xbox disk state is part of that promise.
- Rust owns substantive new application and integration policy. Mature engines
  are reusable and modifiable; a vendor directory is not an engineering barrier.
- Original work is MIT. Borrowed material retains its actual terms, attribution
  and corresponding-source obligations. This is not a claim to upstream ownership.

## Start here

| Reader | Entry |
| --- | --- |
| Fresh product Master | [Complete successor seed](handoff/MASTER_SEED.md) |
| Local Codex | [Assignment](handoff/CODEX_START.txt), then [AGENTS.md](AGENTS.md) |
| Product/design | [Product](docs/PRODUCT.md) and [architecture](docs/ARCHITECTURE.md) |
| Xbox engine selection | [Feasibility brief](docs/XBOX_FEASIBILITY.md) |
| Firmware and release | [Distribution](docs/DISTRIBUTION.md) |
| Transfer from PSX | [Reuse and separation](docs/PSX_TRANSFER.md) |
| Tests and resource safety | [Acceptance](docs/ACCEPTANCE.md), [operations](docs/OPERATIONS.md) |
| Source evidence | [Sources](docs/SOURCES.md), [candidate observations](upstream-candidates.json) |

The delivered archive's `bootstrap.sh` creates this repository publicly under
`WigglyDon/xbx` through the owner's local GitHub CLI, pushes this foundation,
and creates four issue scopes. [Local setup](handoff/LOCAL_SETUP.md) explains
exactly what it changes. `docs/github-state.toml` becomes the actual bootstrap
receipt; its initial form deliberately makes no remote-creation claim.

Once published:

```sh
git clone https://github.com/WigglyDon/xbx.git
cd xbx
bash scripts/start-codex.sh
```

There is deliberately no speculative Cargo framework or fake emulator in this
seed. The first implementation must establish a real engine boundary and build
one useful executable slice, not manufacture a test badge for an empty player.

## Independence

xbx and psx have separate application IDs, signing identities, saves, processes
and releases. Selective reuse of licensed public PSX code is encouraged; copying
private files, inherited console assumptions or a second active checkout's work
is not. Xbox is a Microsoft trademark. This independent project is not affiliated
with or endorsed by Microsoft. The working name is not a trademark clearance.
