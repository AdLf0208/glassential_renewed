//! Game-side glue for Glassential's server behaviour: the redstone signal answers and the
//! smart-glass panel state.
//!
//! The retired `.hnmod` ran this file as the injected `glassential_state` module and reached it
//! from five per-hook helpers; the SDK calls the typed methods in [`crate::Mod`] instead (the
//! mapping is in `notes.md`, "Migration to the SDK format"). [`crate::runtime`] is the carried,
//! pure model of the 24 glasses and their facts; this module is the only place that reads the
//! world and writes it back.
//!
//! The panel rule, unchanged from the retired glue: a smart-glass panel is the six-neighbour
//! connected component of `glassential:smart_glass` around a changed cell, capped at
//! [`MAX_PANEL`] blocks, and it is frosted (`powered=true`, which selects the model) exactly
//! when some cell of it has a neighbour signal. A rebuild happens at `level_tick_post`, at most
//! [`PANELS_PER_TICK`] panels per level per tick.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};

use mod_api::{
    BlockChangedEvent, BlockKey, BlockPart, BlockSignalEvent, BlockState, Host,
    IsSignalSourceEvent, LevelHost, LevelKey, LevelRef, LevelTicksHost, NeighborChangedEvent,
    Registrar, ServerHost,
};

/// The two upstream redstone glasses: `BlockState.isSignalSource` and 15 on every face
/// (`RedstoneGlassBlock.java:20-30` upstream).
const REDSTONE: [&str; 2] = [
    "glassential:glass_redstone",
    "glassential:glass_redstone_tinted",
];

/// The smart-glass block whose connected panel is frosted by a signal.
const SMART: &str = "glassential:smart_glass";

/// The state property the frosted model is selected by.
const POWERED: &str = "powered";

/// The six face neighbours, in the retired glue's order.
const NEIGHBOURS: [[i32; 3]; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

/// Upstream's smart-glass limit: a panel is at most 1024 blocks, and a larger connected mass is
/// split where the walk reaches the cap (`Config.smartMaxPanel`). It keeps a rebuild bounded:
/// without it a huge connected component walked and rewrote every member in one server tick.
const MAX_PANEL: usize = 1024;

/// How many panels one level tick rebuilds; the rest stay marked for the next tick, so a world
/// with many changed panels spreads the work instead of stalling one tick.
const PANELS_PER_TICK: usize = 64;

/// The cells a rebuild still owes, keyed by the level's own dimension id and a block position.
///
/// One set for the session, shared with the smart-glass Part (see [`crate::Mod`]); the lock is
/// dropped before any game call, so a read that fires another hook can re-enter it.
#[derive(Default)]
pub struct Dirty {
    cells: Mutex<HashSet<(LevelKey, [i32; 3])>>,
}

impl Dirty {
    /// The cell and its six neighbours: a panel standing here may have changed shape or power.
    fn mark(&self, dimension: LevelKey, pos: [i32; 3]) {
        let mut guard = self.cells();
        mark_in(&mut guard, dimension, pos);
    }

    /// This dimension's cells, removed from the set so the next pass starts empty.
    fn take(&self, dimension: LevelKey) -> Vec<[i32; 3]> {
        let mut guard = self.cells();
        let mut mine = Vec::new();
        guard.retain(|(dim, pos)| {
            if *dim == dimension {
                mine.push(*pos);
                false
            } else {
                true
            }
        });
        mine
    }

    fn cells(&self) -> MutexGuard<'_, HashSet<(LevelKey, [i32; 3])>> {
        self.cells
            .lock()
            .expect("the dirty-cell lock is held for one insert or drain")
    }
}

fn mark_in(cells: &mut HashSet<(LevelKey, [i32; 3])>, dimension: LevelKey, pos: [i32; 3]) {
    cells.insert((dimension, pos));
    for [dx, dy, dz] in NEIGHBOURS {
        cells.insert((dimension, [pos[0] + dx, pos[1] + dy, pos[2] + dz]));
    }
}

// ---------------------------------------------------------------------------
// The two block Parts
// ---------------------------------------------------------------------------

/// The two redstone glasses: a signal source returning 15 on every face.
///
/// The Part is attached only to those two blocks, so it answers without testing an id — and
/// therefore without a registry read on a hook the level asks very often.
pub struct Redstone;

#[mod_api::horizon_part]
impl BlockPart for Redstone {
    /// `BlockState.getDirectSignal`/`getSignal`: 15 on every face.
    fn block_signal(&self, _level: &dyn LevelHost, event: &mut BlockSignalEvent) {
        event.signal = 15;
    }

    /// `BlockState.isSignalSource`: so redstone wire beside the pane connects to it.
    fn is_signal_source(&self, _level: &dyn LevelHost, event: &mut IsSignalSourceEvent) {
        event.source = true;
    }
}

/// Smart glass: every neighbour update may change what powers its panel.
pub struct Smart {
    dirty: Arc<Dirty>,
}

#[mod_api::horizon_part]
impl BlockPart for Smart {
    /// The vanilla neighbour update the upstream block answers (`SmartGlassBlock.neighborChanged`):
    /// remember the cell for the panel rebuild after the level's tick.
    fn neighbor_changed(&self, _level: &mut dyn LevelTicksHost, event: &mut NeighborChangedEvent) {
        self.dirty.mark(
            event.dimension.key(),
            [event.pos.x, event.pos.y, event.pos.z],
        );
    }
}

/// Attach the Parts to the three blocks the server half answers for.
pub fn declare(r: &mut dyn Registrar, dirty: &Arc<Dirty>) {
    for id in REDSTONE {
        // The key is named first: `block_part` also borrows the registrar mutably.
        let block = r.block_key(id);
        r.block_part(block, Redstone);
    }
    let smart = r.block_key(SMART);
    r.block_part(
        smart,
        Smart {
            dirty: Arc::clone(dirty),
        },
    );
}

// ---------------------------------------------------------------------------
// The panel rule
// ---------------------------------------------------------------------------

/// A smart pane was placed, replaced or removed: its panel gained or lost a cell, so mark the
/// cell and its neighbours. A write that leaves the smart glass alone cannot change a panel's
/// membership, and the power side is the Part's `neighbor_changed`, so everything else is
/// ignored.
pub fn on_block_changed(dirty: &Dirty, level: &dyn LevelHost, event: &mut BlockChangedEvent) {
    if event.old == event.new {
        return;
    }
    if !is_smart(level, event.old) && !is_smart(level, event.new) {
        return;
    }
    dirty.mark(
        event.dimension.key(),
        [event.pos.x, event.pos.y, event.pos.z],
    );
}

fn is_smart(level: &dyn LevelHost, state: BlockState) -> bool {
    level.state_block(state) == Some(SMART)
}

/// Rebuild every marked smart-glass panel of `level`, after its tick pass.
pub fn on_level_tick_post(dirty: &Dirty, host: &mut dyn ServerHost, level: LevelRef) {
    // The write belongs to the level the server ticks now: outside that window the cells stay
    // marked and the next pass retries rather than writing into another level.
    if !level.in_place(&*host) {
        return;
    }
    let Some(smart) = host.block_key(SMART) else {
        return;
    };
    let Some(dimension) = level
        .dimension()
        .id(&*host)
        .and_then(|id| LevelKey::parse(&id).ok())
    else {
        return;
    };
    let mut starts = dirty.take(dimension);
    if starts.is_empty() {
        return;
    }
    // Only a smart-glass cell can start a panel. A `fill` marks every cell it touches, so the
    // others are dropped here instead of spending the per-tick budget on lookups that can never
    // rebuild; the rest beyond the budget stay marked for the next tick.
    starts.sort_unstable();
    starts.dedup();
    starts.retain(|pos| level.block(&*host, *pos) == Some(smart));
    if starts.len() > PANELS_PER_TICK {
        for pos in &starts[PANELS_PER_TICK..] {
            mark_in(&mut dirty.cells(), dimension, *pos);
        }
        starts.truncate(PANELS_PER_TICK);
    }
    let mut rebuilt: HashSet<[i32; 3]> = HashSet::new();
    for start in starts {
        if !rebuilt.insert(start) {
            continue;
        }
        // A start was filtered to a smart pane above, so its panel always holds it.
        let panel = component(&*host, level, smart, start);
        for pos in &panel {
            rebuilt.insert(*pos);
        }
        let powered = panel.iter().any(|pos| {
            level
                .best_neighbor_signal(&*host, *pos)
                .is_some_and(|signal| signal > 0)
        });
        let want = if powered { "true" } else { "false" };
        for pos in panel {
            if level.block_property(&*host, pos, POWERED).as_deref() == Some(want) {
                continue;
            }
            host.set_block_property(pos, POWERED, want);
        }
    }
}

/// The connected smart-glass panel containing `start`, at most [`MAX_PANEL`] blocks.
fn component(host: &dyn Host, level: LevelRef, smart: BlockKey, start: [i32; 3]) -> Vec<[i32; 3]> {
    let mut seen = HashSet::from([start]);
    let mut stack = vec![start];
    while let Some(pos) = stack.pop() {
        if seen.len() >= MAX_PANEL {
            break;
        }
        for [dx, dy, dz] in NEIGHBOURS {
            if seen.len() >= MAX_PANEL {
                break;
            }
            let next = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
            if seen.contains(&next) {
                continue;
            }
            if level.block(host, next) == Some(smart) {
                seen.insert(next);
                stack.push(next);
            }
        }
    }
    seen.into_iter().collect()
}
