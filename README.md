# Glassential Renewed for Horizon

Twenty-four full-cube glasses for Minecraft Java 26.3 on
[Horizon](https://github.com/horizon-rust/horizon), ported from
[Glassential Renewed 3.4.7](https://github.com/bigenergy/glassential-renewed)
(branch `1.21.1-neoforge`, MIT).

This is **one flat SDK package** (mod API `0.2`): `glassential.hmod` carries the content
(blockstates, models, textures, item definitions, loot, recipes, tags, creative tabs and
lang) **and** the small server half the mod's own behaviour needs.

## What it adds

| Family | Blocks |
|---|---|
| Ethereal / ghostly | `glass_dark_ethereal`, `glass_dark_ethereal_reverse`, `glass_ethereal`, `glass_ethereal_reverse`, `glass_ghostly` |
| Colour-able | `colorable_glass`, `colorable_stained_glass` |
| Light | `glass_light`, `glass_light_tinted`, `glass_lava_lamp`, `glass_lava_lamp_tinted`, `glass_glowstone_lamp`, `glass_glowstone_lamp_tinted` |
| Redstone | `glass_redstone`, `glass_redstone_tinted` |
| Smart | `smart_glass` |
| Solid-looking | `obsidian_glass`, `stone_glass`, `sandstone_glass`, `ice_glass`, `iron_glass` |
| Special | `gravity_glass`, `clear_fluid_glass`, `clear_fluid_fake_glass` |

Every block drops itself only with silk touch (upstream's `dropWhenSilkTouch`), sits in
`#c:glass_blocks` (`#c:glass_blocks/tinted` for the four tinted ones) and in
`#minecraft:impermeable`, and appears in one of the mod's two creative tabs.

**Implemented in code:** the two redstone glasses are redstone signal sources giving 15 on
every face, and a smart-glass panel (the six-neighbour connected run) is frosted while a
signal touches any cell.

## Known gaps

The upstream per-entity collision (ethereal/ghostly/reverse), gravity glass's fall behaviour,
the colour-able painter, the clear-fluid face occlusion, the lava lamp's damage and the
one-way glasses are not implemented; the colour, collision and facing behaviours the pin
cannot express as content are listed in `notes.md` under "Gaps and deviations".

## Building

```sh
cargo test                       # the pure registry model (9 tests)
cargo clippy --all-targets -- -D warnings
rm -f Cargo.lock                 # a packed lock changes the artifact
cargo horizon-mod build          # -> target/horizon-mod/glassential.hmod
cargo horizon-mod check glassential.hmod
```

The game half needs a game the launcher built with this package
(`horizon-launcher native-build ports/glassential --instance <i> --trust-native-code`);
`tools/e2e.sh` drives the scenario.

## Licence

MIT. The upstream licence and copyright notice travel in `LICENSE` and
`LICENSE-NOTICE.md`, both inside the package. The port is local-only until the maintainer
approves publishing (campaign rule, `ARCHITECTURE.md` §8).
