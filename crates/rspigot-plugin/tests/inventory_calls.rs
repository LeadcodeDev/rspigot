mod fake_host;

use std::sync::Mutex;
use std::time::Duration;

use fake_host::*;
use rspigot_plugin::{
    ArmorSlot, ClickKind, ClickedSlot, CustomInventory, Error, InventoryClick, InventoryClose,
    InventorySlot, ItemStack, Material,
};
use rspigot_plugin_protocol::v1::{
    self, answer, clicked_slot, custom_inventory_order, event, from_plugin, player_order, question,
    server_order, to_plugin,
};
use tonic::Status;

static OLD: Mutex<Option<CustomInventory>> = Mutex::new(None);

fn creation(body: from_plugin::Body) -> v1::CreateInventory {
    match body {
        from_plugin::Body::Server(v1::ServerOrder {
            order: Some(server_order::Order::CreateInventory(created)),
        }) => created,
        body => panic!("expected an inventory to be created, heard {body:?}"),
    }
}

fn stocked(inventory: u64, order: custom_inventory_order::Order) -> from_plugin::Body {
    from_plugin::Body::CustomInventory(v1::CustomInventoryOrder {
        inventory,
        order: Some(order),
    })
}

fn ordered(order: player_order::Order) -> from_plugin::Body {
    from_plugin::Body::Player(v1::PlayerOrder {
        player: STEVE.to_owned(),
        order: Some(order),
    })
}

fn item(key: &str, count: i32) -> v1::Item {
    v1::Item {
        key: key.to_owned(),
        count,
        ..v1::Item::default()
    }
}

fn click(
    slot: Option<clicked_slot::Slot>,
    kind: v1::ClickKind,
    hotbar_key: i32,
    item: Option<v1::Item>,
) -> event::Payload {
    event::Payload::InventoryClick(v1::InventoryClick {
        player: Some(steve()),
        inventory: 7,
        slot: Some(v1::ClickedSlot { slot }),
        kind: kind as i32,
        hotbar_key,
        item,
    })
}

fn close(inventory: u64, by_player: bool) -> event::Payload {
    event::Payload::InventoryClose(v1::InventoryClose {
        player: Some(steve()),
        inventory,
        by_player,
    })
}

fn settled(id: u64, handler: u32, payload: event::Payload) -> to_plugin::Body {
    event(id, &[handler], v1::Phase::Settled, false, payload)
}

async fn clicked(marks: &mut Marks) -> InventoryClick {
    match marks.next().await {
        Mark::Clicked(click) => click,
        mark => panic!("expected a click, got {mark:?}"),
    }
}

async fn closed(marks: &mut Marks) -> InventoryClose {
    match marks.next().await {
        Mark::Closed(close) => close,
        mark => panic!("expected a close, got {mark:?}"),
    }
}

fn wire(id: u32, event: v1::EventKind) -> v1::Handler {
    v1::Handler {
        id,
        event: event as i32,
        access: v1::Access::Observe as i32,
        priority: v1::Priority::Normal as i32,
        ignore_cancelled: false,
    }
}

#[tokio::test]
async fn creating_an_inventory_does_not_wait_and_ids_are_unique() {
    let mut called = calling(|player| {
        Box::pin(async move {
            let server = player.server();
            let menu = server.create_inventory(1, "§6Menu");
            let storage = server.create_storage(2, "Box");
            assert_ne!(menu.id(), storage.id());
            menu.set(4, Some(ItemStack::new(Material::Compass, 1)));
            player.open_inventory(&menu);
        })
    })
    .await;

    let session = &mut called.session;
    let menu = creation(session.hear().await);
    assert_ne!(menu.id, 0);
    assert_eq!(
        (menu.rows, menu.title.as_str(), menu.locked),
        (1, "§6Menu", true)
    );
    let storage = creation(session.hear().await);
    assert_eq!(
        (storage.rows, storage.title.as_str(), storage.locked),
        (2, "Box", false)
    );
    assert_ne!(storage.id, menu.id);
    assert_eq!(
        session.hear().await,
        stocked(
            menu.id,
            custom_inventory_order::Order::Set(v1::SetSlot {
                slot: 4,
                item: Some(item("minecraft:compass", 1)),
            })
        )
    );
    assert_eq!(
        session.hear().await,
        ordered(player_order::Order::OpenInventory(menu.id))
    );
    called.finished().await;
}

#[tokio::test]
async fn inventory_orders_and_resolve_use_menu_slots() {
    let mut called = calling(|player| {
        Box::pin(async move {
            let menu = player.server().create_inventory(6, "Shop");
            menu.set(53, None);
            menu.clear();
            let state = menu.resolve().await.unwrap();
            assert_eq!(
                state.items,
                [None, Some(ItemStack::new(Material::Diamond, 3))]
            );
            let viewers: Vec<&str> = state
                .viewers
                .iter()
                .map(|viewer| viewer.name.as_str())
                .collect();
            assert_eq!(viewers, ["Steve", "Alex"]);
            let shuffled = menu.resolve().await;
            assert!(matches!(shuffled, Err(Error::Malformed(_))), "{shuffled:?}");
            let gone = menu.resolve().await;
            assert!(matches!(gone, Err(Error::NotFound)), "{gone:?}");
            menu.close();
            menu.discard();
        })
    })
    .await;

    let session = &mut called.session;
    let menu = creation(session.hear().await).id;
    for order in [
        custom_inventory_order::Order::Set(v1::SetSlot {
            slot: 53,
            item: None,
        }),
        custom_inventory_order::Order::Clear(v1::Empty {}),
    ] {
        assert_eq!(session.hear().await, stocked(menu, order));
    }
    let slot = |slot, item| v1::Slot { slot, item };
    for body in [
        answer::Body::CustomInventory(v1::CustomInventoryState {
            slots: vec![slot(0, None), slot(1, Some(item("minecraft:diamond", 3)))],
            viewers: vec![steve(), alex()],
        }),
        answer::Body::CustomInventory(v1::CustomInventoryState {
            slots: vec![slot(1, None), slot(0, None)],
            viewers: Vec::new(),
        }),
        answer::Body::NotFound(v1::NotFound {}),
    ] {
        let (request, asked) = session.asked().await;
        assert_eq!(asked, question::Question::CustomInventory(menu));
        session.say(answer(request, body));
    }
    for order in [
        custom_inventory_order::Order::Close(v1::Empty {}),
        custom_inventory_order::Order::Discard(v1::Empty {}),
    ] {
        assert_eq!(session.hear().await, stocked(menu, order));
    }
    called.finished().await;
}

#[tokio::test]
async fn inventory_events_reach_their_handlers() {
    let mut called = calling(|_| Box::pin(async {})).await;
    called.finished().await;

    called.session.say(settled(
        2,
        2,
        click(
            Some(clicked_slot::Slot::Player(39)),
            v1::ClickKind::NumberKey,
            3,
            Some(item("minecraft:diamond_helmet", 1)),
        ),
    ));
    let helmet = clicked(&mut called.marks).await;
    assert_eq!(helmet.id(), 2);
    assert_eq!(helmet.player.name, "Steve");
    assert_eq!(helmet.inventory.id(), 7);
    assert_eq!(
        helmet.slot,
        ClickedSlot::Player(InventorySlot::Armor(ArmorSlot::Helmet))
    );
    assert_eq!(helmet.kind, ClickKind::NumberKey(3));
    assert_eq!(
        helmet.item.map(|item| item.material),
        Some(Material::DiamondHelmet)
    );
    helmet.inventory.close();
    assert_eq!(
        called.session.hear().await,
        stocked(7, custom_inventory_order::Order::Close(v1::Empty {}))
    );

    let menu = |slot| Some(clicked_slot::Slot::Menu(slot));
    let own = |slot| Some(clicked_slot::Slot::Player(slot));
    let outside = || Some(clicked_slot::Slot::Outside(v1::Empty {}));
    for (slot, expected) in [
        (menu(0), ClickedSlot::Menu(0)),
        (menu(53), ClickedSlot::Menu(53)),
        (own(0), ClickedSlot::Player(InventorySlot::Hotbar(0))),
        (own(9), ClickedSlot::Player(InventorySlot::Main(0))),
        (own(40), ClickedSlot::Player(InventorySlot::Offhand)),
        (outside(), ClickedSlot::Outside),
    ] {
        called
            .session
            .say(settled(3, 2, click(slot, v1::ClickKind::Left, 0, None)));
        let click = clicked(&mut called.marks).await;
        assert_eq!((click.slot, click.kind), (expected, ClickKind::Left));
        assert_eq!(click.item, None);
    }
    for (kind, hotbar_key, expected) in [
        (v1::ClickKind::Right, 0, ClickKind::Right),
        (v1::ClickKind::ShiftLeft, 0, ClickKind::ShiftLeft),
        (v1::ClickKind::ShiftRight, 0, ClickKind::ShiftRight),
        (v1::ClickKind::NumberKey, 8, ClickKind::NumberKey(8)),
        (v1::ClickKind::SwapOffhand, 0, ClickKind::SwapOffhand),
        (v1::ClickKind::Drop, 0, ClickKind::Drop),
        (v1::ClickKind::ControlDrop, 0, ClickKind::ControlDrop),
        (v1::ClickKind::DoubleClick, 0, ClickKind::DoubleClick),
        (v1::ClickKind::Middle, 0, ClickKind::Middle),
    ] {
        called
            .session
            .say(settled(4, 2, click(menu(4), kind, hotbar_key, None)));
        let click = clicked(&mut called.marks).await;
        assert_eq!((click.slot, click.kind), (ClickedSlot::Menu(4), expected));
    }

    let bad_item = Some(item("minecraft:nothing", 1));
    for unreadable in [
        click(menu(4), v1::ClickKind::NumberKey, 9, None),
        click(menu(4), v1::ClickKind::Unspecified, 0, None),
        click(menu(-1), v1::ClickKind::Left, 0, None),
        click(menu(65_536), v1::ClickKind::Left, 0, None),
        click(own(41), v1::ClickKind::Left, 0, None),
        click(None, v1::ClickKind::Left, 0, None),
        click(outside(), v1::ClickKind::Left, 0, bad_item),
    ] {
        called.session.say(settled(20, 2, unreadable));
    }

    called.session.say(settled(30, 3, close(7, true)));
    let by_player = closed(&mut called.marks).await;
    assert_eq!(by_player.id(), 30);
    assert_eq!(by_player.player.name, "Steve");
    assert_eq!(by_player.inventory.id(), 7);
    assert!(by_player.by_player);
    called.session.say(settled(31, 3, close(8, false)));
    let by_server = closed(&mut called.marks).await;
    assert_eq!(by_server.inventory.id(), 8);
    assert!(!by_server.by_player);
    assert!(called.marks.left().is_empty());
}

#[tokio::test]
async fn on_enable_runs_before_events_on_every_connection_and_on_disable_when_it_ends() {
    let mut host = Host::start().await;
    let (mut marks, ended) = start_caller(
        &host,
        |_| Box::pin(async {}),
        |_| Box::pin(tokio::time::sleep(Duration::from_millis(50))),
    );
    let knock = host.knock().await;
    assert_eq!(
        knock.hello.handlers,
        [
            wire(1, v1::EventKind::PlayerJoin),
            wire(2, v1::EventKind::InventoryClick),
            wire(3, v1::EventKind::InventoryClose),
        ]
    );
    let first = knock.accept();
    first.say(join(1));
    marks.expect(|mark| matches!(mark, Mark::Enabled)).await;
    marks.expect(|mark| matches!(mark, Mark::Finished)).await;
    first.end(Status::unavailable("the server is stopping"));
    marks.expect(|mark| matches!(mark, Mark::Disabled)).await;

    let second = host.knock().await.accept();
    second.say(join(2));
    marks.expect(|mark| matches!(mark, Mark::Enabled)).await;
    marks.expect(|mark| matches!(mark, Mark::Finished)).await;
    second.say(to_plugin::Body::Replaced(v1::Replaced {}));

    let ended = within(ended).await.unwrap();
    assert!(matches!(ended, Err(Error::Replaced)), "got {ended:?}");
    let left = marks.left();
    assert!(
        matches!(left.as_slice(), [Mark::Disabled]),
        "expected the last on_disable before the plugin stopped, got {left:?}"
    );
}

#[tokio::test]
async fn an_order_on_an_inventory_from_an_earlier_connection_is_dropped() {
    let mut host = Host::start().await;
    let (mut marks, _) = start_caller(
        &host,
        |player| {
            Box::pin(async move {
                let old = OLD.lock().unwrap().clone().expect("a stored inventory");
                old.set(0, None);
                player.open_inventory(&old);
                let resolved = old.resolve().await;
                assert!(matches!(resolved, Err(Error::NotFound)), "{resolved:?}");
                player.send_message("marker");
            })
        },
        |server| {
            Box::pin(async move {
                let mut old = OLD.lock().unwrap();
                if old.is_none() {
                    *old = Some(server.create_inventory(1, "M"));
                }
            })
        },
    );
    let mut first = host.knock().await.accept();
    let created = creation(first.hear().await);
    assert_eq!(
        (created.rows, created.title.as_str(), created.locked),
        (1, "M", true)
    );
    marks.expect(|mark| matches!(mark, Mark::Enabled)).await;
    first.end(Status::unavailable("the server is stopping"));
    marks.expect(|mark| matches!(mark, Mark::Disabled)).await;

    let mut second = host.knock().await.accept();
    marks.expect(|mark| matches!(mark, Mark::Enabled)).await;
    second.say(join(2));
    marks.expect(|mark| matches!(mark, Mark::Finished)).await;
    assert_eq!(
        second.hear().await,
        ordered(player_order::Order::SendMessage("marker".to_owned()))
    );
}
