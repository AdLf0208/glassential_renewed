// Game-side glue for Glassential's server behaviour: the redstone signal
// answers and the smart-glass panel state.
//
// Injected into `crate::game` as `glassential_state`; it needs the game, so it
// is not part of the standalone library (only `runtime.rs` is).

use std::collections::HashSet;
use std::sync::Mutex;

use horizon_client::mod_api::BlockStateId;

/// The two upstream redstone glasses: a signal source returning 15.
const REDSTONE: [&str; 2] = [
    "glassential:glass_redstone",
    "glassential:glass_redstone_tinted",
];
/// The smart-glass block whose connected panel is frosted by a signal.
const SMART: &str = "glassential:smart_glass";

static DIRTY: Mutex<Option<HashSet<(u8, [i32; 3])>>> = Mutex::new(None);

fn dirty() -> std::sync::MutexGuard<'static, Option<HashSet<(u8, [i32; 3])>>> {
    DIRTY.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn dimension_key(dimension: horizon_world::level::Dimension) -> u8 {
    match dimension {
        horizon_world::level::Dimension::Overworld => 0,
        horizon_world::level::Dimension::Nether => 1,
        horizon_world::level::Dimension::End => 2,
    }
}

fn dimension_id(key: u8) -> &'static str {
    match key {
        1 => "minecraft:the_nether",
        2 => "minecraft:the_end",
        _ => "minecraft:overworld",
    }
}

fn name_of(game: &crate::game::GameLevel<'_>, state: BlockStateId) -> String {
    game.facts.get(state).name.to_string()
}

/// `BlockState.isSignalSource`: the two redstone glasses are sources.
///
/// `pub(super)`: the hook helpers live in `crate::game`, and `GameLevel` is
/// private to that module, so this is the widest visibility that can name it.
pub(super) fn is_signal_source(
    game: &crate::game::GameLevel<'_>,
    state: BlockStateId,
    source: &mut bool,
) {
    if REDSTONE.contains(&name_of(game, state).as_str()) {
        *source = true;
    }
}

/// `BlockState.getDirectSignal`/`getSignal`: 15 on every face.
pub(super) fn block_signal(
    game: &crate::game::GameLevel<'_>,
    state: BlockStateId,
    signal: &mut i32,
) {
    if REDSTONE.contains(&name_of(game, state).as_str()) {
        *signal = 15;
    }
}

fn mark(dimension: u8, pos: [i32; 3]) {
    let mut guard = dirty();
    let set = guard.get_or_insert_with(HashSet::new);
    set.insert((dimension, pos));
    for (dx, dy, dz) in [
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ] {
        set.insert((dimension, [pos[0] + dx, pos[1] + dy, pos[2] + dz]));
    }
}

/// A block changed or received a neighbour update: remember its cell and the
/// six neighbours, because a smart-glass panel there may need rebuilding.
pub(crate) fn on_changed(dimension: horizon_world::level::Dimension, pos: [i32; 3]) {
    mark(dimension_key(dimension), pos);
}

/// The connected smart-glass panel containing `start`, if `start` is one.
fn component(game: &crate::game::Game<'_>, start: [i32; 3]) -> Vec<[i32; 3]> {
    if game.native_block(start) != SMART {
        return Vec::new();
    }
    let mut seen = HashSet::from([start]);
    let mut stack = vec![start];
    while let Some(pos) = stack.pop() {
        for (dx, dy, dz) in [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            let next = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
            if seen.contains(&next) {
                continue;
            }
            if game.native_block(next) == SMART {
                seen.insert(next);
                stack.push(next);
            }
        }
    }
    seen.into_iter().collect()
}

/// Rebuild every dirty smart-glass panel for one level after its tick.
pub(super) fn on_level_tick_post(game: &mut crate::game::Game<'_>, dimension: &str) {
    let key = match dimension {
        "minecraft:the_nether" => 1,
        "minecraft:the_end" => 2,
        _ => 0,
    };
    let starts: Vec<[i32; 3]> = {
        let mut guard = dirty();
        let Some(set) = guard.as_mut() else {
            return;
        };
        let starts: Vec<[i32; 3]> = set
            .iter()
            .filter(|(dim, _)| *dim == key)
            .map(|(_, pos)| *pos)
            .collect();
        set.retain(|(dim, _)| *dim != key);
        starts
    };
    if starts.is_empty() {
        return;
    }
    let dim = dimension_id(key);
    let mut rebuilt = HashSet::new();
    for start in starts {
        if !rebuilt.insert(start) {
            continue;
        }
        let panel = component(game, start);
        if panel.is_empty() {
            continue;
        }
        for pos in &panel {
            rebuilt.insert(*pos);
        }
        let powered = panel
            .iter()
            .any(|pos| game.native_neighbor_signal(dim, *pos) > 0);
        let want = if powered { "true" } else { "false" };
        for pos in panel {
            if game.native_block_property(pos, "powered").as_deref() != Some(want) {
                game.native_set_block_property(pos, "powered", want);
            }
        }
    }
}
