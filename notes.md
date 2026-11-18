# Glassential Renewed — port notes

Ported from **Glassential Renewed 3.4.7**
(`bigenergy/glassential-renewed`, branch `1.21.1-neoforge`, commit
`0387cd173788d5348efdfaf80ffb129ad8ea56d2`, 2026-09-13) for Minecraft Java
26.3 on Horizon **c9d474d** / horizon-mixins **d9c83d7**.

**Licence: MIT** (`gradle.properties:24`, `LICENSE`). The textures, models,
blockstates, lang and `.mcmeta` files are the upstream mod's own MIT assets;
the Rust registry model is a clean-room rewrite. Publishing still needs the
maintainer's approval (ARCHITECTURE.md §8).

Status: **slice 1 implemented and verified in-game.** Slice 1 is the **24
full-cube glasses**: the ethereal/ghostly/tinted-ethereal set, luminous and
redstone glass, smart and colour-able glass, stone/sandstone/ice/iron/
obsidian/gravity glass, the two clear-fluid glasses and the four lamps. The
two **one-way glasses are deferred** because their block model needs a custom
baked model (`glassential:one_way_loader`); panes, doors, trapdoors, the slab
and the glass painter are later slices. The staged ladder was blocked by the
campaign lock (see Verification); the standalone crate, the content `.hmod`
and the in-game e2e are green.

> **0.2.0 (2026-11-18): republished as one flat SDK `.hmod`.** Horizon's mod-API merge retired
> `.hnmod` and the registry closed this port's earlier submission **PR #867** by that purge. The
> behaviour, the carried content and the pure model are unchanged; the glue, the manifest and the
> packaging are new. Read [`## Migration to the SDK format`](#migration-to-the-sdk-format-020-2026-11-18)
> at the end of this file for the file map, the helper→hook table and the new traps. The in-game
> e2e has **not** been re-run: `horizon-launcher` is not installed here and no game exists at the
> SDK pin, so the last verified runs are the retired bundle's (see "Not re-run" in that section).

## Upstream analysis

Full detail and the source references are in
[`../_shared/upstream/glassential/FINDINGS.md`](../_shared/upstream/glassential/FINDINGS.md).
The analysed branch is the pack's 1.21.1 line. A later branch
`26.3-neoforge` (commit `7b72383`, version **5.2.0**) was read only to confirm
26.3's recipe/item-definition formats; it removed Smart Glass, so it is not
the port's source.

### Variant table

| # | id | template | light | states | tinted | tab |
|---|---|---|---:|---:|---|---|
| 1 | `glass_dark_ethereal` | `minecraft:tinted_glass` | 0 | 1 | – | functional |
| 2 | `glass_dark_ethereal_reverse` | `minecraft:tinted_glass` | 0 | 1 | – | functional |
| 3 | `glass_ethereal` | `minecraft:glass` | 0 | 1 | – | functional |
| 4 | `glass_ethereal_reverse` | `minecraft:glass` | 0 | 1 | – | functional |
| 5 | `glass_ghostly` | `minecraft:glass` | 0 | 1 | – | functional |
| 6 | `colorable_glass` | `minecraft:glass` | 0 | 2 (`lit`) | – | functional |
| 7 | `colorable_stained_glass` | `minecraft:white_stained_glass` | 0 | 2 (`lit`) | – | functional |
| 8 | `glass_light` | `minecraft:glass` | **15** | 1 | – | functional |
| 9 | `glass_light_tinted` | `minecraft:tinted_glass` | **15** | 1 | ✓ | functional |
| 10 | `glass_redstone` | `minecraft:glass` | 0 | 1 | – | functional |
| 11 | `glass_redstone_tinted` | `minecraft:tinted_glass` | 0 | 1 | ✓ | functional |
| 12 | `smart_glass` | `minecraft:glass` | 0 | 2 (`powered`) | – | functional |
| 13 | `obsidian_glass` | `minecraft:glass` | 0 | 1 | – | functional |
| 14 | `stone_glass` | `minecraft:glass` | 0 | 1 | – | blocks |
| 15 | `sandstone_glass` | `minecraft:glass` | 0 | 1 | – | blocks |
| 16 | `ice_glass` | `minecraft:glass` | 0 | 1 | – | blocks |
| 17 | `iron_glass` | `minecraft:glass` | 0 | 1 | – | blocks |
| 18 | `gravity_glass` | `minecraft:glass` | 0 | 1 | – | functional |
| 19 | `clear_fluid_glass` | `minecraft:glass` | 0 | **64** (`up/down/east/west/north/south`) | – | functional |
| 20 | `clear_fluid_fake_glass` | `minecraft:glass` | 0 | **64** (same six) | – | functional |
| 21 | `glass_lava_lamp` | `minecraft:glass` | **15** | 1 | – | blocks |
| 22 | `glass_lava_lamp_tinted` | `minecraft:tinted_glass` | **15** | 1 | ✓ | blocks |
| 23 | `glass_glowstone_lamp` | `minecraft:glass` | **15** | 1 | – | blocks |
| 24 | `glass_glowstone_lamp_tinted` | `minecraft:tinted_glass` | **15** | 1 | ✓ | blocks |

State total: 19 single + 2 + 2 + 2 + 64 + 64 = **153**.
Templates: 17 `minecraft:glass`, 6 `minecraft:tinted_glass` (the four
`GLASSES_TINTED` plus the dark-ethereal pair, which blocks light), one
`minecraft:white_stained_glass`.

## Behaviour and the content-schema boundary

Content API 0.2 gives a block a vanilla template's class, states and shapes,
plus `hardness`, `explosion_resistance`, `light`, `requires_tool`,
`friction`, `map_color`, `sounds` and `random_ticks`
(`horizon-mods/src/content.rs:73-95`). That carries the **look, light,
hardness, states, drops, recipes, tags and tabs**. The upstream *methods* are
native-mod territory and are recorded as gaps:

- **Ethereal / ghostly / reverse** (`EtherealGlassBlock.java:26-31`,
  `GhostGlassBlock.java:38-41`) choose the collision shape per entity. A
  content block cannot override `getCollisionShape`, so all three are solid
  in this slice.
- **Gravity glass** (`GravityGlassBlock.java:29-73`) clamps downward velocity,
  scales fall distance and grants slow falling; it is a plain glass here.
- **Redstone glass** (`RedstoneGlassBlock.java:20-30`) is a signal source
  returning 15; a content block cannot set `isSignalSource`, so it emits no
  redstone signal.
- **Smart glass** (`SmartGlassBlock.java:36-141`) declares the real `powered`
  state and both models, but nothing recomputes the panel; `powered` only
  changes when set explicitly.
- **Colour-able glass** (`ColorableGlassBlock.java:36-155`) declares the real
  `lit` state and the model, but the block entity, painter, tint, light and
  pass-through flags are absent.
- **Clear fluid glass** (`BigGlassBlockEntity.java:12-70`) declares the real
  six booleans and the multipart blockstate, but the placement/neighbour
  update that sets them and the fluid-face occlusion are absent; setblock
  binds all 64 states.
- **Lava lamp** (`CustomGlassBlock.java:63-92`) emits 15, but the step damage
  and the water-above bubble column are absent.
- **Obsidian glass** keeps `hardness 1`, `explosion_resistance 1200` and
  `requires_tool`; its tooltip is not carried.
- **Ice glass** (`IceGlassBlock.java:20-29`) is a glass that never melts; the
  glass template never random-ticks, so the effect is preserved without a
  rule. Upstream copies glass' friction, so it is not slippery.
- **One-way glass** and **tinted one-way glass** are deferred: their model is
  `glassential:one_way_loader` (a custom baked model), and the block entity
  holds a mimic state.

## Content generation and asset adaptation

`tools/gen_content.py` reproduces the tree from the upstream checkout:

- **Assets copied byte for byte:** every blockstate, the Fusion block models,
  the lamp parent model, the clear-fluid multipart pieces, the textures and
  their `.mcmeta`. Horizon implements Fusion connected textures
  (`horizon-client-render/src/fusion.rs`), so the CTM sheets stay as the
  author wrote them.
- **26.3 item definitions:** `assets/glassential/items/<id>.json` names an
  item model. For Fusion glasses the icon cannot use the CTM sheet (the item
  renderer has no Fusion pass), so the generator crops the sheet's plain
  top-left tile to `textures/block/<id>_item.png` and makes a `cube_all` item
  model. This is an adaptation of the author's own texture, not a new one.
  Lamps keep their element model; the clear-fluid item keeps its 16x16 item
  texture.
- **Recipes:** the 24 upstream recipes are converted from 1.21.1's
  `{"item": X}` / `{"tag": T}` objects to 26.3's `"X"` / `"#T"` strings, the
  same upgrade the author's 26.3 branch applies.
- **Loot:** one 26.3 pool per glass with
  `"condition": "minecraft:tool/can_silk_touch"` — upstream's
  `dropWhenSilkTouch` for every block (`GlassentialBlockLoot.java:19-31`).
- **Tags:** `minecraft:impermeable`, `c:glass_blocks` (all 24) and
  `c:glass_blocks/tinted` (the four `GLASSES_TINTED`). Upstream's datagen
  never puts `obsidian_glass` in a mineable tag even though it sets
  `requiresCorrectToolForDrops`; without one no tool is "correct", so the port
  adds `minecraft:mineable/pickaxe` for that one block (a datagen deviation,
  proved by the e2e's silk-touch drop).
- **Tabs:** upstream's two tabs, `glassential:glassential_tab` (Functional,
  16 entries) and `glassential:glassential_tab_misc` (Blocks, 8 entries),
  with the upstream titles and icons.
- **Lang:** the 24 upstream `block.glassential.*` names and the two tab keys.

Skills: superpowers/using-superpowers, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

## Verification

### Unit tests (standalone crate)

`cargo test --locked --offline`: **9 tests, 9 passed**; `cargo clippy
--all-targets --locked --offline -- -D warnings`: 0 warnings; `cargo fmt
--all --check`: clean. The tests pin the 24 ids, the 153-state total, the six
luminous glasses, the four tinted ids, the dark-ethereal pair's tinted
template, silk-touch-only loot, the 16/8 tab split and the deferred one-way
pair.

### Ladder (staged game, pin `83ad384f`)

Recipe: `HORIZON_MIXINS=target/horizon-mixins`,
`HORIZON_STAGE_ROOT`/`HORIZON_TARGET_DIR` on `/home` (the port's own stage and
target), `CARGO_INCREMENTAL=0`, every step through
`ports/tools/build-slot.sh`. Logs: `target/repin-ladder.log` (step exit codes)
and the per-step `target/horizon-{stage,check,test,build}.log`.

| step | command | result |
|---|---|---|
| stage | `tools/horizon.py stage` | exit 0, `stage_changed: true`, revision `83ad384f…`, 5 injections + 2 modules |
| check | `tools/horizon.py check` | exit 0, **0 warnings** |
| test | `tools/horizon.py test` | exit 0, **9 passed; 0 failed; 913 filtered out**, **0 warnings** (`target/horizon-test.log`, 22:38) |
| build | `tools/horizon.py build` | TBD |
| ctl | `tools/horizon.py ctl` | TBD |

The four warnings the old `horizon-test.log` carried are gone; see the warning
fix below.

### Content mod

`cargo horizon-mod build` (from `content/`) → exit 0,
`target/horizon-mod/glassential.hmod`. `cargo horizon-mod check` → exit 0:
`glassential 0.1.0 (mod, API 0.2, both)`, **111,168 bytes**, SHA-256
`e9134c09…41c1`, "no code: content only", 246 entries.

### Port version `0.1.0` (2026-11-10, manifest-only change)

The upstream reviewer's rule (`AGENTS.md` §1.7) is that the registry version is
the **port's** version, not the original mod's: a partial port ships `0.1.0`.
The version lived in three places and now agrees everywhere — the entry, the
files and the release:

- `content/horizon-mod.toml` `version` (read by `cargo horizon-mod build` into the `.hmod` manifest)
- `mixins/glassential/horizon-mixin.toml` `[mod] version` (read by `tools/package.py` into `horizon-native.json`)
- `Cargo.toml`/`Cargo.lock` (the standalone crate's own version; not packaged)

`tools/gen_content.py` has no version constant, so nothing else changed. No
behaviour source, asset, data file or hook changed, so **no e2e re-run was
needed** — the previous `PASS`/`EXIT:0` legs below still describe this content.
Repackaged with `tools/package.sh` through a build-slot lease, twice, and both
runs gave byte-identical artifacts:

| artifact | size | SHA-256 | entries |
|---|---:|---|---:|
| `content/glassential.hmod` | 111,168 B | `e9134c09e528856c9d56b035b07e4b570462cab70859a74d04ece759ff1141c1` | 246 |
| `glassential.hnmod` | 76,387 B | `56465263dab5a22776ef40a4df67dc11c9af375cc4b26cd376cf233dd19d4221` | 10 |

The bundle's `mixins/glassential/data/glassential-content.hmod` is byte-identical
to the built `.hmod` (same SHA-256; `tools/package.py` refuses a stale copy) and
the embedded copy inside the `.hnmod` hashes to the same bytes.
`cargo horizon-mod check … --kind mod` and `… --kind native-mod` both exit 0 on
these files; `cargo fmt --check`, `clippy -D warnings` and the 9 unit tests stay
clean at the new crate version.

### In-game e2e (`tools/e2e.sh`, offscreen renderer, The End)

| Run | Result |
|---|---|
| default | exit 0: 24/24 default states + 3 boolean states + 128/128 clear-fluid states (**153 distinct**); 6/6 luminous glasses emit 15 and the two controls 0; dark ethereal blocks a glowstone's light (plain glass reads 14, both dark glasses 0); vanilla tags (`#minecraft:impermeable`, `#minecraft:mineable/pickaxe`) with stone/ethereal negatives and the built `c:` table (24/4); without silk our four glasses and vanilla glass drop 0 while vanilla tinted glass drops 1; with silk all six drop themselves (including obsidian glass); the four recipes craft and the stick controls stay empty; 24/24 items give; the two tabs list 24 with their titles; 3/3 block items place; unknown ids and undeclared states are refused (`/tmp/opencode/glassential-e2e-prelim6.log`) |
| counterfactual (`COUNTERFACTUAL=1`) | exit 0; vanilla glass drops nothing without silk and itself with silk, vanilla tinted glass drops itself without silk, and none drops a Glassential item (`/tmp/opencode/glassential-e2e-cf.log`) |
| negative (`COUNTERFACTUAL=1 NEGATIVE=1`) | exit 1, as required: the run asserts a Glassential drop from vanilla tinted glass, observes none, and fails (`NEGATIVE run did not observe the false assertion (Glassential drops=0)`; `/tmp/opencode/glassential-e2e-negative.log`) |

The default run's differentials: vanilla tinted glass drops itself without
silk while `glassential:glass_light_tinted` does not (so the port's loot
table, not the harness, is what makes the drop 0); plain glass lets the
glowstone's light into its own cell (14) while both dark-ethereal glasses
read 0; stone is not in `#minecraft:impermeable` and `glass_ethereal` is not
in `#minecraft:mineable/pickaxe`.

Harness note: `execute if block … #c:glass_blocks` is refused at the pin
because the command registry's tag-id set is the prepared vanilla one (gap
10); the e2e therefore reads the convention tags from the built
`block-tags.json` and keeps the runtime tag checks on vanilla tag ids.

### Default e2e on the `0.1.0` bytes (2026-11-10)

The bundle and the content `.hmod` are byte-identical to the slice-2 artifacts
(hashes above), so this leg re-runs the **default** scenario on exactly those
bytes; the only default log on disk was red.

| Run | Result |
|---|---|
| default | **PASS + `EXIT:0`** (`../_shared/evidence/glassential/e2e.log`): every phase of the table above, now including the two creative tabs (24 entries, both titles) and **3/3** block placements |
| counterfactual / negative | not re-run, unchanged: PASS + `EXIT:0` (`e2e-counterfactual.log`) and FAIL + `EXIT:1` by design (`e2e-selftest.log`). The fix below is in the default-only section, after the counterfactual's `exit 0` |

**Symptom.** `e2e-before-fix.log` (kept) stopped at
`FAIL: no creative-inventory.json at …/.horizon-cache/mod-content/creative-inventory.json`
after 153/153 states, the light, redstone and smart-glass phases, the silk-touch
differential, the four recipes and 24/24 give-able items: the tab listing and
the placement phase never ran.

**Root cause.** The tab phase still resolved the **pre-stamp** content root. The
pin builds one root per mod set — `mod_content::resource_root` writes into
`cache/mod-content/<stamp16>/` (`ROOT_NAME = 16`, `docs/design/mod-hotload.md`),
`build`/`tables` rewrite `creative-inventory.json` there and
`Plan::creative_tabs` appends the mod tabs
(`crates/horizon-client/src/mod_content.rs`) — so the merged table is one level
below where the phase looked. The tag phase already used the stamp layout (its
`-maxdepth 2` find found `block-tags.json` in the same root, which is why the run
got that far). It is **not** a race and **not** a pruned root: the tables are
written during startup, before `ctl wait-ready`, each run gets a fresh scratch
`--game-dir`, and a sibling port's cache at the same pin holds the merged file
with its own mod tab.

**Fix (harness only, `tools/e2e.sh`).** The phase resolves the table with
`find … -maxdepth 2 -name creative-inventory.json -print -quit` — the per-stamp
layout at depth 2, the old in-place root at depth 1 — waiting up to 30 s in
one-second steps so a slow build cannot race it; `-print -quit` also avoids the
`find | head` SIGPIPE that `set -o pipefail` would turn into a failure. No
assertion was weakened: 24 items, every id and both tab titles are still
required, and the failure message now lists what `mod-content` holds.

**Infra note.** Two earlier attempts died for a reason unrelated to the port:
the runner closed its lease fd in the parent shell, so two jobs shared a slot, one
deleted the other's `stage/modded`, and cargo reported `could not parse/generate
dep info … (os error 2)` / a missing `stage/modded`. AGENTS.md now forbids that
fd close; after the runner fix the same command ran green. Those attempts are not
evidence.

`python3 ports/tools/check-e2e-logs.py glassential --strict` → exit 0, all four
logs accepted (`e2e.log` PASS + `EXIT:0`). Artifacts untouched by this work and
re-hashed afterwards: `glassential.hnmod` `56465263…d4221` (76,387 B) and
`content/glassential.hmod` `e9134c09…41c1` (111,168 B); nothing repackaged.

## Re-pin to 83ad384f and slice 2: redstone and smart glass (2026-11-06)

**Class: content · tier: data** (`ports/ledger.csv`, per
`horizon/docs/modpacks.md`). The data tier is the 24-block carried-over content
(assets, models, states, loot, recipes, tags, tabs). This slice adds the
**server half** the mod's own behaviour needs — the redstone signal answers and
the smart-glass panel state — which is logic-shaped work; it is recorded here as
an extra half rather than changing the ledger's tier.

Migrated from `c9d474d…`/`d9c83d7…` to Horizon
`83ad384f48d0742746d518737511882680aab5b7` and horizon-mixins
`e8cd0aa38cffef49cb2e088ef716baa107148c02`:
`compatibility.json` (`horizon_revision`, mixins revision, `game.rs` SHA-256
`996432b9…`), the manifest's `[compatibility] horizon_revision`,
`tools/horizon.py` (`--lib` embedded-test target, a `ctl` action building
`-p horizon-client-lib --bin horizon-ctl`, and the shared-cache stage/target
overrides). No content file changed, so the delivered `.hmod` is unchanged.

### The four warnings (0-warning rule)

`target/horizon-test.log` at 16:44 reported four warnings; the fresh run has
**0**. All four were in the injected `glassential_state` module:

| warning | fix |
|---|---|
| `is_signal_source` exposes private `game::GameLevel<'_>` | `pub(crate)` → `pub(super)`, the widest visibility that can name a `game`-private type, with a comment saying why |
| `block_signal` exposes private `game::GameLevel<'_>` | same |
| `on_level_tick_post` exposes private `game::Game<'_>` | `pub(crate)` → `pub(super)` |
| `on_placed` is never used | removed: a placed glass already fires `block_changed`, so the dirty set is complete without it |

No `#[allow]` was added.

### Behaviour (stamped hooks only)

| Hook (API) | Helper | What it does |
|---|---|---|
| `is_signal_source` (38) | `signal_source.rs` | the two redstone glasses answer as sources, so wire connects to them |
| `block_signal` (38) | `block_signal.rs` | 15 on every face, direct and ordinary |
| `block_changed` / `neighbor_changed` | `changed.rs` | record the cell and its six neighbours as dirty |
| `level_tick_post` | `level_tick.rs` | rebuild each dirty smart-glass panel once per level tick |

`glass_redstone`/`glass_redstone_tinted` answer 15 on every face
(`RedstoneGlassBlock.java:20-30` upstream). A signal touching one pane of a
smart-glass run floods its six-neighbour connected component and writes the real
`powered` state on every pane (that state selects the frosted model); removing
the signal clears the panel again.

### In-game e2e — redstone + smart glass

TBD

## Gaps and deviations

1. **Remaining behaviour is still content-only.** Redstone glass and the
   smart-glass panel are native now (slice 2); the per-entity collision
   (ethereal/ghostly/reverse), gravity glass, the colour-able painter, the
   clear-fluid face occlusion, the lava-lamp step damage and the one-way
   glasses still need hooks the pin does not have (or a block-entity renderer).
2. **Smart-glass panel size.** The panel flood is an unbounded six-neighbour
   BFS; upstream caps it at 1024 blocks. A panel that large would cost one
   tick of work instead of being cut off — not exercised by the e2e.
3. **The panel updates once per level tick**, after `level_tick_post`, not
   inside vanilla's neighbour update; a powered change is visible one tick
   later than upstream's. Same-tick queries (a comparator) are not served.
4. **One-way glasses deferred** (custom baked model); panes, doors, trapdoors,
   slab, dyed doors/trapdoors and the glass painter are later slices.
5. **Fusion is kept.** Horizon has a Fusion implementation; the port relies on
   it rather than flattening the models to plain cubes. The block models name
   the CTM sheets, so the connected look depends on that renderer path (the
   e2e does not check pixels).
6. **Item icons are cropped tiles.** The upstream item models inherit the CTM
   block model; the item renderer has no Fusion pass, so the generator crops
   tile 0 to a 16x16 icon. Recorded as an adaptation.
7. **Tinted glasses need silk touch here.** Upstream's loot table uses
   `dropWhenSilkTouch` for every block, including the tinted ones, while
   vanilla tinted glass drops itself. The e2e proves the difference.
8. **No tooltips.** Upstream's `appendHoverText` lines (`tooltip.glassential.*`)
   are client code; content 0.2 has no item tooltip component.
9. **Map colour** is the template's (glass has no map colour), matching
   upstream's `ofFullCopy(Blocks.GLASS)`; not visually checked.
10. **`/recipe give` does not list mod recipes** at the pin (the recipe index
   reads `data/minecraft/recipe` only); the crafting path is the evidence, as
   in the Dyenamics port.
11. **The pack's installed jar version was not in the campaign tree.** The
   analysed source is the pack's 1.21.1 line, 3.4.7; the version is recorded
   as such.
12. **Mod tag ids are not known to the command parser at the pin.** The
    content build writes `c:glass_blocks` and `c:glass_blocks/tinted` into
    `block-tags.json` (and the gameplay tag loader reads the files), but the
    command registry's tag-id set comes from the prepared vanilla
    `registries.json`, so `execute if block … #c:glass_blocks` reports
    "Unknown block tag" even though the tag exists. The e2e checks the
    vanilla tags (`#minecraft:impermeable`, `#minecraft:mineable/pickaxe`)
    at runtime and the convention tags in the built table. Issue shape: mod
    tag ids in the command registry (the same gap affects any content mod's
    new tags).

## Files

```
Cargo.toml, Cargo.lock, compatibility.json, LICENSE, rust-toolchain.toml,
.gitignore
mixins/glassential/horizon-mixin.toml   1 module, 0 injections
mixins/glassential/runtime.rs           test-only glass registry, 9 tests
content/horizon-mod.toml                API 0.2, registries, both
content/glassential.hmod                delivered content mod (111,168 B,
                                        SHA-256 e9134c09…41c1, 246 entries)
content/assets/glassential/             24 blockstates, 24 block models,
                                        24 item models + icons, 24 26.3 item
                                        definitions, textures + Fusion mcmeta,
                                        lang
content/data/glassential/horizon/       24 block declarations, 2 tabs
content/data/glassential/loot_table/    24 silk-touch loot tables
content/data/glassential/recipe/        24 recipes
content/data/minecraft/tags/            impermeable, mineable/pickaxe
content/data/c/tags/                    glass_blocks, glass_blocks/tinted
tools/gen_content.py                    reproduces the content from upstream
tools/craft_check.py                    the four recipe checks
tools/e2e.sh                            default/counterfactual/negative runs
tools/horizon.py                        stage/check/test/build ladder
notes.md
```

## Skills used

- Superpowers: `test-driven-development` (the registry test was written first
  with an empty table: 6 of 9 failed, then the 24 entries made all 9 pass),
  `systematic-debugging` (the generator's missing lamp/clear-fluid model
  paths, the item-icon Fusion gap, and the bash arithmetic in the state
  loop), and `verification-before-completion` (every claim re-run and logged).
- Beautiful Rust:
  `ports/_shared/skills/BeautifulRust/skills/rust-best-practices/SKILL.md`;
  `cargo fmt`, `cargo clippy --all-targets --locked --offline -- -D warnings`
  and `cargo test --locked --offline` are clean.

## Content embedded in the bundle; re-verified in game (2026-11-10)

**Beat: one file per project.** The registry takes one `file` per version, so a
port that ships a `.hmod` *and* a `.hnmod` cannot be listed faithfully. This port
now follows the AE2 pattern proved on `charginggadgets` (PR #526): the built
content mod travels **inside** the bundle at
`mixins/glassential/data/glassential-content.hmod`, and `mixins/glassential/embedded_content.rs`
compiles it in with `horizon_client::native_data!` and injects it at
`content_root` (`target = "horizon_client::game::mods::content_root"`,
`kind = "HEAD"`, `cancellable = true`). The helper writes the file to
`.horizon-cache/glassential-embedded-content.hmod` once, appends it to the dev-mod list
and returns the merged root; the recursive call stops at its own
`dev.contains(&file)` guard, so the vanilla body runs once. The `.hmod` stays in
`content/` as the build input; `tools/package.py` refuses a stale copy in `data/`.

**Proved with no `--dev-mod`:** the e2e launches the game *without* passing the
content folder, so every content id and every behaviour it asserts comes from the
bundle. `EMBEDDED=1` is the flag; the launch line passes an empty dev-mod list.

| Artifact | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `content/glassential.hmod` (input) | 111,168 B | `e9134c09e528856c` | 246 |
| `glassential.hnmod` (deliverable, embeds it) | 76,387 B | `56465263dab5a227` | 10 |

E2E (one lease each, `CARGO_INCREMENTAL=0`, shared `1d63e0a5` `horizon-ctl`, the
port's own fast game): default `PASS` + `EXIT:0`; `COUNTERFACTUAL=1` `PASS` + `EXIT:0`; `COUNTERFACTUAL=1 NEGATIVE=1` `FAIL` + `EXIT:1`.

`python3 ports/tools/check-e2e-logs.py glassential --strict` → every log carries its
own `PASS`/`FAIL` + `EXIT:` pair. `python3 ports/tools/upstream_ready.py glassential` →
**READY (exit 0)**.

## Review repairs (PR #617)

- **[P1] staging** is the orchestrator's (horizon-mods#686).
- **[P2] attribution** — the entry links
  `https://github.com/bigenergy/glassential-renewed` and credits
  `bigenergy aka Big_Energy`.
- **Item texture namespaces** — all 20 non-lamp item models had
  `"sprite": "block/…"`, which resolves to `minecraft:block/…`; every sprite is
  now `glassential:block/…`, so the carried textures resolve.
- **Notices** — `mixins/glassential/LICENSE` and `NOTICE.md` travel inside the
  `.hnmod`, and both sit at the embedded content archive's root.
- **Smart-panel flood bounded** — the component walk is capped at **1024**
  blocks (upstream's limit) and one level tick rebuilds at most 32 panels,
  keeping the rest marked for the next tick. Gap 2 below is closed.

### Verification after the repairs

- Artifact: `glassential.hnmod`, 79,872 B, SHA-256 `6980d7bc499d4de5aee240870140c860a454908fbd2d4a0ab01099bcf658576b` (rebuilt twice, identical bytes; `tools/package.py`; the embedded content equals `content/glassential.hmod`).
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked --offline`: clean, 9 passed; the embedded run (`python3 tools/horizon.py test --horizon ports/_shared/pins/horizon-1d63e0a`) is 9 passed, 0 failed (`target/horizon-test.log`).
- `build-slot.sh e2e --job … --port ports/glassential -- tools/e2e.sh`: PASS, log `ports/_shared/evidence/glassential/e2e-repair.log` ends `EXIT:0` (`check-e2e-logs.py glassential --strict` exit 0).
- `python3 ports/tools/upstream_ready.py glassential` and `check-skills.py --notes ports/glassential`: exit 0.

## Migration to the SDK format (0.2.0, 2026-11-18)

*Format migration only: the behaviour, the carried content and the pure model are unchanged.
This port's former submission, **PR #867** (`glassential` 0.1.0, one `.hnmod` embedding its
content), was closed on 2026-11-18 by the SDK purge; **0.2.0** is the same port as one flat
`.hmod`. The changelog text is at the end of this section.*

**Why.** Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired the `.hnmod`
container; the registry closed every `.hnmod` submission with *"republish this as a native
`.hmod` with the SDK from the next Nightly"*, and an `.hnmod` no longer loads. The new unit is
**one flat `.hmod`**: content, native code, or both. For this port that removes the AE2
embedded-content pattern entirely — the package carries `assets/` and `data/` itself, so there
is no second archive, no `.horizon-cache` write and no `content_root` injection.

**Pins.** Horizon / SDK **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (nightly 2026-11-18,
`package_format` **6**, `content_api` `"0.2"`), horizon-mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**. The SDK is cached in-tree at
`../_shared/sdk/0b14f8a/` and fetched into `.horizon/sdk/` by `cargo horizon-mod`; it is never
packed. The retired tree stays as evidence in
`../_shared/evidence/glassential/retired-hnmod-0.1.0/` (moved there **before** the first build;
`pre-move-tree.sha256` records the tree it came from).

**Version: `0.2.0`** — a resubmission is always a new version, and the retired bundle was
`0.1.0`.

**Class + tier unchanged: class `content`, tier `data`** (`ports/ledger.csv`,
`atm10-sources.csv`). Approach from `docs/modpacks.md`: §"4. Carry-over: data and assets" is
the bulk of the job (the 24 block declarations, models, textures, item definitions, loot,
recipes, tags, tabs and lang are carried byte for byte), and §"5. Code: one agent per mod" is
the small `src/` crate that keeps the registry model testable. The **server half** — the two
redstone glasses and the smart-glass panel — was already this port's declared extra half at
0.1.0 (see "Re-pin to 83ad384f and slice 2"); the tier is not re-tiered.

### Files created, files deleted, and where the retired tree is

| Created | What it holds |
|---|---|
| `horizon-mod.toml` | the only manifest and identity: `id = "glassential"`, `version = "0.2.0"`, `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`, `license`/`homepage`/`issues`, and `[native]` with the exact three pins |
| `Cargo.toml` | one crate (`mod-glassential`); its only dependency is `horizon-mod-api` from `.horizon/sdk/` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks` (register, block_changed, level_tick_post) and `pub mod runtime;` |
| `src/support.rs` | the game glue: the dirty-cell set, the two `BlockPart`s and the panel rebuild |
| `src/runtime.rs` | the retired `runtime.rs`, **byte-identical** (`cp`, then `cmp`; SHA-256 `f0804f5f…33d7`, 396 lines) |
| `assets/glassential/**`, `data/**` | the retired content package's own tree, **moved** to the package root: 245 files (167 assets + 78 data), byte-identical |
| `README.md` | rewritten for the flat layout (it is packed) |
| `.gitignore` | `/target`, `/.horizon`, `/Cargo.lock` |
| `glassential.hmod` | the delivered package, at the port root (never `target/`) |

| Deleted / moved away | Why | Where it went |
|---|---|---|
| `mixins/glassential/` (`horizon-mixin.toml`, `runtime.rs`, `support.rs`, `block_signal.rs`, `changed.rs`, `embedded_content.rs`, `level_tick.rs`, `signal_source.rs`, `data/glassential-content.hmod`) | the plan/injection schema, per-hook helpers and the embedded copy are not part of an SDK package | `runtime.rs` → `src/`; `support.rs` → `src/support.rs` (rewritten as typed methods); the four hook helpers → `BlockPart`/`Hooks` methods; `embedded_content.rs` → deleted (the package carries the content); all of it to `retired-hnmod-0.1.0/mixins/` |
| `content/` (`horizon-mod.toml`, `glassential.hmod`, `assets/`, `data/`, `target/`) | a second package; the flat package carries the content itself | `assets/` + `data/` moved to the root; the old manifest and the 0.1.0 `.hmod` to `retired-hnmod-0.1.0/` |
| `glassential.hnmod` (79,872 B, `6980d7bc…`) | the retired container; unloadable on launcher ≥ 0.2.70 | `retired-hnmod-0.1.0/` |
| `compatibility.json` | `[native]` in the manifest is the pin now | `retired-hnmod-0.1.0/` |
| `rust-toolchain.toml` (1.95.0) | not part of any SDK package; the pinned SDK decides the toolchain (1.99.1) | `retired-hnmod-0.1.0/` |
| `tools/horizon.py`, `tools/package.py`, `tools/package.sh`, `tools/add-notices.py` | staging, `.hnmod` packaging and the notice stamper belong to the retired format | `retired-hnmod-0.1.0/` |
| `tools/__pycache__/` | stale bytecode | deleted |
| `Cargo.lock` | written by `cargo test`; packed by the build and changes the artifact (imfast trap 5) | not kept; `/Cargo.lock` is gitignored |

**Kept on purpose:** `tools/e2e.sh` (the scenario and its assertions are format-independent;
the launch line is updated below), `tools/gen_content.py` (its `--out` default now points at
the package root and it no longer writes a manifest), `tools/craft_check.py`, `notes.md`,
`REPIN-AUDIT.md`, `VALIDATION.md` (history), `LICENSE`, `LICENSE-NOTICE.md` (rewritten for the
flat package).

### The wire: every retired module and injection, and where it went

The retired tree had **2 `[[module]]`s and 6 `[[injection]]`s** (`horizon-mixin.toml`). Each is
mapped below; none is deleted silently.

| Retired | SDK 0.2 home | Notes |
|---|---|---|
| `[[module]] runtime` → `runtime.rs` | `pub mod runtime;` in `src/lib.rs` | no plan section; the file stays byte-identical |
| `[[module]] state` → `support.rs` (`glassential_state`) | `src/support.rs`, called from `src/lib.rs` | the module is no longer injected into `crate::game`; it is an ordinary module of the crate |
| `[[injection]] embedded-content` (`content_root` HEAD, cancellable, AE2 pattern) | **deleted** | the flat `.hmod` carries `assets/` and `data/` itself, so there is nothing to write out or append to the dev list |
| `[[injection]] signal-source` (`is_signal_source` v1, HEAD, `signal_source.rs`) | `BlockPart::is_signal_source` on `support::Redstone`, attached to both redstone glasses | the Part is asked **only** for the blocks it is attached to, so the retired `REDSTONE.contains(name_of(game, state))` id test is gone |
| `[[injection]] block-signal` (`block_signal` v1, HEAD, `block_signal.rs`) | `BlockPart::block_signal` on `support::Redstone` | sets `event.signal = 15` |
| `[[injection]] dirty-block` (`block_changed` v1, HEAD, `changed.rs`) | `Hooks::block_changed` → `support::on_block_changed` | unscoped hook, so it stays a `Hooks` method; it now filters on the smart-glass id instead of marking every world change |
| `[[injection]] dirty-neighbor` (`neighbor_changed` v1, HEAD, `changed.rs`) | `BlockPart::neighbor_changed` on `support::Smart` | a scoped hook: no `Hooks` method exists for it |
| `[[injection]] smart-tick` (`level_tick_post` v1, HEAD, `level_tick.rs`) | `Hooks::level_tick_post` → `support::on_level_tick_post` | takes a `LevelRef` and a `HostSide`, not `(&mut Game, &str)` |

No hook is left without a home, and no retired helper was dropped except the embedded-content
one, whose whole job no longer exists.

### Content carry (proof it is mechanical)

The tree was moved, never copied back: `content/assets/` → `assets/`, `content/data/` → `data/`.
Proof, in this order:

- `pre-move-tree.sha256` (276 lines, written **before** the move) lists every old file; its 245
  `content/assets` + `content/data` lines, re-rooted, diff clean against the same 245 files
  taken out of the delivered package (`diff` → 0 lines, 245/245).
- The retired 0.1.0 content archive (`retired-hnmod-0.1.0/glassential.hmod`, 112,569 B,
  `69c150df…a9b4`, 248 entries) extracted and compared with the tree (`diff -rq`): **no
  differing byte, no file only on one side**. Its 248 entries are the 245 content files plus
  `horizon-mod.toml`, `LICENSE` and `NOTICE.md` (the old packer mixed the two notices into the
  content package; the flat package carries `LICENSE` and `LICENSE-NOTICE.md` at its root
  instead, which the SDK packer always includes).
- Classes carried: 63 block textures (+ `.mcmeta`), 24 blockstates, 24 block models (+ 4 lamp
  models, 1 custom parent), 24 item models, 24 26.3 item definitions, 1 lang file; 24 block
  declarations, 2 creative tabs, 24 loot tables, 24 recipes, 2 vanilla tags and 2 `c:` tags.

### Runtime carry (the campaign rule)

`src/runtime.rs` is the retired `mixins/glassential/runtime.rs`, copied before that file was
moved to evidence: `cmp` clean, SHA-256
`f0804f5fef920bf00a7cc96fad37028bc060465286add44d6aa1dcbd662933d7`, 396 lines. Its **9 tests ran
green before any SDK glue existed** (`src/lib.rs` was `pub mod runtime;` alone:
`evidence/glassential/sdk-unit-before-glue.log`), then again with the glue in place
(`sdk-test.log`), and `cargo fmt` did not touch it (`cmp` after `cargo fmt`,
`sdk-fmt-runtime-carry.log`). The packaged entry hashes to the same value
(`unzip -p glassential.hmod src/runtime.rs | sha256sum`).

The module is entirely `#[cfg(test)] mod tests` (the registry model and its assertions), so the
package runs no runtime code from it; `pub mod runtime;` keeps it the module the tests live in
and produces no `dead_code` warning.

### The glue, and what changed in it

| Rule (unchanged) | Where it lives now |
|---|---|
| the two redstone glasses are signal sources giving 15 on every face | `BlockPart::is_signal_source` / `block_signal` on `Redstone` |
| a smart-glass panel is the six-neighbour connected run, capped at 1024 | `support::component` |
| a panel is frosted when some cell has a neighbour signal | `level.best_neighbor_signal(host, pos) > 0` (the retired `native_neighbor_signal`) |
| the `powered` state is written only when it differs | `level.block_property` compare + `host.set_block_property` (the retired `native_block_property`/`native_set_block_property`) |
| at most 64 panels per level tick; the rest stay marked | `PANELS_PER_TICK`, re-marked from the drained cells |
| a changed cell dirties itself and its six neighbours | `Dirty::mark` |

### Build and check (verbatim, in a build slot)

`build-slot.sh` still names the retired tool's steps, so each command runs **directly in the
port** with `BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0` and no `--target-dir`; `--port` is a label
and the runner does not `cd`:

```sh
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job glassential-sdk --port ports/glassential --step test \
    -- bash -c 'cd ports/glassential && CARGO_INCREMENTAL=0 cargo test --offline'
# → test result: ok. 9 passed; 0 failed

BUILD_LOCAL=1 ports/tools/build-slot.sh run --job glassential-sdk --port ports/glassential --step lint \
    -- bash -c 'cd ports/glassential && CARGO_INCREMENTAL=0 cargo clippy --all-targets --offline -- -D warnings
        && cargo fmt --check'
# → Finished `dev` profile … (0 warnings); CLIPPY_EXIT:0 FMT_EXIT:0

BUILD_LOCAL=1 ports/tools/build-slot.sh run --job glassential-sdk --port ports/glassential --step build \
    -- bash -c 'cd ports/glassential && rm -f Cargo.lock && CARGO_INCREMENTAL=0 cargo horizon-mod build'
# → SDK 0b14f8a (build 0b14f8acfd97fa2c750e42795249dcc981178ca4) in ./.horizon/sdk
# → Built ./target/horizon-mod/glassential.hmod

BUILD_LOCAL=1 ports/tools/build-slot.sh run --job glassential-sdk --port ports/glassential --step check \
    -- bash -c 'cd ports/glassential && cargo horizon-mod check glassential.hmod
        && cargo horizon-mod fits glassential.hmod --build nightly'
# → glassential 0.2.0 (native-mod, format native-1/0b14f8ac…/e59da909…, for 26.3)
# → 125693 bytes, SHA-256 a479b244…7469, unsigned          CHECK_EXIT:0
# → Glassential Renewed: / 0b14f8a  made for               FITS_EXIT:0
```

### `uses.json` from the package

`unzip -p glassential.hmod uses.json` — 121 entries: 21 `hook:` (6 hooks and their 15 fields),
15 `method:`, 17 `handle:`, 51 `type:`, 13 `trait:`, 2 `macro:` (`horizon_mod`, `horizon_part`),
1 `part:block`, 1 `capability:registries`. The hooks are exactly the five the port uses:
`block_changed`, `block_signal`, `is_signal_source`, `level_tick_post`, `neighbor_changed`
(plus `blocks`, which `Registrar::block_key`/`block_part` reach through the registration
surface). The methods are the three `BlockPart` ones, `Hooks::{register, block_changed,
level_tick_post}`, `LevelRef::{block, block_property, dimension, in_place,
best_neighbor_signal}`, `LevelHost::state_block`, `KeysHost::block_key` and
`BlockStatesServerHost::set_block_property`.

### Behaviour that changed with the format (deliberate, and why)

- **A custom dimension now works.** The retired key was `(u8, [i32;3])` with `Dimension` matched
  on three variants; the SDK's `LevelDimension` has `Custom(LevelKey)`, so the key is
  `(LevelKey, [i32;3])` (`event.dimension.key()` in the hooks, `level.dimension().id(host)` +
  `LevelKey::parse` at the tick). A panel in a mod or data-pack level is rebuilt too.
- **`block_changed` filters before it marks.** 0.1.0 marked every changed cell in the world and
  dropped the non-smart ones at tick time; the SDK hook can ask `state_block` for free, so a
  fill, an explosion or another mod's excavation no longer fills the dirty set. The panel power
  side still comes from the pane's own `neighbor_changed`, which is vanilla's moment.
- **The read/write path is the host API**, not `Game` internals: `level.block(host, pos)`,
  `level.best_neighbor_signal(host, pos)`, `level.block_property(host, pos, "powered")` and
  `host.set_block_property(pos, "powered", want)`, with `event.level.in_place(host)` as the
  guard that the write lands in the level that just ticked.
- **The notices moved to the package root** (`LICENSE`, `LICENSE-NOTICE.md`) from inside the
  bundle, because that is what an SDK package packs.
- **No log line anywhere.** The retired `embedded_content.rs` logged a failed cache write; that
  helper is gone and the SDK exports no logger (no `log` crate for a mod). Nothing in the glue
  has a failure to report that a player could act on, so nothing is lost by it.

### Gaps after the migration

Everything in "Gaps and deviations" above still stands (per-entity collision, gravity glass,
the colour-able painter, clear-fluid face occlusion, the lava lamp's damage, the one-way
glasses, the 1024-block panel cap, one-tick-late panel updates, no tooltips, map colour, `/recipe
give`, mod tag ids in the command parser). Added by this format:

1. **No in-game re-run in this environment.** The e2e needs a game built by `horizon-launcher`
   with this package, and `horizon-launcher` is not installed here (the same trap `imfast`,
   `enderstorage` and `cloudglass` hit). The 0.1.0 runs remain the behaviour's evidence
   (`../_shared/evidence/glassential/e2e*.log`); what changed is the *transport* of the same
   rules, not the rules, which are byte-identical and tested.
2. **The panel state still updates at `level_tick_post`**, one tick after the signal changes,
   and a panel larger than 1024 blocks is split at the cap — unchanged from 0.1.0, now with a
   `level_tick_post` that must be the ticking level (`in_place`).
3. **`DEV_MOD=1` proves only the content half** on a source-built game: a native `.hmod`'s code
   is compiled in by the launcher, so the redstone and smart-glass phases need the launcher
   build.

### Traps (measured here; new to this port)

1. **`block_signal`, `is_signal_source` and `neighbor_changed` are scoped hooks with no `Hooks`
   method.** The generated trait has no `fn block_signal`; the only home is a `BlockPart`
   attached with `Registrar::block_part`. Attempting the `Hooks` route does not compile, so the
   retired per-hook helper files become Part methods.
2. **`Registrar::block_key` and `Registrar::block_part` both borrow the registrar mutably.**
   `r.block_part(r.block_key(id), part)` is a double `&mut`; name the key in a local first (the
   item-side variant of this is the shared trap list's #15).
3. **A Part must be `'static`, so it cannot borrow the session instance.** The dirty set is an
   `Arc<Dirty>` owned by `Mod` and cloned into `Smart`; a `static Mutex` (the enderstorage
   shape) is the alternative.
4. **`LevelDimension` is not `Hash`** (only `Clone/Copy/Debug/Eq/PartialEq`), and it and the
   `LevelRef`'s `DimensionKey` are different types. `LevelKey` is the one key both sides can
   produce (`event.dimension.key()` and `level.dimension().id(host)` + `LevelKey::parse`), and
   it is `Copy + Hash`, so the set needs no per-cell `String`.
5. **`&dyn LevelHost` cannot resolve a `BlockKey`** (no `block_key` on that trait), so
   `block_changed` tests the id with `level.state_block(state) == Some(SMART)`. A
   `Host`-level id test would need the mut host and does not exist in this hook.
6. **`set_block_property` takes no level.** Guard the pass with `event.level.in_place(&*host)`
   first (the laserio/xnet pattern), or a write could land in whatever level the host holds;
   cells stay marked and the next tick retries.
7. **Read by level handle, not by the host's own level reads**: `level.block(host, pos)`,
   `level.best_neighbor_signal(host, pos)`, `level.block_property(host, pos, key)`. The retired
   glue called `game.native_block(pos)` with a dimension string; the SDK's equivalent is the
   `LevelRef` method with the host as context.
8. **The dirty lock is never held across a game call.** The set is drained (one `retain`), then
   the game is read; a read can fire another mod's hook, and holding the mutex across it is how
   a lock-poisoning re-entry would start.
9. **`cargo horizon-mod build` kept `license`, `homepage` and `issues`** in the packed
   `horizon-mod.toml` — measured, contrary to the shared trap list's "build strips them". The
   packaged manifest is byte-for-byte the port's file (`unzip -p glassential.hmod
   horizon-mod.toml | cmp - horizon-mod.toml`), so that comparison, not the assumption, is the
   check.
10. **The 0.1.0 e2e's launch line was the retired shape.** `--dev-mod content/` no longer
    exists (`content/` is gone) and a native `.hmod` cannot load its code on an unmodded game,
    so `GAME=` is now required and `CONTROL_GAME=` optionally names a game without the package
    for the counterfactual leg; `SELFTEST=1` is the one-flag negative. `tools/e2e.sh` fails
    fast with a message when `GAME` is unset rather than launching a game that cannot answer.

### Changelog (0.2.0)

> **0.2.0 — republished as an SDK `.hmod`.** Horizon's mod-API merge (launcher ≥ 0.2.70)
> retired `.hnmod` and closed **#867** (`glassential` 0.1.0); this version supersedes it. One
> flat `glassential.hmod` now carries the code *and* the content — the 24 full-cube glasses
> with their blockstates, models, textures, 26.3 item definitions, silk-touch loot, 24 recipes,
> the `#minecraft:impermeable` / `#c:glass_blocks` tags, the two creative tabs and the lang —
> plus `src/lib.rs` (the one `impl mod_api::Hooks`), `src/support.rs` (the redstone answers and
> the smart-glass panel) and `src/runtime.rs`, which is byte-identical to 0.1.0. Built against
> SDK `0b14f8a` / mixins `e59da90` for 26.3. The `.hnmod`, the mixin plan, the five per-hook
> helpers and the embedded-content injection are gone; the two redstone glasses and the smart
> panel are now a `BlockPart` on their own blocks, the panel key also covers custom dimensions,
> and a block change is filtered before it is marked. Same feature set and the same gaps
> (per-entity collision, gravity glass, the painter, clear-fluid occlusion, the one-way glasses).

### Migration verification (SDK `0b14f8a`)

Skills: superpowers/using-superpowers, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

Evidence lives in `../_shared/evidence/glassential/` (`sdk-*.log`); each command ran in a
`build-slot.sh` lease with `BUILD_LOCAL=1` and `CARGO_INCREMENTAL=0`.

| Check | Command | Result |
|---|---|---|
| Runtime tests before glue | `cargo test --offline` with only `pub mod runtime;` | **9 passed**, 0 failed (`sdk-unit-before-glue.log`) |
| Runtime tests with glue | `cargo test --offline` | **9 passed**, 0 failed (`sdk-test.log`) |
| Lints | `cargo clippy --all-targets --offline -- -D warnings` | `CLIPPY_EXIT:0`, 0 warnings (`sdk-clippy.log`) |
| Formatting | `cargo fmt --check` (never `--all`: it formats the fetched SDK) | `FMT_CHECK_EXIT:0` (`sdk-fmt.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build` | `target/horizon-mod/glassential.hmod`, EXIT 0 (`sdk-build.log`) |
| Reproducible | second build, `cmp` | byte-identical, same SHA-256 (`sdk-build-second.log`, `sdk-sha256.log`) |
| Registry check | `cargo horizon-mod check glassential.hmod` | `glassential 0.2.0 (native-mod, format native-1/0b14f8ac…/e59da909…, for 26.3)`, **125,693 B**, SHA-256 `a479b244…7469`, **CHECK_EXIT:0** (`sdk-check.log`, `sdk-check-json.log`) |
| Fits the pinned build | `cargo horizon-mod fits glassential.hmod --build nightly` | `0b14f8a  made for`, **FITS_EXIT:0** (`sdk-fits.log`) |
| `runtime.rs` untouched | `unzip -p glassential.hmod src/runtime.rs \| sha256sum` | `f0804f5f…33d7` = the retired file, byte for byte (`sdk-runtime-carry.log`) |
| Content untouched | 245 `assets/`+`data/` files vs `pre-move-tree.sha256` and vs the retired 0.1.0 archive | same set, no differing byte (`sdk-content-carry.log`) |
| Entries | `unzip -l glassential.hmod` | 254 entries: 245 content + `horizon-mod.toml`, `Cargo.toml`, `src/lib.rs`, `src/runtime.rs`, `src/support.rs`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`, `uses.json`; **no `Cargo.lock`** (`sdk-entries.log`) |
| Uses | `unzip -p glassential.hmod uses.json` | 121 entries, 5 hooks (`sdk-uses.json`, `sdk-uses-summary.log`) |
| Skills line | `python3 ports/tools/check-skills.py --notes ports/glassential --strict` | `SKILLS GATE: PASS` |

### Not re-run in this migration

- **The in-game e2e.** No `horizon-launcher` and no game build at `0b14f8a` (the shared
  binaries are at `1d63e0a5`), so the last verified run is the retired bundle's:
  `e2e.log`, `e2e-counterfactual.log`, `e2e-selftest.log`, `e2e-repair.log` under
  `../_shared/evidence/glassential/`. `tools/e2e.sh` is updated for a launcher-built game
  (`GAME=`, optional `CONTROL_GAME=`, `SELFTEST=1`) and is the first job of the next slice.
