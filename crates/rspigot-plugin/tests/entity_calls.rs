mod fake_host;

use fake_host::*;
use rspigot_plugin::{EntityType, Hand, Position};
use rspigot_plugin_protocol::v1::{self, entity_order, event, from_plugin, question};

const HERE: Position = Position {
    x: 4.5,
    y: 64.0,
    z: 4.5,
};

fn guide() -> v1::EntitySnapshot {
    v1::EntitySnapshot {
        entity_id: 77,
        uuid: "00000000-0000-0000-0000-000000000009".to_owned(),
        kind: "minecraft:villager".to_owned(),
        position: Some(v1::Vec3 {
            x: 4.5,
            y: 64.0,
            z: 4.5,
        }),
    }
}

fn interaction(
    hand: v1::Hand,
    sneaking: bool,
    entity: Option<v1::EntitySnapshot>,
) -> event::Payload {
    event::Payload::EntityInteract(v1::EntityInteract {
        player: Some(steve()),
        entity,
        hand: hand as i32,
        sneaking,
    })
}

fn ordered(order: entity_order::Order) -> from_plugin::Body {
    from_plugin::Body::Entity(v1::EntityOrder {
        entity: 77,
        order: Some(order),
    })
}

#[tokio::test]
async fn an_entity_interaction_reaches_the_handler() {
    let mut host = Host::start().await;
    let (mut marks, _) = start_caller(&host, |_| Box::pin(async {}), idle);
    let session = host.knock().await.accept();
    marks.expect(|mark| matches!(mark, Mark::Enabled)).await;

    session.say(event(
        1,
        &[4],
        v1::Phase::Settled,
        false,
        interaction(v1::Hand::Off, true, Some(guide())),
    ));
    let Mark::Interacted(interact) = marks.next().await else {
        panic!("expected an interaction");
    };
    assert_eq!(interact.id(), 1);
    assert_eq!(interact.player.name, "Steve");
    assert_eq!(interact.entity.entity_id, 77);
    assert_eq!(interact.entity.kind, EntityType::Villager);
    assert_eq!(interact.entity.position, HERE);
    assert_eq!(interact.hand, Hand::Off);
    assert!(interact.sneaking);

    session.say(event(
        2,
        &[4],
        v1::Phase::Settled,
        false,
        interaction(v1::Hand::Main, false, Some(guide())),
    ));
    let Mark::Interacted(interact) = marks.next().await else {
        panic!("expected an interaction");
    };
    assert_eq!(interact.hand, Hand::Main);
    assert!(!interact.sneaking);
}

#[tokio::test]
async fn a_malformed_interaction_is_skipped() {
    let mut host = Host::start().await;
    let (mut marks, _) = start_caller(&host, |_| Box::pin(async {}), idle);
    let session = host.knock().await.accept();
    marks.expect(|mark| matches!(mark, Mark::Enabled)).await;

    session.say(event(
        1,
        &[4],
        v1::Phase::Settled,
        false,
        interaction(v1::Hand::Main, false, None),
    ));
    session.say(event(
        2,
        &[4],
        v1::Phase::Settled,
        false,
        interaction(v1::Hand::Unspecified, false, Some(guide())),
    ));
    session.say(event(
        3,
        &[4],
        v1::Phase::Settled,
        false,
        interaction(v1::Hand::Main, false, Some(guide())),
    ));
    let Mark::Interacted(interact) = marks.next().await else {
        panic!("expected an interaction");
    };
    assert_eq!(interact.id(), 3);
}

#[tokio::test]
async fn appearance_calls_are_orders() {
    let mut called = calling(|player| {
        Box::pin(async move {
            let guide = player
                .server()
                .world()
                .spawn(EntityType::Villager, HERE)
                .await
                .expect("the guide spawns");
            guide.set_custom_name("§aGuide");
            guide.set_name_visible(true);
            guide.set_no_ai(true);
            guide.set_invulnerable(true);
            guide.set_silent(true);
            guide.clear_custom_name();
        })
    })
    .await;

    let session = &mut called.session;
    let (request, asked) = session.asked().await;
    assert!(matches!(asked, question::Question::Spawn(_)));
    session.say(answer(request, v1::answer::Body::Spawned(guide())));

    let expected = [
        entity_order::Order::SetCustomName("§aGuide".to_owned()),
        entity_order::Order::SetNameVisible(true),
        entity_order::Order::SetNoAi(true),
        entity_order::Order::SetInvulnerable(true),
        entity_order::Order::SetSilent(true),
        entity_order::Order::ClearCustomName(v1::Empty {}),
    ];
    for order in expected {
        assert_eq!(session.hear().await, ordered(order));
    }
    called.finished().await;
}
