# Glassential Renewed — a Horizon port

**Glassential Renewed** is a Minecraft mod. Various types of glass to aid in your building. Updated to new MC. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [bigenergy/glassential-renewed](https://github.com/bigenergy/glassential-renewed) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Glassential Renewed ([source](https://github.com/bigenergy/glassential-renewed)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `glassential.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- **Ethereal / ghostly / reverse** (`EtherealGlassBlock.java:26-31`,
- **Gravity glass** (`GravityGlassBlock.java:29-73`) clamps downward velocity,
- **Redstone glass** (`RedstoneGlassBlock.java:20-30`) is a signal source
- **Smart glass** (`SmartGlassBlock.java:36-141`) declares the real `powered`
- **Colour-able glass** (`ColorableGlassBlock.java:36-155`) declares the real
- **Clear fluid glass** (`BigGlassBlockEntity.java:12-70`) declares the real
- **Lava lamp** (`CustomGlassBlock.java:63-92`) emits 15, but the step damage
- **Obsidian glass** keeps `hardness 1`, `explosion_resistance 1200` and

## Not yet ported

- **Remaining behaviour is still content-only.** Redstone glass and the
- **Smart-glass panel size.** The panel flood is an unbounded six-neighbour
- **The panel updates once per level tick**, after `level_tick_post`, not
- **One-way glasses deferred** (custom baked model); panes, doors, trapdoors,
- **Fusion is kept.** Horizon has a Fusion implementation; the port relies on
- **Item icons are cropped tiles.** The upstream item models inherit the CTM
- **Tinted glasses need silk touch here.** Upstream's loot table uses
- **No tooltips.** Upstream's `appendHoverText` lines (`tooltip.glassential.*`)

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `glassential.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Glassential Renewed was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/bigenergy/glassential-renewed

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run glassential` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/glassential/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
