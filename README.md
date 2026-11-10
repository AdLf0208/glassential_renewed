# glassential (Horizon port)

A Horizon port of [glassential](bigenergy/glassential-renewed) — MIT.

**Status: first slices.** What is below is what the port actually does today.

## Implemented so far

- Content API 0.2 gives a block a vanilla template's class, states and shapes,
- plus `hardness`, `explosion_resistance`, `light`, `requires_tool`,
- `friction`, `map_color`, `sounds` and `random_ticks`
- (`horizon-mods/src/content.rs:73-95`). That carries the **look, light,
- hardness, states, drops, recipes, tags and tabs**. The upstream *methods* are
- native-mod territory and are recorded as gaps:
- Ethereal / ghostly / reverse** (`EtherealGlassBlock.java:26-31`,
- `GhostGlassBlock.java:38-41`) choose the collision shape per entity. A

## Not yet ported

- Remaining behaviour is still content-only.** Redstone glass and the
- smart-glass panel are native now (slice 2); the per-entity collision
- (ethereal/ghostly/reverse), gravity glass, the colour-able painter, the
- clear-fluid face occlusion, the lava-lamp step damage and the one-way
- glasses still need hooks the pin does not have (or a block-entity renderer).
- Smart-glass panel size.** The panel flood is an unbounded six-neighbour
- BFS; upstream caps it at 1024 blocks. A panel that large would cost one
- tick of work instead of being cut off — not exercised by the e2e.

## Install

Build the bundle with a native-mod capable launcher, or add the file from the
registry once it is listed. Pinned to Horizon `1d63e0a` and horizon-mixins
`5a64af93` for game 26.3.

## Build from source

```sh
cargo test
python3 tools/horizon.py check --horizon <horizon checkout>
python3 tools/horizon.py build --horizon <horizon checkout>
python3 tools/package.py
```

## Credits and licence

Original mod: bigenergy/glassential-renewed. Port: AdLf0208. Licence: MIT — see [LICENSE](LICENSE);
the original notices are preserved.
