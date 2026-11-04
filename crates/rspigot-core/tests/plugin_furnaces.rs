mod common;
#[path = "common/screen.rs"]
mod screen;

use common::*;
use rspigot_core::events::ClickInput;
use rspigot_core::plugins::events::{
    BlockCookPatch, FurnaceBurn, FurnaceBurnPatch, FurnaceExtract, FurnaceExtractPatch,
    FurnaceSmelt, FurnaceStartSmelt, FurnaceStartSmeltPatch,
};
use rspigot_core::plugins::{
    Answer, EventKind, EventPatch, EventPayload, HandlerSpec, Phase, Question,
};
use rspigot_domain::{BlockPos, Vec3};
use screen::*;

const FURNACE: BlockPos = BlockPos::new(0, 64, -2);
const HOTBAR: i16 = 30;
const POTATO: &str = "minecraft:potato";
const BAKED: &str = "minecraft:baked_potato";
const COAL: &str = "minecraft:coal";
const LIT_TIME: i16 = 0;
const COOK_TOTAL: i16 = 3;

fn furnace_scene(
    handlers: Vec<HandlerSpec>,
    block: &str,
    input: (&str, u8),
) -> (Scene, Player, View) {
    let settings = rspigot_core::config::PluginSettings {
        decision_ticks: 100,
        ..rspigot_core::config::PluginSettings::default()
    };
    let mut s = scene_on(server(settings), &["Alice"], handlers);
    let alice = s.players[0].clone();
    set_block(&mut s, FURNACE, block);
    give(&mut s, &alice, 0, input.0, i32::from(input.1));
    give(&mut s, &alice, 1, COAL, 2);
    alice.link.take();
    let mut view = open_block(&mut s, &alice, FURNACE);
    for (hotbar, slot, item) in [(HOTBAR, 0, input), (HOTBAR + 1, 1, (COAL, 2))] {
        let take = predict(
            &mut view,
            press(hotbar, 0, ClickInput::Pickup),
            &[(hotbar, None)],
            stack(item.0, item.1),
        );
        send_click(&mut s, &alice, &mut view, take);
        let put = predict(
            &mut view,
            press(slot, 0, ClickInput::Pickup),
            &[(slot, stack(item.0, item.1))],
            None,
        );
        send_click(&mut s, &alice, &mut view, put);
    }
    (s, alice, view)
}

fn data(player: &Player, view: &mut View, id: i16) -> Vec<i16> {
    view.apply(&player.link.take());
    view.data
        .iter()
        .filter(|(property, _)| *property == id)
        .map(|(_, value)| *value)
        .collect()
}

fn reopened(s: &mut Scene, alice: &Player, view: &mut View) -> View {
    close_window(s, alice, view);
    open_block(s, alice, FURNACE)
}

fn burn_of(event: &Fired) -> FurnaceBurn {
    match &event.payload {
        EventPayload::FurnaceBurn(burn) => burn.clone(),
        other => panic!("expected a burn, got {other:?}"),
    }
}

fn orbs(s: &mut Scene) -> usize {
    match s.ask(Question::Entities {
        near: Some((Vec3::new(0.5, 64.0, 0.5), 8.0)),
        kind: Some("minecraft:experience_orb".to_owned()),
    }) {
        Answer::Entities(found) => found.len(),
        other => panic!("expected entities, got {other:?}"),
    }
}

fn quick(total: i32) -> Option<EventPatch> {
    Some(EventPatch::FurnaceStartSmelt(FurnaceStartSmeltPatch {
        total_cook_time: Some(total),
    }))
}

#[test]
fn a_furnace_asks_to_burn_its_fuel_once_at_a_time_and_a_refusal_keeps_it_unlit() {
    let (mut s, alice, mut view) = furnace_scene(
        vec![
            deciding(1, EventKind::FurnaceBurn),
            observe(2, EventKind::FurnaceBurn),
        ],
        "minecraft:furnace",
        (POTATO, 1),
    );
    s.tick(3);
    let asked = asked_for(&s.plugin, EventKind::FurnaceBurn);
    assert_eq!(asked.len(), 1);
    assert_eq!(
        burn_of(&asked[0]),
        FurnaceBurn {
            block: FURNACE,
            burn_time: 1600,
            burning: true,
            fuel: spec(COAL, 2),
        }
    );
    rule_on(&mut s.server, 1, asked[0].id, true, None);
    let mut view = reopened(&mut s, &alice, &mut view);
    assert_eq!(view.at(1), Some((COAL, 2)));
    assert!(
        data(&alice, &mut view, LIT_TIME)
            .iter()
            .all(|value| *value == 0)
    );
    s.tick(1);
    assert_eq!(asked_for(&s.plugin, EventKind::FurnaceBurn).len(), 2);
    assert!(settled_for(&s.plugin, EventKind::FurnaceBurn)[0].cancelled);
}

#[test]
fn a_burn_patch_sets_the_flame_and_not_burning_keeps_the_fuel() {
    let (mut s, alice, mut view) = furnace_scene(
        vec![deciding(1, EventKind::FurnaceBurn)],
        "minecraft:furnace",
        (POTATO, 1),
    );
    s.tick(1);
    let asked = last_asked(&s.plugin, EventKind::FurnaceBurn);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::FurnaceBurn(FurnaceBurnPatch {
            burn_time: Some(40),
            burning: Some(false),
        })),
    );
    s.tick(2);
    let mut view = reopened(&mut s, &alice, &mut view);
    assert_eq!(view.at(1), Some((COAL, 2)));
    let lit = data(&alice, &mut view, LIT_TIME);
    assert!(lit.iter().any(|value| (35..40).contains(value)), "{lit:?}");
    assert_eq!(asked_for(&s.plugin, EventKind::FurnaceBurn).len(), 1);

    let (mut s, alice, mut view) = furnace_scene(
        vec![deciding(1, EventKind::FurnaceBurn)],
        "minecraft:furnace",
        (POTATO, 1),
    );
    s.tick(1);
    let asked = last_asked(&s.plugin, EventKind::FurnaceBurn);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::FurnaceBurn(FurnaceBurnPatch {
            burn_time: Some(40),
            burning: None,
        })),
    );
    let view = reopened(&mut s, &alice, &mut view);
    assert_eq!(view.at(1), Some((COAL, 1)));
}

#[test]
fn the_start_of_a_smelt_carries_the_recipe_and_its_patch_sets_the_cook_time() {
    let (mut s, alice, mut view) = furnace_scene(
        vec![
            deciding(1, EventKind::FurnaceStartSmelt),
            observe(2, EventKind::FurnaceSmelt),
        ],
        "minecraft:furnace",
        (POTATO, 2),
    );
    s.tick(1);
    let asked = last_asked(&s.plugin, EventKind::FurnaceStartSmelt);
    assert_eq!(
        asked.payload,
        EventPayload::FurnaceStartSmelt(FurnaceStartSmelt {
            block: FURNACE,
            recipe: "minecraft:baked_potato".to_owned(),
            source: spec(POTATO, 2),
            total_cook_time: 200,
        })
    );
    rule_on(&mut s.server, 1, asked.id, false, quick(3));
    s.tick(3);
    let totals = data(&alice, &mut view, COOK_TOTAL);
    assert!(totals.contains(&3), "{totals:?}");
    assert_eq!(settled_for(&s.plugin, EventKind::FurnaceSmelt).len(), 1);
    let view = reopened(&mut s, &alice, &mut view);
    assert_eq!(view.at(2), Some((BAKED, 1)));
    assert_eq!(view.at(0), Some((POTATO, 1)));
    assert_eq!(asked_for(&s.plugin, EventKind::FurnaceStartSmelt).len(), 2);

    let (mut s, _alice, _view) = furnace_scene(
        vec![deciding(1, EventKind::FurnaceStartSmelt)],
        "minecraft:blast_furnace",
        ("minecraft:raw_iron", 1),
    );
    s.tick(1);
    let EventPayload::FurnaceStartSmelt(start) =
        last_asked(&s.plugin, EventKind::FurnaceStartSmelt).payload
    else {
        panic!("a start");
    };
    assert_eq!(start.total_cook_time, 100);
    assert_eq!(start.recipe, "minecraft:iron_ingot_from_blasting_raw_iron");
}

#[test]
fn a_smelt_reaches_the_block_cook_handlers_whose_cancel_loses_the_progress_and_whose_result_is_kept()
 {
    let (mut s, alice, mut view) = furnace_scene(
        vec![
            deciding(1, EventKind::BlockCook),
            deciding(3, EventKind::FurnaceStartSmelt),
            observe(2, EventKind::FurnaceSmelt),
        ],
        "minecraft:furnace",
        (POTATO, 2),
    );
    s.tick(1);
    let start = last_asked(&s.plugin, EventKind::FurnaceStartSmelt);
    judge(&mut s.server, 1, start.id, 3, false, quick(1));
    let smelt = last_asked(&s.plugin, EventKind::FurnaceSmelt);
    assert_eq!(
        smelt.payload,
        EventPayload::FurnaceSmelt(FurnaceSmelt {
            block: FURNACE,
            result: spec(BAKED, 1),
            source: spec(POTATO, 2),
        })
    );
    let handlers = s
        .plugin
        .received()
        .into_iter()
        .find_map(|message| match message {
            rspigot_core::plugins::ToPlugin::Event {
                id,
                handlers,
                phase: Phase::Decide,
                ..
            } if id == smelt.id => Some(handlers),
            _ => None,
        });
    assert_eq!(handlers, Some(vec![1]));
    judge(&mut s.server, 1, smelt.id, 1, true, None);
    let mut view = reopened(&mut s, &alice, &mut view);
    assert_eq!(view.at(2), None);
    assert_eq!(view.at(0), Some((POTATO, 2)));
    assert!(settled_for(&s.plugin, EventKind::FurnaceSmelt)[0].cancelled);

    s.tick(1);
    let start = last_asked(&s.plugin, EventKind::FurnaceStartSmelt);
    judge(&mut s.server, 1, start.id, 3, false, quick(1));
    let smelt = last_asked(&s.plugin, EventKind::FurnaceSmelt);
    judge(
        &mut s.server,
        1,
        smelt.id,
        1,
        false,
        Some(EventPatch::BlockCook(BlockCookPatch {
            result: Some(spec("minecraft:gold_ingot", 3)),
        })),
    );
    let view = reopened(&mut s, &alice, &mut view);
    assert_eq!(view.at(2), Some(("minecraft:gold_ingot", 3)));
    assert_eq!(view.at(0), Some((POTATO, 1)));
}

fn smelt_two(s: &mut Scene) {
    for _ in 0..2 {
        s.tick(1);
        let start = last_asked(&s.plugin, EventKind::FurnaceStartSmelt);
        judge(&mut s.server, 1, start.id, 3, false, quick(1));
    }
}

#[test]
fn taking_the_output_fires_one_extract_per_recipe_with_the_amount_taken() {
    let (mut s, alice, mut view) = furnace_scene(
        vec![
            deciding(1, EventKind::FurnaceExtract),
            deciding(3, EventKind::FurnaceStartSmelt),
            observe(2, EventKind::InventoryClick),
            observe(4, EventKind::FurnaceExtract),
        ],
        "minecraft:furnace",
        (POTATO, 2),
    );
    smelt_two(&mut s);
    let mut view = reopened(&mut s, &alice, &mut view);
    assert_eq!(view.at(2), Some((BAKED, 2)));
    let take = predict(
        &mut view,
        press(2, 0, ClickInput::Pickup),
        &[(2, None)],
        stack(BAKED, 2),
    );
    send_click(&mut s, &alice, &mut view, take);
    let asked = asked_for(&s.plugin, EventKind::FurnaceExtract);
    assert_eq!(asked.len(), 1);
    let EventPayload::FurnaceExtract(FurnaceExtract {
        block,
        item_amount,
        item_type,
        exp_to_drop,
        player,
    }) = &asked[0].payload
    else {
        panic!("an extract");
    };
    assert_eq!(*block, FURNACE);
    assert_eq!(*item_amount, 2);
    assert_eq!(item_type, BAKED);
    assert!((0..=1).contains(exp_to_drop));
    assert_eq!(player.name, "Alice");
    let clicks = settled_for(&s.plugin, EventKind::InventoryClick).len();
    let put = predict(
        &mut view,
        press(HOTBAR + 2, 0, ClickInput::Pickup),
        &[(HOTBAR + 2, stack(BAKED, 2))],
        None,
    );
    send_click(&mut s, &alice, &mut view, put);
    assert_eq!(
        settled_for(&s.plugin, EventKind::InventoryClick).len(),
        clicks
    );
    let before = orbs(&mut s);
    rule_on(
        &mut s.server,
        1,
        asked[0].id,
        false,
        Some(EventPatch::FurnaceExtract(FurnaceExtractPatch {
            exp_to_drop: Some(7),
        })),
    );
    assert!(orbs(&mut s) > before);
    assert_eq!(
        settled_for(&s.plugin, EventKind::InventoryClick).len(),
        clicks + 1
    );
    let settled = settled_for(&s.plugin, EventKind::FurnaceExtract);
    assert!(matches!(
        settled[0].payload,
        EventPayload::FurnaceExtract(FurnaceExtract { exp_to_drop: 7, .. })
    ));
}

#[test]
fn an_extract_with_no_experience_still_fires_and_a_zero_patch_drops_none() {
    let (mut s, alice, mut view) = furnace_scene(
        vec![
            deciding(1, EventKind::FurnaceExtract),
            deciding(3, EventKind::FurnaceStartSmelt),
        ],
        "minecraft:furnace",
        (POTATO, 2),
    );
    smelt_two(&mut s);
    let mut view = reopened(&mut s, &alice, &mut view);
    let take = predict(
        &mut view,
        press(2, 1, ClickInput::Pickup),
        &[(2, stack(BAKED, 1))],
        stack(BAKED, 1),
    );
    send_click(&mut s, &alice, &mut view, take);
    let asked = last_asked(&s.plugin, EventKind::FurnaceExtract);
    let EventPayload::FurnaceExtract(extract) = &asked.payload else {
        panic!("an extract");
    };
    assert_eq!(extract.item_amount, 1);
    let before = orbs(&mut s);
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::FurnaceExtract(FurnaceExtractPatch {
            exp_to_drop: Some(0),
        })),
    );
    assert_eq!(orbs(&mut s), before);
    let take = predict(
        &mut view,
        press(2, 0, ClickInput::Pickup),
        &[(2, None)],
        stack(BAKED, 2),
    );
    send_click(&mut s, &alice, &mut view, take);
    assert_eq!(asked_for(&s.plugin, EventKind::FurnaceExtract).len(), 1);
}
