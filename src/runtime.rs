// Glassential Renewed for Horizon — the slice-1 glass registry (pure).
//
// Ported from Glassential Renewed 3.4.7
// (`bigenergy/glassential-renewed`, branch `1.21.1-neoforge`, commit
// `0387cd173788d5348efdfaf80ffb129ad8ea56d2`, MIT) for Minecraft Java 26.3 on
// Horizon. Upstream was read for behaviour and data only; this is a
// clean-room rewrite. Slice 1 is content-only: the blocks are declared in
// `../content`, and this model is the single place the ids, templates, light
// levels, state counts, tinted set and creative-tab split are spelled out.

#[cfg(test)]
mod tests {
    /// Which creative tab upstream puts a block in
    /// (`Glassential.java:47-66`: `ITEMS_FOR_TAB_LIST_FUNC` is the
    /// functional tab, `ITEMS_FOR_TAB_LIST` the blocks tab).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Tab {
        /// `glassential:glassential_tab`, "Glassential Renewed - Functional".
        Functional,
        /// `glassential:glassential_tab_misc`, "Glassential Renewed - Blocks".
        Blocks,
    }

    /// One full-cube glass and the numbers its content declaration uses.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Glass {
        /// `<mod id>:<path>` (`GlassentialBlocks.java:171-240`).
        pub id: &'static str,
        /// The vanilla block whose behaviour, states and shapes it copies.
        pub template: &'static str,
        /// Light emitted, 0 to 15.
        pub light: u8,
        /// The block's state count in 26.3: the template's states times the
        /// declared ones (`1` unless the block declares properties).
        pub states: u32,
        /// A `GLASSES_TINTED` member (`c:glass_blocks/tinted`).
        pub tinted: bool,
        /// Upstream `dropWhenSilkTouch` (`GlassentialBlockLoot.java:19-31`).
        pub silk_touch_only: bool,
        /// Upstream's creative tab.
        pub tab: Tab,
    }

    /// The slice-1 glasses, in upstream registration order.
    pub const GLASSES: [Glass; 24] = [
        Glass {
            id: "glassential:glass_dark_ethereal",
            template: "minecraft:tinted_glass",
            light: 0,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:glass_dark_ethereal_reverse",
            template: "minecraft:tinted_glass",
            light: 0,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:glass_ethereal",
            template: "minecraft:glass",
            light: 0,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:glass_ethereal_reverse",
            template: "minecraft:glass",
            light: 0,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:glass_ghostly",
            template: "minecraft:glass",
            light: 0,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:colorable_glass",
            template: "minecraft:glass",
            light: 0,
            states: 2,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:colorable_stained_glass",
            template: "minecraft:white_stained_glass",
            light: 0,
            states: 2,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:glass_light",
            template: "minecraft:glass",
            light: 15,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:glass_light_tinted",
            template: "minecraft:tinted_glass",
            light: 15,
            states: 1,
            tinted: true,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:glass_redstone",
            template: "minecraft:glass",
            light: 0,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:glass_redstone_tinted",
            template: "minecraft:tinted_glass",
            light: 0,
            states: 1,
            tinted: true,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:smart_glass",
            template: "minecraft:glass",
            light: 0,
            states: 2,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:obsidian_glass",
            template: "minecraft:glass",
            light: 0,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:stone_glass",
            template: "minecraft:glass",
            light: 0,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Blocks,
        },
        Glass {
            id: "glassential:sandstone_glass",
            template: "minecraft:glass",
            light: 0,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Blocks,
        },
        Glass {
            id: "glassential:ice_glass",
            template: "minecraft:glass",
            light: 0,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Blocks,
        },
        Glass {
            id: "glassential:iron_glass",
            template: "minecraft:glass",
            light: 0,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Blocks,
        },
        Glass {
            id: "glassential:gravity_glass",
            template: "minecraft:glass",
            light: 0,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:clear_fluid_glass",
            template: "minecraft:glass",
            light: 0,
            states: 64,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:clear_fluid_fake_glass",
            template: "minecraft:glass",
            light: 0,
            states: 64,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Functional,
        },
        Glass {
            id: "glassential:glass_lava_lamp",
            template: "minecraft:glass",
            light: 15,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Blocks,
        },
        Glass {
            id: "glassential:glass_lava_lamp_tinted",
            template: "minecraft:tinted_glass",
            light: 15,
            states: 1,
            tinted: true,
            silk_touch_only: true,
            tab: Tab::Blocks,
        },
        Glass {
            id: "glassential:glass_glowstone_lamp",
            template: "minecraft:glass",
            light: 15,
            states: 1,
            tinted: false,
            silk_touch_only: true,
            tab: Tab::Blocks,
        },
        Glass {
            id: "glassential:glass_glowstone_lamp_tinted",
            template: "minecraft:tinted_glass",
            light: 15,
            states: 1,
            tinted: true,
            silk_touch_only: true,
            tab: Tab::Blocks,
        },
    ];

    /// One recipe per glass (`data/glassential/recipe/<id>.json`).
    pub const RECIPES: usize = 24;

    #[test]
    fn the_slice_ships_twenty_four_glasses() {
        assert_eq!(GLASSES.len(), 24);
    }

    #[test]
    fn every_id_is_namespaced_and_unique() {
        let mut ids: Vec<&str> = GLASSES.iter().map(|g| g.id).collect();
        for id in &ids {
            assert!(
                id.strip_prefix("glassential:")
                    .is_some_and(|p| !p.is_empty()),
                "{id} is not a glassential id"
            );
        }
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), GLASSES.len(), "duplicate glass id");
    }

    #[test]
    fn the_state_space_is_one_hundred_fifty_three() {
        let total: u32 = GLASSES.iter().map(|g| g.states).sum();
        assert_eq!(total, 153);
        for id in [
            "glassential:clear_fluid_glass",
            "glassential:clear_fluid_fake_glass",
        ] {
            let glass = GLASSES.iter().find(|g| g.id == id).expect(id);
            assert_eq!(glass.states, 64, "{id} declares six booleans");
        }
        for id in [
            "glassential:colorable_glass",
            "glassential:colorable_stained_glass",
            "glassential:smart_glass",
        ] {
            let glass = GLASSES.iter().find(|g| g.id == id).expect(id);
            assert_eq!(glass.states, 2, "{id} declares one boolean");
        }
        let singles = GLASSES.iter().filter(|g| g.states == 1).count();
        assert_eq!(singles, 19);
    }

    #[test]
    fn only_the_six_luminous_glasses_emit_fifteen() {
        let lit: Vec<&str> = GLASSES
            .iter()
            .filter(|g| g.light == 15)
            .map(|g| g.id)
            .collect();
        assert_eq!(
            lit,
            [
                "glassential:glass_light",
                "glassential:glass_light_tinted",
                "glassential:glass_lava_lamp",
                "glassential:glass_lava_lamp_tinted",
                "glassential:glass_glowstone_lamp",
                "glassential:glass_glowstone_lamp_tinted",
            ]
        );
        for glass in GLASSES.iter().filter(|g| g.light != 15) {
            assert_eq!(glass.light, 0, "{} emits {}", glass.id, glass.light);
        }
    }

    #[test]
    fn the_four_tinted_glasses_use_the_tinted_template() {
        let tinted: Vec<&str> = GLASSES.iter().filter(|g| g.tinted).map(|g| g.id).collect();
        assert_eq!(
            tinted,
            [
                "glassential:glass_light_tinted",
                "glassential:glass_redstone_tinted",
                "glassential:glass_lava_lamp_tinted",
                "glassential:glass_glowstone_lamp_tinted",
            ]
        );
        for glass in GLASSES.iter().filter(|g| g.tinted) {
            assert_eq!(glass.template, "minecraft:tinted_glass", "{}", glass.id);
        }
        // The dark-ethereal pair blocks light like vanilla tinted glass but
        // is in `GLASSES`, not `GLASSES_TINTED` (`GlassentialBlocks.java:172-176`).
        for id in [
            "glassential:glass_dark_ethereal",
            "glassential:glass_dark_ethereal_reverse",
        ] {
            let glass = GLASSES.iter().find(|g| g.id == id).expect(id);
            assert_eq!(glass.template, "minecraft:tinted_glass", "{id}");
            assert!(!glass.tinted, "{id} is not in c:glass_blocks/tinted");
        }
    }

    #[test]
    fn every_glass_drops_only_with_silk_touch() {
        for glass in &GLASSES {
            assert!(glass.silk_touch_only, "{}", glass.id);
        }
    }

    #[test]
    fn the_two_tabs_partition_the_glasses() {
        let functional = GLASSES.iter().filter(|g| g.tab == Tab::Functional).count();
        let blocks = GLASSES.iter().filter(|g| g.tab == Tab::Blocks).count();
        assert_eq!(functional, 16);
        assert_eq!(blocks, 8);
        assert_eq!(functional + blocks, GLASSES.len());
    }

    #[test]
    fn the_deferred_one_way_glasses_are_not_declared() {
        assert!(
            GLASSES.iter().all(|g| !g.id.contains("one_way")),
            "the one-way glasses need a custom baked model and are deferred"
        );
    }

    #[test]
    fn templates_are_vanilla_and_recipes_match_the_glasses() {
        for glass in &GLASSES {
            assert!(
                glass.template.strip_prefix("minecraft:").is_some(),
                "{} copies {}",
                glass.id,
                glass.template
            );
        }
        assert_eq!(RECIPES, GLASSES.len());
    }
}
