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
//! connected component of `glassential:smart_glass` around a changed cell. Its membership is the
//! whole component, canonically ordered in [`crate::panel`], and it is frosted (`powered=true`,
//! which selects the model) exactly when some cell of it has a neighbour signal — one value for
//! the whole component, so batches of one panel can never disagree. A rebuild happens at
//! `level_tick_post`, at most [`panel::CELLS_PER_TICK`] writes per level per tick, and the glue
//! ignores its own `powered` writes when it decides whether membership changed.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};

use mod_api::{
    BlockChangedEvent, BlockKey, BlockPart, BlockSignalEvent, BlockState, Direction, Host,
    IsSignalSourceEvent, LevelHost, LevelKey, LevelRef, LevelTicksHost, NeighborChangedEvent,
    Registrar, ServerHost, StateForPlacementEvent, UpdateShapeEvent,
};

use crate::connect::{Face, Links};
use crate::panel;

/// The two upstream redstone glasses: `BlockState.isSignalSource` and 15 on every face
/// (`RedstoneGlassBlock.java:20-30` upstream).
const REDSTONE: [&str; 2] = [
    "glassential:glass_redstone",
    "glassential:glass_redstone_tinted",
];

/// The smart-glass block whose connected panel is frosted by a signal.
const SMART: &str = "glassential:smart_glass";

/// The two clear-fluid glasses whose six face booleans follow their own kind.
const CLEAR_FLUID: [&str; 2] = [
    "glassential:clear_fluid_glass",
    "glassential:clear_fluid_fake_glass",
];

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

/// A clear-fluid glass: the six face booleans follow the block beside each face.
///
/// Upstream derives them in two moments, and so does this Part: `getStateForPlacement` picks the
/// state the block is placed with ([`BlockPart::state_for_placement`]), and `updateShape` answers
/// the state it takes when a neighbour changes ([`BlockPart::update_shape`]). A face links only to
/// the *same* block, so `clear_fluid_glass` and `clear_fluid_fake_glass` never link to each other;
/// the Part is attached per block and carries that block's own key, so the test costs one
/// comparison and no registry lookup.
pub struct Connected {
    block: BlockKey,
}

#[mod_api::horizon_part]
impl BlockPart for Connected {
    /// The state the block is placed with: the six faces, as its neighbours are.
    ///
    /// `None` leaves vanilla's answer, which is the block's default state — every face false —
    /// and that is also what this rule wants when nothing beside the block is the same block, so
    /// an isolated pane costs no write.
    fn state_for_placement(
        &self,
        level: &dyn LevelHost,
        event: &mut StateForPlacementEvent<'_>,
    ) -> Option<Option<BlockState>> {
        let pos = event.context.pos;
        let links = self.links(level, [pos.x, pos.y, pos.z], None);
        apply(level, event.state, links).map(Some)
    }

    /// The state the block takes now that its neighbour in `event.direction` changed.
    ///
    /// `None` keeps the state as it is, so a neighbour that cannot change any face costs nothing.
    fn update_shape(
        &self,
        level: &mut dyn LevelTicksHost,
        event: &mut UpdateShapeEvent,
    ) -> Option<BlockState> {
        let pos = [event.pos.x, event.pos.y, event.pos.z];
        let links = self.links(
            level,
            pos,
            Some((face_of(event.direction), event.neighbour)),
        );
        apply(level, event.state, links)
    }
}

impl Connected {
    /// The six faces of the block at `pos`, read from the block beside each one.
    ///
    /// `changed` is the neighbour a shape update is about: its new state comes from the event
    /// rather than from the level, which may still hold the one it replaced.
    fn links(
        &self,
        level: &dyn LevelHost,
        pos: [i32; 3],
        changed: Option<(Face, BlockState)>,
    ) -> Links {
        Links::from_connected(|face| {
            let beside = match changed {
                Some((changed_face, state)) if changed_face == face => Some(state),
                _ => {
                    let [dx, dy, dz] = face.offset();
                    level.block_state([pos[0] + dx, pos[1] + dy, pos[2] + dz])
                }
            };
            beside.is_some_and(|state| state.block() == self.block)
        })
    }
}

/// The face a direction names.
fn face_of(direction: Direction) -> Face {
    match direction {
        Direction::Down => Face::Down,
        Direction::Up => Face::Up,
        Direction::North => Face::North,
        Direction::South => Face::South,
        Direction::West => Face::West,
        Direction::East => Face::East,
    }
}

/// `state` with its six face properties set as `links` says, or `None` when that changes nothing.
///
/// A property the declaration is missing makes the whole answer `None` rather than a half-written
/// state: the caller then keeps vanilla's state (the default, every face false), and
/// `connect`'s declaration test is what keeps the two in step.
fn apply(level: &dyn LevelHost, state: BlockState, links: Links) -> Option<BlockState> {
    let mut answered = state;
    for face in Face::ALL {
        let value = if links.is_connected(face) {
            "true"
        } else {
            "false"
        };
        answered = level.state_with_property(answered, face.property(), value)?;
    }
    (answered != state).then_some(answered)
}

/// Attach the Parts to the blocks the server half answers for.
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
    for id in CLEAR_FLUID {
        let block = r.block_key(id);
        r.block_part(block, Connected { block });
    }
}

// ---------------------------------------------------------------------------
// The panel rule
// ---------------------------------------------------------------------------

/// A smart pane was placed, replaced or removed: its panel gained or lost a cell, so mark the
/// cell and its neighbours. A state-only change (the glue's own `powered` write, or a mod
/// re-setting another smart-glass property) cannot change membership and is ignored — that is
/// what used to make each panel write dirty the panel again. The power side is the Part's
/// `neighbor_changed`, so everything else is ignored too.
pub fn on_block_changed(dirty: &Dirty, level: &dyn LevelHost, event: &mut BlockChangedEvent) {
    if event.old == event.new {
        return;
    }
    if !panel::membership_changed(is_smart(level, event.old), is_smart(level, event.new)) {
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
    if starts.len() > panel::PANELS_PER_TICK {
        for pos in &starts[panel::PANELS_PER_TICK..] {
            mark_in(&mut dirty.cells(), dimension, *pos);
        }
        starts.truncate(panel::PANELS_PER_TICK);
    }
    // One budget for the whole level tick. It is spent on writes only: a cell that already has
    // the wanted value costs nothing, so a large panel always makes progress and is finished by
    // the following ticks, each writing the same value.
    let mut budget = panel::CELLS_PER_TICK;
    let mut rebuilt: HashSet<[i32; 3]> = HashSet::new();
    for start in starts {
        if !rebuilt.insert(start) {
            continue;
        }
        // A start was filtered to a smart pane above, so its panel always holds it. A component
        // beyond the scan bound is left alone: a partial, start-dependent subset must never be
        // written (that is the defect this rule fixes).
        let Some(component) = component(&*host, level, smart, start) else {
            continue;
        };
        let cells = panel::canonical(&component);
        let mut signals = Vec::with_capacity(cells.len());
        for pos in &cells {
            signals.push(level.best_neighbor_signal(&*host, *pos).unwrap_or(0));
        }
        let want = if panel::panel_powered(&signals) {
            "true"
        } else {
            "false"
        };
        for batch in panel::batches(cells.len(), panel::MAX_PANEL) {
            for pos in &cells[batch] {
                rebuilt.insert(*pos);
                if budget == 0 {
                    // Out of this tick's writes: keep the cell marked so a later tick finishes
                    // the same panel with the same value.
                    mark_in(&mut dirty.cells(), dimension, *pos);
                    continue;
                }
                if level.block_property(&*host, *pos, POWERED).as_deref() == Some(want) {
                    continue;
                }
                if host.set_block_property(*pos, POWERED, want) {
                    budget -= 1;
                }
            }
        }
    }
}

/// The connected smart-glass panel containing `start`, at most [`panel::MAX_SCAN`] cells.
///
/// `None` means the component is larger than the scan bound: the caller leaves it untouched
/// rather than writing a subset picked by the walk order.
fn component(
    host: &dyn Host,
    level: LevelRef,
    smart: BlockKey,
    start: [i32; 3],
) -> Option<Vec<[i32; 3]>> {
    let mut seen = HashSet::from([start]);
    let mut stack = vec![start];
    while let Some(pos) = stack.pop() {
        for [dx, dy, dz] in NEIGHBOURS {
            let next = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
            if seen.contains(&next) {
                continue;
            }
            if level.block(host, next) != Some(smart) {
                continue;
            }
            seen.insert(next);
            if seen.len() > panel::MAX_SCAN {
                return None;
            }
            stack.push(next);
        }
    }
    Some(seen.into_iter().collect())
}
