use rspigot_blocks::{
    Air, Block, BlockId, BlockKind, Face, Furnace, Glass, GrassBlock, Lava, OakLog, PerState,
    PlaceContext, Placement, Property, ShortGrass, StateFlags, StateId, Stone, Torch, UseContext,
    WallTorch, Water,
};

#[test]
fn default_states_match_vanilla_ids() {
    assert_eq!(Air::DEFAULT_STATE, StateId(0));
    assert_eq!(StateId::AIR, StateId(0));
    assert_eq!(Stone::DEFAULT_STATE, StateId(1));
    assert_eq!(GrassBlock::DEFAULT_STATE, StateId(9));
    assert_eq!(GrassBlock::FIRST_STATE, StateId(8));
}

#[test]
fn oak_log_axis_states() {
    assert_eq!(OakLog::NAME, "minecraft:oak_log");
    assert_eq!(OakLog::FIRST_STATE, StateId(139));
    assert_eq!(OakLog::STATE_COUNT, 3);
    assert_eq!(OakLog::DEFAULT_STATE, StateId(140));
    assert_eq!(
        OakLog::PROPERTIES,
        &[Property {
            name: "axis",
            values: &["x", "y", "z"]
        }]
    );
    let axes: Vec<_> = (139..=141).map(|id| StateId(id).property("axis")).collect();
    assert_eq!(axes, [Some("x"), Some("y"), Some("z")]);
    assert_eq!(StateId(140).property("axis"), Some("y"));
    assert_eq!(StateId(140).property("facing"), None);
    assert_eq!(StateId(140).with_property("axis", "z"), Some(StateId(141)));
    assert_eq!(StateId(140).with_property("axis", "w"), None);
    assert_eq!(StateId(140).with_property("facing", "z"), None);
    assert_eq!(StateId(140).kind(), Some(BlockKind::OakLog));
    assert_eq!(
        StateId(141).properties().collect::<Vec<_>>(),
        [("axis", "z")]
    );
}

#[test]
fn multi_property_states_decompose_last_fastest() {
    let furnace = BlockKind::Furnace;
    assert_eq!(
        furnace.first_state().properties().collect::<Vec<_>>(),
        [("facing", "north"), ("lit", "true")]
    );
    assert_eq!(
        StateId(furnace.first_state().0 + 1)
            .properties()
            .collect::<Vec<_>>(),
        [("facing", "north"), ("lit", "false")]
    );
    assert_eq!(
        StateId(furnace.first_state().0 + 2)
            .properties()
            .collect::<Vec<_>>(),
        [("facing", "south"), ("lit", "true")]
    );
    let lit_east = furnace.state_with(&[("facing", "east"), ("lit", "true")]);
    assert_eq!(lit_east, Some(StateId(furnace.first_state().0 + 6)));
    assert_eq!(
        furnace.state_with(&[("facing", "east")]),
        Some(StateId(furnace.first_state().0 + 7))
    );
    assert_eq!(furnace.state_with(&[]), Some(furnace.default_state()));
    assert_eq!(furnace.state_with(&[("facing", "up")]), None);
    assert_eq!(furnace.state_with(&[("axis", "x")]), None);
}

#[test]
fn every_state_roundtrips_through_its_properties() {
    for kind in BlockKind::ALL {
        let first = kind.first_state().0;
        for id in first..first + kind.state_count() {
            let state = StateId(id);
            let pairs: Vec<_> = state.properties().collect();
            assert_eq!(pairs.len(), kind.properties().len());
            assert_eq!(kind.state_with(&pairs), Some(state));
            for (name, value) in pairs {
                assert_eq!(state.property(name), Some(value));
                assert_eq!(state.with_property(name, value), Some(state));
            }
        }
    }
}

#[test]
fn lookup_by_name_id_and_item() {
    assert_eq!(BlockKind::from_name("stone"), Some(BlockKind::Stone));
    assert_eq!(
        BlockKind::from_name("minecraft:stone"),
        Some(BlockKind::Stone)
    );
    assert_eq!(BlockKind::from_name("minecraft:air"), Some(BlockKind::Air));
    assert_eq!(BlockKind::from_name("other:stone"), None);
    assert_eq!(BlockKind::from_name("not_a_block"), None);
    assert_eq!(BlockKind::Stone.name(), "minecraft:stone");
    assert_eq!(Stone::ITEM, Some(1));
    assert_eq!(BlockKind::from_item(1), Some(BlockKind::Stone));
    assert_eq!(BlockKind::from_item(0), None);
    assert_eq!(BlockKind::from_item(u16::MAX), None);
    assert_eq!(Air::ITEM, None);
    assert_eq!(BlockKind::from_id(BlockId(1)), Some(BlockKind::Stone));
    assert_eq!(BlockKind::from_id(BlockId(1286)), None);
    let torch_item = Torch::ITEM.unwrap();
    assert_eq!(WallTorch::ITEM, Some(torch_item));
    assert_eq!(BlockKind::from_item(torch_item), Some(BlockKind::Torch));
    for kind in BlockKind::ALL {
        assert_eq!(BlockKind::from_name(kind.name()), Some(kind));
        assert_eq!(BlockKind::from_id(kind.id()), Some(kind));
        assert_eq!(kind.id(), BlockId(kind as u16));
        if let Some(block) = kind.item().and_then(BlockKind::from_item) {
            assert_eq!(block.item(), kind.item());
        }
    }
}

#[test]
fn light_values() {
    assert_eq!(Torch::DEFAULT_STATE.light_emission(), 14);
    assert_eq!(Torch::LIGHT_EMISSION, PerState::Uniform(14));
    assert_eq!(Stone::DEFAULT_STATE.light_dampening(), 15);
    assert_eq!(Glass::DEFAULT_STATE.light_dampening(), 0);
    assert_eq!(Lava::DEFAULT_STATE.light_emission(), 15);
    assert_eq!(StateId(u16::MAX).light_emission(), 0);
}

#[test]
fn furnace_light_varies_with_lit() {
    assert!(matches!(Furnace::LIGHT_EMISSION, PerState::Varies(_)));
    let lit = BlockKind::Furnace.state_with(&[("lit", "true")]);
    let unlit = BlockKind::Furnace.state_with(&[("lit", "false")]);
    assert_eq!(lit.map(StateId::light_emission), Some(13));
    assert_eq!(unlit.map(StateId::light_emission), Some(0));
    assert_eq!(Furnace::DEFAULT_STATE.light_emission(), 0);
}

#[test]
fn settings() {
    assert_eq!(Stone::SETTINGS.hardness, 1.5);
    assert_eq!(Stone::SETTINGS.resistance, 6.0);
    const { assert!(Stone::SETTINGS.requires_correct_tool) };
    assert_eq!(BlockKind::Stone.settings(), Stone::SETTINGS);
    assert_eq!(OakLog::SETTINGS.hardness, 2.0);
    const { assert!(OakLog::SETTINGS.ignited_by_lava) };
    assert_eq!(BlockKind::Bedrock.settings().hardness, -1.0);
    assert_eq!(BlockKind::Ice.settings().friction, 0.98);
}

#[test]
fn flags() {
    assert!(StateId::AIR.is_air());
    assert!(BlockKind::CaveAir.default_state().is_air());
    assert!(!Stone::DEFAULT_STATE.is_air());
    let stone = Stone::DEFAULT_STATE.flags();
    for flag in [
        StateFlags::BLOCKS_MOTION,
        StateFlags::HAS_COLLISION,
        StateFlags::CAN_OCCLUDE,
        StateFlags::SOLID_RENDER,
        StateFlags::FULL_COLLISION_CUBE,
    ] {
        assert!(stone.contains(flag));
    }
    assert!(!stone.contains(StateFlags::PROPAGATES_SKYLIGHT));
    assert!(
        Glass::DEFAULT_STATE
            .flags()
            .contains(StateFlags::PROPAGATES_SKYLIGHT)
    );
    assert!(Water::DEFAULT_STATE.flags().contains(StateFlags::WATER));
    assert!(!Water::DEFAULT_STATE.flags().contains(StateFlags::LAVA));
    assert!(Lava::DEFAULT_STATE.flags().contains(StateFlags::LAVA));
    assert!(Lava::DEFAULT_STATE.flags().has_fluid());
    assert!(
        ShortGrass::DEFAULT_STATE
            .flags()
            .contains(StateFlags::REPLACEABLE)
    );
    assert!(
        !ShortGrass::DEFAULT_STATE
            .flags()
            .contains(StateFlags::HAS_COLLISION)
    );
    let waterlogged = BlockKind::OakLeaves.state_with(&[("waterlogged", "true")]);
    assert_eq!(
        waterlogged.map(|state| state.flags().contains(StateFlags::WATER)),
        Some(true)
    );
    assert_eq!(StateId(u16::MAX).flags(), StateFlags::default());
}

#[test]
fn wall_variants() {
    assert_eq!(Torch::WALL_VARIANT, Some(WallTorch::ID));
    assert_eq!(BlockKind::Torch.wall_variant(), Some(BlockKind::WallTorch));
    assert_eq!(WallTorch::WALL_VARIANT, None);
    assert_eq!(Stone::WALL_VARIANT, None);
}

#[test]
fn counts_and_contiguous_states() {
    assert_eq!(BlockKind::ALL.len(), 1286);
    let total: usize = BlockKind::ALL
        .iter()
        .map(|kind| usize::from(kind.state_count()))
        .sum();
    assert_eq!(total, 35723);
    let mut next = 0u16;
    for (index, kind) in BlockKind::ALL.iter().enumerate() {
        assert_eq!(usize::from(kind.id().0), index);
        assert_eq!(kind.first_state(), StateId(next));
        assert!(kind.state_count() > 0);
        let range = next..next + kind.state_count();
        assert!(range.contains(&kind.default_state().0));
        for id in range {
            assert_eq!(StateId(id).kind(), Some(*kind));
            assert_eq!(BlockKind::from_state(StateId(id)), Some(*kind));
        }
        next += kind.state_count();
    }
    assert_eq!(next, 35723);
    assert_eq!(StateId(35723).kind(), None);
    assert_eq!(StateId(35723).properties().count(), 0);
    assert_eq!(StateId(35723).with_property("axis", "x"), None);
}

#[test]
fn plain_behavior_dispatch() {
    let context = PlaceContext {
        default_state: Stone::DEFAULT_STATE,
        wall_variant: None,
        face: Face::Up,
        yaw: 0.0,
        pitch: 0.0,
        cursor_y: 0.5,
    };
    assert_eq!(
        BlockKind::Stone.place(&context),
        Some(Placement {
            state: Stone::DEFAULT_STATE,
            linked: None,
        })
    );
    let use_context = UseContext {
        state: Stone::DEFAULT_STATE,
        sneaking: false,
    };
    assert_eq!(BlockKind::Stone.use_block(&use_context), None);
}
