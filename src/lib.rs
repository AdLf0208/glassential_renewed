//! Glassential Renewed for Horizon — the one `impl Hooks` at the crate root.
//!
//! The 24 full-cube glasses are **content**: their declarations, blockstates, models,
//! textures, item definitions, loot, recipes, tags, tabs and lang live in `data/` and
//! `assets/` inside this same `.hmod`, exactly as the retired content package carried them.
//!
//! [`runtime`] is the port's pure registry model, byte-identical to the retired bundle's
//! `runtime.rs` and exercised by `cargo test` without the game. The code this package adds on
//! top of the content is the server half the mod's own behaviour needs, in [`support`]:
//!
//! | Moment | Where | What it does |
//! |---|---|---|
//! | `is_signal_source`, `block_signal` | [`support::Redstone`] Part | the two redstone glasses answer as sources and give 15 on every face |
//! | `neighbor_changed` | [`support::Smart`] Part | a smart pane took a neighbour update, so its panel may need rebuilding |
//! | `state_for_placement`, `update_shape` | [`support::Connected`] Part | a clear-fluid glass takes the six face booleans of its own kind, at placement and on every neighbour change |
//! | [`Hooks::block_changed`] | this module | a smart pane was placed or removed |
//! | [`Hooks::level_tick_post`] | this module | rebuild every marked smart-glass panel of the level that just ticked |

use std::sync::Arc;

use mod_api::{
    BlockChangedEvent, HostSide, LevelHost, LevelTickPostEvent, Registrar, RegistrationContext,
};

pub mod connect;
pub mod panel;
pub mod runtime;
mod support;

use support::Dirty;

/// The mod instance the generated `hooks()` boxes once for the session.
///
/// The dirty cells sit behind an `Arc` because the smart-glass `BlockPart` outlives
/// `register` and cannot borrow the session instance, so it holds its own handle to the same
/// set instead of a second copy. `Hooks` is `Send + Sync`, and [`Dirty`] is a `Mutex`, so the
/// state is shared safely across the window and server threads.
#[derive(Default)]
struct Mod {
    dirty: Arc<Dirty>,
}

#[mod_api::horizon_mod]
impl mod_api::Hooks for Mod {
    /// Attach the two block Parts: the redstone glasses answer redstone, and smart glass is
    /// told about its neighbour updates.
    fn register(&self, r: &mut dyn Registrar, _cx: &RegistrationContext) {
        support::declare(r, &self.dirty);
    }

    /// A smart pane was placed, replaced or removed: its panel changed shape, so rebuild the
    /// cells around it after the level's next tick pass. Every other block's write is ignored,
    /// which is what keeps a mass change (a `fill`, an excavation) from marking thousands of
    /// cells the panel rule can never use.
    fn block_changed(&self, level: &dyn LevelHost, event: &mut BlockChangedEvent) {
        support::on_block_changed(&self.dirty, level, event);
    }

    /// Rebuild this level's marked smart-glass panels, in the level that has just ticked.
    fn level_tick_post(&self, host: HostSide<'_>, event: &mut LevelTickPostEvent) {
        let HostSide::Server(host) = host else {
            return;
        };
        support::on_level_tick_post(&self.dirty, host, event.level);
    }
}
