# Glassential — re-pin audit (Horizon `83ad384f` / mixins `e8cd0aa`)

Read-only audit, 2026-11-05. No port code, manifests, content or builds were
changed; this file is the only write.

| | |
|---|---|
| New Horizon pin | `83ad384f48d0742746d518737511882680aab5b7` (427 hooks, 159 helpers), `ports/_shared/pins/horizon-83ad384f` |
| New mixins pin | `e8cd0aa38cffef49cb2e088ef716baa107148c02`, `ports/_shared/pins/horizon-mixins-e8cd0aa` |
| Port still pinned at | `c9d474dd…` / `d9c83d7…` (`compatibility.json`, `mixins/glassential/horizon-mixin.toml`) |
| Sources read | `ports/glassential/notes.md`, ARCHITECTURE item 30, `ports/_shared/upstream/glassential/FINDINGS.md`; pin `crates/horizon-client/src/game/native_hooks.rs`, `native_block_entity_draw.rs`, `native_motion.rs`, `native_entity_motion.rs`, `native_entity_data.rs`, `horizon-mods/src/content.rs`, `native-hooks.json` |
| Method | Each deferred feature searched in `native-hooks.json` and the pin's client/world sources; classified **usable** (writable out / cancel / helper), **hook-only** (listen-only), or **absent**. `listen` in the catalog was checked against the Rust signature: an event field taken by `&mut` is writable even when the catalog says `listen`. |

## Summary

| | |
|---|---:|
| Deferred feature groups audited | **11** |
| Realizable now at this pin | **6** |
| Partially realizable (one half blocked) | **4** |
| Still blocked | **1** |
| Highest-value next step | Add a native module and implement **redstone glass + smart glass** first (API 38 writable signal answers; API 96 neighbour signal; API 18 state write). Both are small, server-only and e2e-testable, and they prove the content-only port can carry behaviour. Then the block-entity renderer slice for clear-fluid and one-way glass. |

The repin itself is mechanical (heyberryshutup's `REPIN.md` recipe:
`compatibility.json`, manifest `[compatibility]`, `--lib` embedded-test target,
`-p horizon-client-lib --bin horizon-ctl`), but this port currently has **zero
injections**; every "realizable now" item below is a new native slice, not a
change to the content `.hmod`.

## Deferred feature audit

Legend: **class** = usable / hook-only / absent; **verdict** = realizable now /
partial / blocked. Line numbers are in the pin root above.

| # | Deferred feature (notes / ARCHITECTURE) | New-pin symbol(s) (file:line) | Class | Verdict | Exact next step |
|---|---|---|---|---|---|
| 1 | **Ethereal / ghostly / reverse per-entity collision** (`EtherealGlassBlock`, `GhostGlassBlock`): solid only to players, or empty to every entity | No collision override hook. `native_block_collision` (`native_hooks.rs:9050`) is a *read* helper; content `Block.collision_shapes` (`horizon-mods/src/content.rs:271`) is per-state, not per-entity. Catalog has only `block_collision` among collision names. | absent | **blocked** | File an API request for a per-entity `getCollisionShape`/`entityCollision` hook. Do not fake it with `collision_shapes` (it would change every entity). |
| 2 | **Gravity glass** — clamp downward velocity, scale fall distance, grant slow falling | `native_entity_set_velocity` (`native_entity_motion.rs:51`; **false for a player**), `native_entity_merge_nbt` (`native_entity_data.rs:32`; non-players), `native_entity_tick_pre/post` (`native_hooks.rs:7336`, `:7350`), `native_block_entity_inside` (`:7913`, static). No catalogued effect-give helper; only the pin-internal `add_player_effect` (`potions.rs:1056`). `native_push_player` (`native_motion.rs:31`) is **client-local only**. | usable for mobs; player half absent | **partial** | Implement the mob half with `block_entity_inside`/entity tick + `entity_set_velocity` + NBT `FallDistance`; record the player half as blocked (same `entity_set_velocity` refusal as ironjetpacks). |
| 3 | **Redstone glass** — signal source returning 15 on every face | `native_is_signal_source` (`native_hooks.rs:1529`, `source: &mut bool`), `native_block_signal` (`:1504`, `signal: &mut i32`), `native_update_signal_neighbors` (helper; catalog). Catalog contract says `listen`, but the generated dispatch writes the field back (verified in the generated `dispatch_is_signal_source`/`dispatch_block_signal`). | usable (writable out) | **realizable now** | Inject `is_signal_source` and `block_signal`; answer only for `glassential:glass_redstone`/`_tinted`. E2E: a redstone lamp/observer beside the glass reads 15, and a plain glass control reads 0. |
| 4 | **Smart glass** — `powered` frosts the whole connected panel (BFS, cap 1024) | `native_neighbor_signal` (`native_hooks.rs:8501`, `&mut self`), `native_set_block_property` (`:8634`, no `Ticks`), `native_block` (`:9041`), `native_neighbor_changed` (`:1314`) / `native_block_changed` (`:1200`) to dirty panels, `native_block_entity_tick` (`:3122`) for the mutable pass. | usable | **realizable now** | Give smart glass a block entity (`native_block_entity`, `:2985`; ticks `:2996`) or record dirty positions from `neighbor_changed` and process them in `server_tick` (`:2456`). BFS with `native_block`, write `powered` with `native_set_block_property`. E2E: signal on one panel block frosts the connected run; removing the signal clears it; a disconnected block stays clear. |
| 5 | **Colour-able glass + glass painter** — block entity colour/light/pass-through, `lit`, tint | `native_block_entity` (`:2985`), `native_block_entity_tick` (`:3122`), `native_block_entity_update_tag` (`:3141`), `native_block_entity_submit` (`:3190`), `native_draw_quad`/`native_draw_cube` (`native_block_entity_draw.rs:548`, `:620`), `native_use_item_on` (`:6482`). No dynamic per-block light emission helper and no per-entity collision hook. | usable for BE/painter/tint; light + pass-through absent | **partial** | Implement the painter + BE colour + tinted BE renderer first (the visible half). Keep the dynamic light level and pass-through collision as API requests; do not approximate with the two-state `lit` light. |
| 6 | **Clear-fluid glass** — six booleans from neighbours, fluid-face occlusion; `clear_fluid_fake_glass` is additionally non-solid to players | `native_neighbor_changed` (`:1314`), `native_set_block_property` (`:8634`), `native_block` (`:9041`), `native_block_entity` (`:2985`), `native_block_entity_tick` (`:3122`, mutable), `native_block_entity_update_tag` (`:3141`), `native_block_entity_submit` (`:3190`), `native_draw_cube` (`native_block_entity_draw.rs:620`); content `render_shape`/`occludes` (`content.rs:92`, `:95`). The block state already has the 64 states. The fake glass's player-only non-solidity has no per-entity collision hook (same gap as #1). | usable for states/occlusion; collision half absent | **partial** | Give each clear-fluid block a BE and set the six booleans + fluid mask in `block_entity_tick` (mutable), dirtying on `neighbor_changed`/`place_block`; sync the mask with `block_entity_update_tag`; draw with `render_shape: ENTITYBLOCK_ANIMATED` and hide faces in `block_entity_submit`. E2E: place against water/lava and assert the matching boolean(s), then a differential with no fluid. `clear_fluid_fake_glass` stays solid to players until the collision hook exists. |
| 7 | **Lava lamp** — step damage unless sneaking; water above becomes a bubble column | `native_block_step_on` (`:7902`, **static, no `Game`**), `native_server_tick` (`:2456`), `native_run_command` (`native_server.rs:479`), `native_server_players` (`:204`), `native_push_mob` (`native_mobs.rs:258`), `native_push_player` (`native_motion.rs:31`, client-local). No generic damage helper. | hook-only for step-on; usable for mob push | **partial** | Step damage: record entity ids in `block_step_on`, then in `server_tick` apply `/damage … minecraft:hot_floor` via `native_run_command` (maps player id→name with `native_server_players`), skipping sneaking. Bubble column: `native_push_mob` for mobs; players stay blocked (`entity_set_velocity` refuses them, `push_player` is local-only). |
| 8 | **One-way / tinted one-way glass** — `OPAQUE_FACE` state, mimic block entity, custom baked model `glassential:one_way_loader` | `native_standalone_models` (`:2955`, only JSON model ids — no custom loader), `native_block_entity` (`:2985`), `native_block_entity_submit` (`:3190`), `native_draw_cube` (`native_block_entity_draw.rs:620`, per-face `Option`), `native_camera` (`:9046`); content `StateProperty::Enum` (`content.rs:211`) and `render_shape` (`:92`). | usable; the exact loader is absent | **realizable now (different implementation)** | Declare the 6-value `opaque_face` state + `render_shape: ENTITYBLOCK_ANIMATED`; store the mimic state in a BE; draw the visible faces with `native_draw_cube`/`native_draw_block` and hide the viewer-facing side using `native_camera`. The upstream `one_way_loader` model cannot be loaded; the BE renderer replaces it. E2E needs a screenshot/render check or at least state + BE NBT. |
| 9 | **Panes, doors, trapdoors, slab** (later slices) | No hook needed. Content `Block.template` (`content.rs:268`), `BlockProperties` (`:73`), `collision_shapes` (`:271`); new optional `occludes`/`render_shape` (`:92`, `:95`). | n/a (content) | **realizable now** | Content slice: declare the pane/door/trapdoor/slab templates with the upstream states, models, loot and recipes. This was deferred scope, not an API block. |
| 10 | **Tooltips** (`tooltip.glassential.*`; obsidian/lamps) | `native_tooltip` (`:6413`, lines are writable), `native_tooltip_component` (`:6437`). Content 0.2 has no tooltip field, but the client hook does. | usable | **realizable now** | Add a client-side tooltip injection matching the block item ids; E2E can assert the line text through the client's tooltip path (or a unit test on the line builder). |
| 11 | **Mod tag ids in the command parser** (notes gap 10: `execute if block … #c:glass_blocks` said "Unknown block tag") | `block_input_matches` (`commands.rs:4040`) reads `[&self.res.items().block_tags, &self.items().block_tags]` (`:2529`); `self.items()` is the merged world/content table (`game.rs:19858`). | engine change (not a hook) | **realizable now / verify** | Re-run the existing e2e after the repin; if `#c:glass_blocks` resolves, delete the harness note. If it still fails, the remaining gap is parse-time tag-id registration, not the runtime table. |

Accepted, not counted: Fusion item icons (the cropped `_item.png` workaround;
there is still no Fusion pass in the item renderer) and map colour (the
template's value, as upstream).

## Requested-hook checks

- **`living_equipment_change` / `living_equipment_listened`** (API 148,
  `native_hooks.rs:7188`, `:7174`): present and writable (`listened` is a
  `&mut bool` collector), but **not used by this port**. Glass has no
  equipment-dependent rule; the per-entity collision is a shape hook, not an
  equipment hook.
- **`player_velocity` / `push_player`**: `native_player_velocity`
  (`native_hooks.rs:5307`) is a **listen-only notification** ("the server
  applied velocity"), not a way to set it. `native_push_player`
  (`native_motion.rs:31`) is **client-local only** (local player, clamped ±4),
  and `native_entity_set_velocity` (`native_entity_motion.rs:51`) **returns
  false for a player**. This is why gravity glass and the bubble column can
  move mobs but not players; there is no server-side player-push helper.
- **`raid_state_change`** (`native_hooks.rs:5909`): **hook-only** (listen,
  raid id/pos/state). No Glassential feature is raid-related; it unblocks
  nothing here.
- **`player_name_entity`** (`native_hooks.rs:5227`): **hook-only** (listen,
  player/entity id/name). No Glassential feature is naming-related; it
  unblocks nothing here.

## Pin state / repin prerequisites

The port is still on `c9d474d` / `d9c83d7`. Before any of the code slices
above:

1. `compatibility.json`: new `horizon_revision`, new `game.rs` hash
   (`996432b9…`), new mixins revision; `mixin_schema` stays 6.
2. `mixins/glassential/horizon-mixin.toml`: same 40-hex revision.
3. `tools/horizon.py`: embedded tests, if added, must use `--lib`; `horizon-ctl`
   builds from `-p horizon-client-lib`.
4. One-time `cargo fetch --locked` for `zune-jpeg` (network), then
   `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2` for every staged build.
5. Re-run the content `.hmod` build/check only if a content field changes; the
   24-block content is API 0.2 and still loads.

## Evidence

- `native-hooks.json` at the pin: **427 hooks / 159 helpers**; the four
  requested hooks and `is_signal_source`/`block_signal` are all v1.
- Writable-out proof: generated `dispatch_is_signal_source(... source: &mut bool)`
  writes `*source = event.source`; same for `dispatch_block_signal`.
- Forbidden paths verified read-only: nothing under `horizon/`,
  `horizon-mixins/`, `ports/_shared/pins/` was modified.
