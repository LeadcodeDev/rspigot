mod common;

use common::*;
use rspigot_core::events::{ClientEvent, Direction, Hand, Inbound, PlayerInput};
use rspigot_core::plugins::events::{
    BlockCook, BlockCookPatch, BlockPhysics, CampfireStart, CampfireStartPatch,
};
use rspigot_core::plugins::{
    Answer, BlockOrder, EventKind, EventPatch, EventPayload, ItemSpec, Order, Question,
};
use rspigot_domain::{BlockPos, Vec3};
use rspigot_items::ItemKind;

const ABOVE: BlockPos = BlockPos::new(2, 64, 0);
const TOP: BlockPos = BlockPos::new(2, 65, 0);
const ASIDE: BlockPos = BlockPos::new(3, 64, 0);

fn console(s: &mut Scene, line: &str) {
    s.server.handle(Inbound::Console(line.to_owned()));
    s.server.deliver();
}

fn set(s: &mut Scene, pos: BlockPos, key: &str) {
    s.act(Order::Block {
        pos,
        order: BlockOrder::Set(key.to_owned()),
    });
}

fn kind_at(s: &mut Scene, pos: BlockPos) -> String {
    match s.ask(Question::Block(pos)) {
        Answer::Block(state) => state.kind,
        other => panic!("expected a block, got {other:?}"),
    }
}

fn use_on(s: &mut Scene, player: &Player, pos: BlockPos, sequence: i32) {
    send_input(
        &mut s.server,
        player,
        PlayerInput::UseItemOn {
            hand: Hand::Main,
            pos,
            face: Direction::Up,
            cursor: Vec3::new(0.5, 1.0, 0.5),
            inside_block: false,
            sequence,
        },
    );
}

fn physics_at(fired: &Fired) -> BlockPos {
    match &fired.payload {
        EventPayload::BlockPhysics(event) => event.block,
        other => panic!("expected a physics event, got {other:?}"),
    }
}

fn answer_physics(s: &mut Scene, answered: &mut Vec<u64>, cancel: impl Fn(BlockPos) -> bool) {
    loop {
        let open: Vec<Fired> = asked_for(&s.plugin, EventKind::BlockPhysics)
            .into_iter()
            .filter(|asked| !answered.contains(&asked.id))
            .collect();
        if open.is_empty() {
            return;
        }
        for asked in open {
            answered.push(asked.id);
            let cancelled = cancel(physics_at(&asked));
            rule_on(&mut s.server, 1, asked.id, cancelled, None);
        }
    }
}

fn sand_on_stone(s: &mut Scene, answered: &mut Vec<u64>) {
    set(s, ABOVE, "minecraft:stone");
    set(s, TOP, "minecraft:sand");
    answer_physics(s, answered, |_| false);
    s.tick(5);
    answer_physics(s, answered, |_| false);
    assert_eq!(kind_at(s, TOP), "minecraft:sand");
}

#[test]
fn a_block_change_asks_physics_with_the_new_block() {
    let mut s = scene_with(&["Alice"], vec![observe(1, EventKind::BlockPhysics)]);
    set(&mut s, ABOVE, "minecraft:stone");
    let heard = settled_for(&s.plugin, EventKind::BlockPhysics);
    assert_eq!(
        heard.last().map(|fired| fired.payload.clone()),
        Some(EventPayload::BlockPhysics(BlockPhysics {
            block: ABOVE,
            changed_type: "minecraft:stone".to_owned(),
            source_block: ABOVE,
        }))
    );
}

#[test]
fn a_cancelled_physics_keeps_the_neighbours_from_reacting() {
    let mut s = scene_with(&["Alice"], vec![deciding(1, EventKind::BlockPhysics)]);
    let mut answered = Vec::new();
    sand_on_stone(&mut s, &mut answered);
    set(&mut s, ABOVE, "minecraft:air");
    answer_physics(&mut s, &mut answered, |pos| pos == ABOVE);
    s.tick(10);
    answer_physics(&mut s, &mut answered, |_| false);
    assert_eq!(kind_at(&mut s, TOP), "minecraft:sand");
    set(&mut s, ABOVE, "minecraft:stone");
    set(&mut s, ABOVE, "minecraft:air");
    answer_physics(&mut s, &mut answered, |_| false);
    for _ in 0..40 {
        s.tick(1);
        answer_physics(&mut s, &mut answered, |_| false);
    }
    assert_eq!(kind_at(&mut s, TOP), "minecraft:air");
}

#[test]
fn two_physics_events_on_one_position_settle_in_order() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::BlockPhysics),
            observe(2, EventKind::BlockPhysics),
        ],
    );
    set(&mut s, ABOVE, "minecraft:stone");
    set(&mut s, ABOVE, "minecraft:dirt");
    let asked = asked_for(&s.plugin, EventKind::BlockPhysics);
    assert_eq!(asked.len(), 2);
    rule_on(&mut s.server, 1, asked[1].id, false, None);
    assert!(settled_for(&s.plugin, EventKind::BlockPhysics).is_empty());
    rule_on(&mut s.server, 1, asked[0].id, false, None);
    let settled: Vec<u64> = settled_for(&s.plugin, EventKind::BlockPhysics)
        .into_iter()
        .map(|fired| fired.id)
        .collect();
    assert_eq!(settled, [asked[0].id, asked[1].id]);
}

#[test]
fn physics_events_on_different_positions_do_not_wait_for_each_other() {
    let mut s = scene_with(
        &["Alice"],
        vec![
            deciding(1, EventKind::BlockPhysics),
            observe(2, EventKind::BlockPhysics),
        ],
    );
    set(&mut s, ABOVE, "minecraft:stone");
    set(&mut s, ASIDE, "minecraft:stone");
    let asked = asked_for(&s.plugin, EventKind::BlockPhysics);
    rule_on(&mut s.server, 1, asked[1].id, false, None);
    let settled = settled_for(&s.plugin, EventKind::BlockPhysics);
    assert_eq!(settled.len(), 1);
    assert_eq!(physics_at(&settled[0]), ASIDE);
}

fn campfire_scene(handlers: Vec<rspigot_core::plugins::HandlerSpec>) -> (Scene, Player) {
    let mut s = scene_with(&["Alice"], handlers);
    let alice = s.players[0].clone();
    set(&mut s, ABOVE, "minecraft:campfire");
    console(&mut s, "give Alice beef 3");
    stream_chunk_of(&mut s, ABOVE);
    alice.link.take();
    (s, alice)
}

fn acks(events: &[ClientEvent]) -> Vec<i32> {
    events
        .iter()
        .filter_map(|event| match event {
            ClientEvent::BlockChangedAck(sequence) => Some(*sequence),
            _ => None,
        })
        .collect()
}

fn ground_keys(s: &Scene) -> Vec<String> {
    s.server
        .ground_items()
        .into_iter()
        .filter_map(|(_, stack)| stack.item.kind().map(|kind| kind.name().to_owned()))
        .collect()
}

fn start_of(fired: &Fired) -> CampfireStart {
    match &fired.payload {
        EventPayload::CampfireStart(event) => event.clone(),
        other => panic!("expected a campfire start, got {other:?}"),
    }
}

#[test]
fn putting_food_on_a_campfire_fires_start_and_its_cook_time_is_patched() {
    let (mut s, alice) = campfire_scene(vec![deciding(1, EventKind::CampfireStart)]);
    use_on(&mut s, &alice, ABOVE, 4);
    assert!(acks(&alice.link.take()).is_empty());
    let asked = last_asked(&s.plugin, EventKind::CampfireStart);
    let start = start_of(&asked);
    assert_eq!(start.block, ABOVE);
    assert_eq!(start.source, ItemSpec::of("minecraft:beef", 3));
    assert_eq!(start.total_cook_time, 600);
    assert_eq!(start.recipe, "minecraft:cooked_beef_from_campfire_cooking");
    rule_on(
        &mut s.server,
        1,
        asked.id,
        false,
        Some(EventPatch::CampfireStart(CampfireStartPatch {
            total_cook_time: Some(3),
        })),
    );
    assert_eq!(acks(&alice.link.take()), [4]);
    s.tick(4);
    assert_eq!(ground_keys(&s), ["minecraft:cooked_beef"]);
}

#[test]
fn a_cook_is_asked_once_in_flight_and_its_result_is_patched() {
    let (mut s, alice) = campfire_scene(vec![
        deciding(1, EventKind::CampfireStart),
        deciding(2, EventKind::BlockCook),
    ]);
    use_on(&mut s, &alice, ABOVE, 4);
    let asked = last_asked(&s.plugin, EventKind::CampfireStart);
    judge(
        &mut s.server,
        1,
        asked.id,
        1,
        false,
        Some(EventPatch::CampfireStart(CampfireStartPatch {
            total_cook_time: Some(2),
        })),
    );
    s.tick(3);
    let cooks = asked_for(&s.plugin, EventKind::BlockCook);
    assert_eq!(cooks.len(), 1);
    let EventPayload::BlockCook(BlockCook {
        block,
        source,
        result,
    }) = &cooks[0].payload
    else {
        panic!("expected a cook");
    };
    assert_eq!(*block, ABOVE);
    assert_eq!(*source, ItemSpec::of("minecraft:beef", 1));
    assert_eq!(*result, ItemSpec::of("minecraft:cooked_beef", 1));
    judge(
        &mut s.server,
        1,
        cooks[0].id,
        2,
        false,
        Some(EventPatch::BlockCook(BlockCookPatch {
            result: Some(ItemSpec::of("minecraft:gold_ingot", 1)),
        })),
    );
    assert_eq!(ground_keys(&s), ["minecraft:gold_ingot"]);
}

#[test]
fn a_cancelled_cook_keeps_the_food_and_is_asked_again() {
    let (mut s, alice) = campfire_scene(vec![
        deciding(1, EventKind::CampfireStart),
        deciding(2, EventKind::BlockCook),
    ]);
    use_on(&mut s, &alice, ABOVE, 4);
    let asked = last_asked(&s.plugin, EventKind::CampfireStart);
    judge(
        &mut s.server,
        1,
        asked.id,
        1,
        false,
        Some(EventPatch::CampfireStart(CampfireStartPatch {
            total_cook_time: Some(2),
        })),
    );
    s.tick(3);
    let first = last_asked(&s.plugin, EventKind::BlockCook);
    judge(&mut s.server, 1, first.id, 2, true, None);
    assert!(ground_keys(&s).is_empty());
    s.tick(1);
    let again = asked_for(&s.plugin, EventKind::BlockCook);
    assert_eq!(again.len(), 2);
    judge(&mut s.server, 1, again[1].id, 2, false, None);
    assert_eq!(ground_keys(&s), ["minecraft:cooked_beef"]);
    let air = ItemKind::Air.id();
    assert!(
        s.server
            .ground_items()
            .iter()
            .all(|(_, stack)| stack.item != air)
    );
}

#[test]
fn an_air_result_removes_the_food_without_a_drop() {
    let (mut s, alice) = campfire_scene(vec![
        deciding(1, EventKind::CampfireStart),
        deciding(2, EventKind::BlockCook),
    ]);
    use_on(&mut s, &alice, ABOVE, 4);
    let asked = last_asked(&s.plugin, EventKind::CampfireStart);
    judge(
        &mut s.server,
        1,
        asked.id,
        1,
        false,
        Some(EventPatch::CampfireStart(CampfireStartPatch {
            total_cook_time: Some(1),
        })),
    );
    s.tick(2);
    let cook = last_asked(&s.plugin, EventKind::BlockCook);
    judge(
        &mut s.server,
        1,
        cook.id,
        2,
        false,
        Some(EventPatch::BlockCook(BlockCookPatch {
            result: Some(ItemSpec::of("minecraft:air", 0)),
        })),
    );
    s.tick(2);
    assert!(ground_keys(&s).is_empty());
    assert!(asked_for(&s.plugin, EventKind::BlockCook).len() == 1);
}
