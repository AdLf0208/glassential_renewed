// Glassential's smart-glass panel rule, as pure logic (no game access).
//
// A panel is the six-neighbour connected component of `glassential:smart_glass` around a changed
// cell. The server half used to rebuild a panel as *truncated* 1024-cell subsets taken in walk
// order, so two starts on the same oversized mass produced different subsets; when their cells
// saw different neighbour signals they wrote opposite `powered` values to shared cells, and each
// write marked the glass dirty again. The rules here are what the glue uses instead:
//
//  * a panel's membership is the whole component, canonically ordered — the same component
//    yields the same membership whatever cell the rebuild started from;
//  * a panel is frosted when **any** of its cells sees a signal, so one rebuild cannot write two
//    different values into one component;
//  * only a change that adds or removes a smart cell can change membership — a state-only
//    change (the glue's own `powered` write) must not re-dirty the panel;
//  * writes are handed out in budgeted batches, so one tick's work is bounded.

/// Upstream's `Config.smartMaxPanel`: the batch size a panel's writes are handed out in.
pub const MAX_PANEL: usize = 1024;

/// The largest component the glue walks. A component beyond this is left untouched rather than
/// written as a start-dependent subset; 4096 cells is four full batches.
pub const MAX_SCAN: usize = MAX_PANEL * 4;

/// Panels one level tick starts rebuilding; the rest stay marked for the next tick.
pub const PANELS_PER_TICK: usize = 64;

/// Block writes one level tick may perform across all panels. A panel larger than this is
/// finished over the following ticks; every batch carries the same value, so a partly written
/// panel is never inconsistent.
pub const CELLS_PER_TICK: usize = MAX_PANEL * 8;

/// Whether a block change can change a panel's membership: only a change that adds or removes a
/// smart-glass cell. A state-only change — `powered` flipped by our own write — cannot.
pub fn membership_changed(old_is_smart: bool, new_is_smart: bool) -> bool {
    old_is_smart != new_is_smart
}

/// A panel is frosted when at least one of its cells sees a live neighbour signal. It is a
/// function of the whole component, so every batch of one panel computes the same value.
pub fn panel_powered(signals: &[i32]) -> bool {
    signals.iter().any(|signal| *signal > 0)
}

/// The panel's cells in a canonical order, duplicates removed: membership and write order do not
/// depend on the walk that found them.
pub fn canonical(cells: &[[i32; 3]]) -> Vec<[i32; 3]> {
    let mut cells = cells.to_vec();
    cells.sort_unstable();
    cells.dedup();
    cells
}

/// The batches one panel's writes are split into, at most `budget` cells each. The ranges
/// partition `0..len` in order and are disjoint, so each cell is written once per rebuild.
pub fn batches(len: usize, budget: usize) -> Vec<std::ops::Range<usize>> {
    let budget = budget.max(1);
    (0..len)
        .step_by(budget)
        .map(|start| start..(start + budget).min(len))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_state_only_change_does_not_rebuild_the_panel() {
        // Our own `powered` write is old=smart, new=smart: membership is unchanged.
        assert!(!membership_changed(true, true));
        assert!(!membership_changed(false, false));
        // A placed or removed smart pane does change it.
        assert!(membership_changed(true, false));
        assert!(membership_changed(false, true));
    }

    #[test]
    fn a_panel_is_frosted_by_any_live_cell() {
        assert!(!panel_powered(&[]), "an empty panel has no signal");
        assert!(!panel_powered(&[0, 0, 0]));
        assert!(panel_powered(&[0, 15, 0]));
        assert!(panel_powered(&[1, 0]), "a weak signal frosts it too");
    }

    #[test]
    fn the_write_budget_partitions_a_panel_stably() {
        assert_eq!(batches(0, 1024), Vec::<std::ops::Range<usize>>::new());
        assert_eq!(batches(10, 1024), vec![0..10]);
        assert_eq!(batches(2048, 1024), vec![0..1024, 1024..2048]);
        let split = batches(2500, 1024);
        assert_eq!(split, vec![0..1024, 1024..2048, 2048..2500]);
        // Every index is covered exactly once, in order.
        let mut covered: Vec<usize> = split.iter().flat_map(|r| r.clone()).collect();
        let expected: Vec<usize> = (0..2500).collect();
        assert_eq!(covered, expected);
        covered.clear();
        // A zero budget still makes progress rather than looping forever.
        assert_eq!(batches(3, 0), vec![0..1, 1..2, 2..3]);
    }

    #[test]
    fn the_same_component_has_one_membership_whatever_the_start() {
        let from_one_start = [[2, 0, 0], [0, 0, 0], [1, 0, 0], [0, 0, 0]];
        let from_another = [[0, 0, 0], [1, 0, 0], [2, 0, 0]];
        assert_eq!(canonical(&from_one_start), canonical(&from_another));
        assert_eq!(
            canonical(&from_one_start),
            vec![[0, 0, 0], [1, 0, 0], [2, 0, 0]],
            "membership is a sorted set, not a walk order"
        );
    }

    #[test]
    fn the_scan_and_batch_bounds_agree() {
        assert_eq!(MAX_SCAN, 4 * MAX_PANEL);
        const { assert!(CELLS_PER_TICK >= MAX_PANEL) };
        assert!(
            batches(MAX_SCAN, MAX_PANEL).len() <= MAX_SCAN / MAX_PANEL,
            "a panel's batches stay within its cells"
        );
    }
}
