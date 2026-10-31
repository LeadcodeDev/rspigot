use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rspigot_blocks::BlockKind;
use rspigot_core::admin::{AdminLists, ListEntry};
use rspigot_core::config::{GameConfig, LevelData};
use rspigot_core::events::{
    ClickInput, ClientEvent, ContainerClick, DigAction, Direction, Hand, HashedSlot, Inbound,
    InputState, PlayerInput,
};
use rspigot_core::ports::{ChunkGenerator, ChunkStore, ClientLink, EntityStore, StoreError};
use rspigot_core::server::Server;
use rspigot_core::status::StatusBoard;
use rspigot_domain::chunk::ChunkColumn;
use rspigot_domain::player::{ClientSettings, GameProfile, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::world::Dimension;
use rspigot_domain::{BiomeId, BlockPos, BlockStateId, ChunkPos, Identifier, Vec3};
use rspigot_nbt::Compound;

#[derive(Clone, Default)]
struct Link {
    events: Arc<Mutex<Vec<ClientEvent>>>,
    closed: Arc<AtomicBool>,
}

impl Link {
    fn take(&self) -> Vec<ClientEvent> {
        self.events
            .lock()
            .map(|mut e| std::mem::take(&mut *e))
            .unwrap_or_default()
    }
}

impl ClientLink for Link {
    fn send(&self, event: ClientEvent) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event);
        }
    }

    fn disconnect(&self, _: Component) {
        self.closed.store(true, Ordering::Release);
    }

    fn is_connected(&self) -> bool {
        !self.closed.load(Ordering::Acquire)
    }
}

struct NoStore;

impl ChunkStore for NoStore {
    fn load(&self, _: &Identifier, _: ChunkPos) -> Result<Option<ChunkColumn>, StoreError> {
        Ok(None)
    }

    fn save(&self, _: &Identifier, _: &ChunkColumn) -> Result<(), StoreError> {
        Ok(())
    }

    fn flush(&self) -> Result<(), StoreError> {
        Ok(())
    }
}

impl EntityStore for NoStore {
    fn load_entities(
        &self,
        _: &Identifier,
        _: ChunkPos,
    ) -> Result<Option<Vec<Compound>>, StoreError> {
        Ok(None)
    }

    fn save_entities(&self, _: &Identifier, _: ChunkPos, _: &[Compound]) -> Result<(), StoreError> {
        Ok(())
    }
}

struct Floor;

impl ChunkGenerator for Floor {
    fn generate(&self, dimension: &Dimension, pos: ChunkPos) -> ChunkColumn {
        let mut chunk = ChunkColumn::empty(pos, dimension.height, BiomeId(0));
        for x in 0..16 {
            for z in 0..16 {
                for y in 60..=63 {
                    chunk.set_block(
                        BlockPos::new(pos.min_block_x() + x, y, pos.min_block_z() + z),
                        BlockStateId(1),
                    );
                }
            }
        }
        chunk
    }

    fn spawn_point(&self, _: &Dimension) -> Vec3 {
        Vec3::new(0.5, 64.0, 0.5)
    }
}

const FLOOR: BlockPos = BlockPos::new(2, 63, 2);
const CHEST: BlockPos = BlockPos::new(2, 64, 2);
const ABOVE: BlockPos = BlockPos::new(2, 65, 2);

struct Player {
    id: PlayerId,
    link: Link,
    log: Vec<ClientEvent>,
}

fn server() -> Server<Link> {
    let config = GameConfig {
        view_distance: 3,
        chunk_workers: 2,
        spawn_protection: 0,
        admin: AdminLists {
            operators: vec![ListEntry::named("Alice")],
            whitelist: Vec::new(),
            whitelist_enabled: false,
        },
        ..GameConfig::default()
    };
    let mut server = Server::new(
        config,
        LevelData::default(),
        Arc::new(NoStore),
        Arc::new(Floor),
        StatusBoard::new(20),
    );
    server.wait_for_spawn(Duration::from_secs(10));
    server
}

fn join(server: &mut Server<Link>, name: &str) -> Player {
    let link = Link::default();
    let id = PlayerId::offline(name);
    server.handle(Inbound::Join {
        profile: GameProfile {
            id,
            name: name.to_owned(),
            properties: Vec::new(),
        },
        settings: ClientSettings::default(),
        link: link.clone(),
    });
    server.deliver();
    let mut player = Player {
        id,
        link,
        log: Vec::new(),
    };
    player.log.extend(player.link.take());
    for event in player.log.clone() {
        if let ClientEvent::Teleport { id: teleport, .. } = event {
            server.handle(Inbound::Input {
                player: id,
                input: PlayerInput::TeleportConfirm(teleport),
            });
        }
    }
    server.handle(Inbound::Input {
        player: id,
        input: PlayerInput::Loaded,
    });
    player
}

fn run_ticks(server: &mut Server<Link>, players: &mut [&mut Player], ticks: usize) {
    let deadline = Instant::now() + Duration::from_secs(20);
    for _ in 0..ticks {
        server.wait_for_spawn(Duration::from_millis(20));
        server.tick();
        for player in players.iter_mut() {
            let events = player.link.take();
            let batches = events
                .iter()
                .filter(|e| matches!(e, ClientEvent::ChunkBatchFinished { .. }))
                .count();
            for _ in 0..batches {
                server.handle(Inbound::Input {
                    player: player.id,
                    input: PlayerInput::ChunkBatchReceived {
                        chunks_per_tick: 64.0,
                    },
                });
            }
            player.log.extend(events);
        }
        if Instant::now() > deadline {
            break;
        }
    }
}

struct World {
    server: Server<Link>,
    alice: Player,
    bob: Player,
}

fn world() -> World {
    let mut server = server();
    let mut alice = join(&mut server, "Alice");
    let mut bob = join(&mut server, "Bob");
    run_ticks(&mut server, &mut [&mut alice, &mut bob], 30);
    World { server, alice, bob }
}

impl World {
    fn settle(&mut self) {
        self.server.deliver();
        run_ticks(&mut self.server, &mut [&mut self.alice, &mut self.bob], 2);
    }

    fn console(&mut self, line: &str) {
        self.server.handle(Inbound::Console(line.to_owned()));
        self.settle();
    }

    fn select(&mut self, slot: u8) {
        self.server.handle(Inbound::Input {
            player: self.alice.id,
            input: PlayerInput::HeldSlot(slot),
        });
    }

    fn use_on(&mut self, pos: BlockPos, face: Direction) {
        self.server.handle(Inbound::Input {
            player: self.alice.id,
            input: PlayerInput::UseItemOn {
                hand: Hand::Main,
                pos,
                face,
                cursor: Vec3::new(0.5, 1.0, 0.5),
                inside_block: false,
                sequence: 7,
            },
        });
        self.settle();
    }

    fn give(&mut self, item: &str) {
        self.console(&format!("give Alice minecraft:{item}"));
    }
}

fn last_update(player: &Player, pos: BlockPos) -> Option<BlockStateId> {
    player.log.iter().rev().find_map(|event| match event {
        ClientEvent::BlockUpdate { pos: at, state } if *at == pos => Some(*state),
        _ => None,
    })
}

fn menu_events(player: &Player) -> Vec<&ClientEvent> {
    player
        .log
        .iter()
        .filter(|event| {
            matches!(
                event,
                ClientEvent::OpenScreen { .. }
                    | ClientEvent::ContainerContent { .. }
                    | ClientEvent::ContainerSlot { .. }
                    | ClientEvent::ContainerClose(_)
            )
        })
        .collect()
}

fn opened(player: &Player) -> Option<(i32, i32)> {
    player.log.iter().rev().find_map(|event| match event {
        ClientEvent::OpenScreen { window, menu, .. } => Some((*window, *menu)),
        _ => None,
    })
}

fn state_id(player: &Player, window: i32) -> i32 {
    player
        .log
        .iter()
        .rev()
        .find_map(|event| match event {
            ClientEvent::ContainerContent {
                window: at,
                state_id,
                ..
            }
            | ClientEvent::ContainerSlot {
                window: at,
                state_id,
                ..
            } if *at == window => Some(*state_id),
            _ => None,
        })
        .unwrap_or(0)
}

fn content(player: &Player, window: i32) -> Option<Vec<Option<(u16, u8)>>> {
    player.log.iter().rev().find_map(|event| match event {
        ClientEvent::ContainerContent {
            window: at, slots, ..
        } if *at == window => Some(
            slots
                .iter()
                .map(|slot| slot.as_ref().map(|stack| (stack.item.0, stack.count.get())))
                .collect(),
        ),
        _ => None,
    })
}

fn slot_update(player: &Player, window: i32, slot: i16) -> Option<Option<(u16, u8)>> {
    player.log.iter().rev().find_map(|event| match event {
        ClientEvent::ContainerSlot {
            window: at,
            slot: index,
            stack,
            ..
        } if *at == window && *index == slot => Some(
            stack
                .as_ref()
                .map(|stack| (stack.item.0, stack.count.get())),
        ),
        _ => None,
    })
}

fn closed(player: &Player, window: i32) -> bool {
    player
        .log
        .iter()
        .any(|event| matches!(event, ClientEvent::ContainerClose(at) if *at == window))
}

fn item(name: &str) -> u16 {
    rspigot_items::ItemKind::from_name(name)
        .expect("known item")
        .id()
        .0
}

impl World {
    fn click_as(&mut self, who: &Player, window: i32, slot: i16, button: i8, input: ClickInput) {
        let state = state_id(who, window);
        self.server.handle(Inbound::Input {
            player: who.id,
            input: PlayerInput::ContainerClick(ContainerClick {
                container: window,
                state_id: state,
                slot,
                button,
                input,
                changed: Vec::new(),
                carried: HashedSlot::default(),
            }),
        });
        self.settle();
    }

    fn click(&mut self, window: i32, slot: i16, button: i8, input: ClickInput) {
        let alice = Player {
            id: self.alice.id,
            link: self.alice.link.clone(),
            log: self.alice.log.clone(),
        };
        self.click_as(&alice, window, slot, button, input);
    }

    fn close(&mut self, window: i32) {
        self.server.handle(Inbound::Input {
            player: self.alice.id,
            input: PlayerInput::ContainerClose(window),
        });
        self.settle();
    }

    fn place_chest(&mut self) {
        self.give("chest");
        self.select(0);
        self.use_on(FLOOR, Direction::Up);
    }

    fn sneak(&mut self, shift: bool) {
        self.server.handle(Inbound::Input {
            player: self.alice.id,
            input: PlayerInput::Input(InputState {
                shift,
                ..InputState::default()
            }),
        });
    }

    fn open_chest(&mut self) {
        self.select(8);
        self.use_on(CHEST, Direction::Up);
    }

    fn open_chest_as_bob(&mut self) {
        self.server.handle(Inbound::Input {
            player: self.bob.id,
            input: PlayerInput::UseItemOn {
                hand: Hand::Main,
                pos: CHEST,
                face: Direction::Up,
                cursor: Vec3::new(0.5, 1.0, 0.5),
                inside_block: false,
                sequence: 9,
            },
        });
        self.settle();
    }
}

const HOTBAR_0: i16 = 54;
const WINDOW: i32 = 1;

#[test]
fn opening_a_chest_sends_the_screen_then_its_content() {
    let mut world = world();
    world.place_chest();
    world.open_chest();

    assert_eq!(opened(&world.alice), Some((WINDOW, 2)));
    let slots = content(&world.alice, WINDOW).expect("content");
    assert_eq!(slots.len(), 63);
    assert!(slots.iter().all(Option::is_none));
}

#[test]
fn items_put_in_the_chest_are_still_there_when_it_is_reopened() {
    let mut world = world();
    world.place_chest();
    world.give("stone");
    world.open_chest();
    world.click(WINDOW, HOTBAR_0, 0, ClickInput::Pickup);
    world.click(WINDOW, 3, 0, ClickInput::Pickup);
    world.close(WINDOW);
    world.open_chest();

    let slots = content(&world.alice, 2).expect("second window content");
    assert_eq!(slots[3], Some((item("minecraft:stone"), 1)));
    assert_eq!(slots[HOTBAR_0 as usize], None);
}

#[test]
fn a_click_for_a_window_that_is_not_open_changes_nothing() {
    let mut world = world();
    world.place_chest();
    world.give("stone");
    world.open_chest();
    world.click(77, HOTBAR_0, 0, ClickInput::Pickup);
    world.click(77, 3, 0, ClickInput::Pickup);
    world.close(WINDOW);
    world.open_chest();

    let slots = content(&world.alice, 2).expect("content");
    assert_eq!(slots[3], None);
}

#[test]
fn two_viewers_of_one_chest_see_each_others_changes() {
    let mut world = world();
    world.place_chest();
    world.give("stone");
    world.open_chest();
    world.open_chest_as_bob();
    assert_eq!(opened(&world.bob), Some((WINDOW, 2)));

    world.click(WINDOW, HOTBAR_0, 0, ClickInput::Pickup);
    world.click(WINDOW, 5, 0, ClickInput::Pickup);
    assert_eq!(
        slot_update(&world.bob, WINDOW, 5),
        Some(Some((item("minecraft:stone"), 1)))
    );
}

#[test]
fn closing_the_window_gives_back_the_stack_in_hand() {
    let mut world = world();
    world.place_chest();
    world.give("stone");
    world.open_chest();
    world.click(WINDOW, HOTBAR_0, 0, ClickInput::Pickup);
    world.close(WINDOW);

    let back = world.alice.log.iter().any(|event| {
        matches!(
            event,
            ClientEvent::PlayerInventorySlot { stack: Some(stack), .. }
                if stack.item.0 == item("minecraft:stone")
        )
    });
    assert!(back, "the carried stack returns to the inventory");
}

#[test]
fn a_solid_block_above_a_chest_keeps_it_shut() {
    let mut world = world();
    world.place_chest();
    world.give("stone");
    world.select(0);
    world.sneak(true);
    world.use_on(CHEST, Direction::Up);
    world.sneak(false);
    assert!(last_update(&world.alice, ABOVE).is_some());
    world.open_chest();

    assert_eq!(opened(&world.alice), None);
}

#[test]
fn breaking_the_chest_closes_the_window() {
    let mut world = world();
    world.place_chest();
    world.open_chest();
    world.server.handle(Inbound::Input {
        player: world.alice.id,
        input: PlayerInput::Dig {
            action: DigAction::Finish,
            pos: CHEST,
            face: Direction::Up,
            sequence: 4,
        },
    });
    world.settle();

    assert!(closed(&world.alice, WINDOW));
}

#[test]
fn walking_away_closes_the_window() {
    let mut world = world();
    world.place_chest();
    world.open_chest();
    world.console("tp Alice 100 64 100");

    assert!(closed(&world.alice, WINDOW));
}

#[test]
fn a_stale_state_id_resends_the_whole_window() {
    let mut world = world();
    world.place_chest();
    world.open_chest();
    let before = menu_events(&world.alice)
        .iter()
        .filter(|event| matches!(event, ClientEvent::ContainerContent { .. }))
        .count();
    world.server.handle(Inbound::Input {
        player: world.alice.id,
        input: PlayerInput::ContainerClick(ContainerClick {
            container: WINDOW,
            state_id: 12345,
            slot: 0,
            button: 0,
            input: ClickInput::Pickup,
            changed: Vec::new(),
            carried: HashedSlot::default(),
        }),
    });
    world.settle();
    let after = menu_events(&world.alice)
        .iter()
        .filter(|event| matches!(event, ClientEvent::ContainerContent { .. }))
        .count();
    assert_eq!(after, before + 1);
}

fn lid_events(player: &Player) -> Vec<u8> {
    player
        .log
        .iter()
        .filter_map(|event| match event {
            ClientEvent::BlockEvent {
                pos,
                action: 1,
                data,
                ..
            } if *pos == CHEST => Some(*data),
            _ => None,
        })
        .collect()
}

fn sounds_heard(player: &Player, name: &str) -> usize {
    player
        .log
        .iter()
        .filter(|event| matches!(event, ClientEvent::Sound { sound, .. } if *sound == name))
        .count()
}

impl World {
    fn place_storage(&mut self, item: &str) {
        self.give(item);
        self.select(0);
        self.use_on(FLOOR, Direction::Up);
    }

    fn close_as_bob(&mut self, window: i32) {
        self.server.handle(Inbound::Input {
            player: self.bob.id,
            input: PlayerInput::ContainerClose(window),
        });
        self.settle();
    }
}

#[test]
fn each_storage_block_opens_its_own_menu() {
    let blocks = [
        ("trapped_chest", 2, 63),
        ("barrel", 2, 63),
        ("shulker_box", 20, 63),
        ("blue_shulker_box", 20, 63),
        ("hopper", 16, 41),
        ("dispenser", 6, 45),
        ("dropper", 6, 45),
    ];
    for (item, menu, slots) in blocks {
        let mut world = world();
        world.place_storage(item);
        world.open_chest();

        assert_eq!(opened(&world.alice), Some((WINDOW, menu)), "{item}");
        let content = content(&world.alice, WINDOW).unwrap_or_else(|| panic!("{item} content"));
        assert_eq!(content.len(), slots, "{item}");
    }
}

#[test]
fn the_chest_lid_follows_the_viewers_and_the_sound_the_first_and_last() {
    let mut world = world();
    world.place_chest();
    world.open_chest();
    assert_eq!(lid_events(&world.bob), vec![1]);
    assert_eq!(sounds_heard(&world.bob, "minecraft:block.chest.open"), 1);

    world.open_chest_as_bob();
    assert_eq!(lid_events(&world.bob), vec![1, 2]);
    assert_eq!(sounds_heard(&world.bob, "minecraft:block.chest.open"), 1);

    world.close(WINDOW);
    assert_eq!(lid_events(&world.bob), vec![1, 2, 1]);
    assert_eq!(sounds_heard(&world.bob, "minecraft:block.chest.close"), 0);

    world.close_as_bob(WINDOW);
    assert_eq!(lid_events(&world.bob), vec![1, 2, 1, 0]);
    assert_eq!(sounds_heard(&world.bob, "minecraft:block.chest.close"), 1);
}

#[test]
fn a_barrel_shows_it_is_open_while_it_is_viewed() {
    let open_state = |open: &str| {
        BlockKind::Barrel
            .default_state()
            .with_property("open", open)
            .expect("open property")
    };
    let mut world = world();
    world.place_storage("barrel");
    let shut = last_update(&world.alice, CHEST).expect("placed");
    assert_eq!(shut.property("open"), Some("false"));

    world.open_chest();
    let viewed = last_update(&world.alice, CHEST).expect("updated");
    assert_eq!(viewed.property("open"), Some("true"));
    assert_eq!(viewed.kind(), open_state("true").kind());
    assert_eq!(sounds_heard(&world.bob, "minecraft:block.barrel.open"), 1);

    world.close(WINDOW);
    let closed_again = last_update(&world.alice, CHEST).expect("updated");
    assert_eq!(closed_again.property("open"), Some("false"));
    assert_eq!(sounds_heard(&world.bob, "minecraft:block.barrel.close"), 1);
}

#[test]
fn a_shulker_box_with_a_block_in_front_does_not_open() {
    let mut world = world();
    world.place_storage("shulker_box");
    world.give("stone");
    world.select(0);
    world.sneak(true);
    world.use_on(CHEST, Direction::Up);
    world.sneak(false);
    assert!(last_update(&world.alice, ABOVE).is_some());
    world.open_chest();

    assert_eq!(opened(&world.alice), None);
}

#[test]
fn a_hopper_has_no_lid_and_makes_no_sound() {
    let mut world = world();
    world.place_storage("hopper");
    world.open_chest();

    assert!(lid_events(&world.bob).is_empty());
    assert!(
        !world
            .bob
            .log
            .iter()
            .any(|event| matches!(event, ClientEvent::Sound { .. }))
    );
}

fn data_events(player: &Player, window: i32) -> Vec<(i16, i16)> {
    player
        .log
        .iter()
        .filter_map(|event| match event {
            ClientEvent::ContainerData {
                window: at,
                id,
                value,
            } if *at == window => Some((*id, *value)),
            _ => None,
        })
        .collect()
}

fn open_as_bob(world: &mut World) {
    world.open_chest_as_bob();
}

const GRID_HOTBAR_0: i16 = 37;

#[test]
fn the_crafting_table_opens_a_grid_menu_with_a_result_slot() {
    let mut world = world();
    world.place_storage("crafting_table");
    world.open_chest();

    assert_eq!(opened(&world.alice), Some((WINDOW, 12)));
    let slots = content(&world.alice, WINDOW).expect("content");
    assert_eq!(slots.len(), 46);
}

#[test]
fn the_crafting_grid_comes_back_when_the_window_closes_and_is_not_kept() {
    let mut world = world();
    world.place_storage("crafting_table");
    world.give("stone");
    world.open_chest();
    world.click(WINDOW, GRID_HOTBAR_0, 0, ClickInput::Pickup);
    world.click(WINDOW, 1, 0, ClickInput::Pickup);
    world.close(WINDOW);

    let back = world.alice.log.iter().any(|event| {
        matches!(
            event,
            ClientEvent::PlayerInventorySlot { stack: Some(stack), .. }
                if stack.item.0 == item("minecraft:stone")
        )
    });
    assert!(back, "the grid item returns to the inventory");
    world.open_chest();
    let slots = content(&world.alice, 2).expect("second window");
    assert!(slots[..10].iter().all(Option::is_none));
}

#[test]
fn two_players_have_separate_crafting_grids() {
    let mut world = world();
    world.place_storage("crafting_table");
    world.give("stone");
    world.open_chest();
    open_as_bob(&mut world);
    world.click(WINDOW, GRID_HOTBAR_0, 0, ClickInput::Pickup);
    world.click(WINDOW, 1, 0, ClickInput::Pickup);

    assert_eq!(slot_update(&world.bob, WINDOW, 1), None);
    let bob_slots = content(&world.bob, WINDOW).expect("bob content");
    assert!(bob_slots[..10].iter().all(Option::is_none));
}

#[test]
fn the_crafting_result_slot_takes_nothing() {
    let mut world = world();
    world.place_storage("crafting_table");
    world.give("stone");
    world.open_chest();
    world.click(WINDOW, GRID_HOTBAR_0, 0, ClickInput::Pickup);
    world.click(WINDOW, 0, 0, ClickInput::Pickup);

    assert_eq!(slot_update(&world.alice, WINDOW, 0), None);
    world.close(WINDOW);
    world.open_chest();
    let slots = content(&world.alice, 2).expect("second window");
    assert_eq!(slots[0], None);
}

#[test]
fn each_furnace_opens_with_its_own_menu_and_four_data_slots() {
    for (item, menu) in [("furnace", 14), ("blast_furnace", 10), ("smoker", 22)] {
        let mut world = world();
        world.place_storage(item);
        world.open_chest();

        assert_eq!(opened(&world.alice), Some((WINDOW, menu)), "{item}");
        let slots = content(&world.alice, WINDOW).unwrap_or_else(|| panic!("{item} content"));
        assert_eq!(slots.len(), 39, "{item}");
        assert_eq!(
            data_events(&world.alice, WINDOW),
            vec![(0, 0), (1, 0), (2, 0), (3, 0)],
            "{item}"
        );
    }
}

#[test]
fn furnace_items_are_kept_and_the_result_slot_refuses_them() {
    let mut world = world();
    world.place_storage("furnace");
    world.give("iron_ore");
    world.open_chest();
    let main_slot = 3 + 27;
    world.click(WINDOW, main_slot, 0, ClickInput::Pickup);
    world.click(WINDOW, 2, 0, ClickInput::Pickup);
    world.click(WINDOW, 0, 0, ClickInput::Pickup);
    world.close(WINDOW);
    world.open_chest();

    let slots = content(&world.alice, 2).expect("second window");
    assert_eq!(slots[0], Some((item("minecraft:iron_ore"), 1)));
    assert_eq!(slots[2], None);
}

const RIGHT_HALF: BlockPos = BlockPos::new(3, 64, 2);

fn chest_type_at(player: &Player, pos: BlockPos) -> Option<String> {
    last_update(player, pos).and_then(|state| state.property("type").map(str::to_owned))
}

impl World {
    fn place_double_chest(&mut self) {
        self.give("chest");
        self.give("chest");
        self.select(0);
        self.use_on(FLOOR, Direction::Up);
        self.use_on(BlockPos::new(3, 63, 2), Direction::Up);
    }

    fn open_at(&mut self, pos: BlockPos) {
        self.select(8);
        self.use_on(pos, Direction::Up);
    }
}

#[test]
fn two_chests_side_by_side_become_the_two_halves_of_a_double_chest() {
    let mut world = world();
    world.place_double_chest();

    assert_eq!(chest_type_at(&world.alice, CHEST).as_deref(), Some("left"));
    assert_eq!(
        chest_type_at(&world.alice, RIGHT_HALF).as_deref(),
        Some("right")
    );
    assert_eq!(chest_type_at(&world.bob, CHEST).as_deref(), Some("left"));
}

#[test]
fn a_double_chest_opens_a_54_slot_window_from_either_half() {
    let mut world = world();
    world.place_double_chest();
    world.open_at(CHEST);

    assert_eq!(opened(&world.alice), Some((WINDOW, 5)));
    assert_eq!(content(&world.alice, WINDOW).expect("content").len(), 90);
    let title_is_double = world.alice.log.iter().any(|event| {
        matches!(event, ClientEvent::OpenScreen { title, .. }
            if format!("{title:?}").contains("container.chestDouble"))
    });
    assert!(title_is_double);
    world.close(WINDOW);
    world.open_at(RIGHT_HALF);
    assert_eq!(opened(&world.alice), Some((2, 5)));
}

#[test]
fn the_two_halves_hold_the_first_and_the_last_27_slots() {
    let mut world = world();
    world.place_double_chest();
    world.give("stone");
    world.open_at(CHEST);
    let hotbar_0 = 54 + 27;
    world.click(WINDOW, hotbar_0 as i16, 0, ClickInput::Pickup);
    world.click(WINDOW, 40, 0, ClickInput::Pickup);
    world.close(WINDOW);
    world.open_at(RIGHT_HALF);

    let slots = content(&world.alice, 2).expect("second window");
    assert_eq!(slots[40], Some((item("minecraft:stone"), 1)));
    assert_eq!(slots[..40].iter().flatten().count(), 0);
}

#[test]
fn both_halves_animate_and_the_sound_comes_once_from_the_middle() {
    let mut world = world();
    world.place_double_chest();
    world.open_at(CHEST);

    let events = |player: &Player, pos: BlockPos| -> Vec<u8> {
        player
            .log
            .iter()
            .filter_map(|event| match event {
                ClientEvent::BlockEvent {
                    pos: at,
                    action: 1,
                    data,
                    ..
                } if *at == pos => Some(*data),
                _ => None,
            })
            .collect()
    };
    assert_eq!(events(&world.bob, CHEST), vec![1]);
    assert_eq!(events(&world.bob, RIGHT_HALF), vec![1]);
    assert_eq!(sounds_heard(&world.bob, "minecraft:block.chest.open"), 1);
    let middle = world.bob.log.iter().find_map(|event| match event {
        ClientEvent::Sound {
            sound, position, ..
        } if *sound == "minecraft:block.chest.open" => Some(*position),
        _ => None,
    });
    assert_eq!(middle, Some(Vec3::new(3.0, 64.5, 2.5)));
}

#[test]
fn breaking_one_half_makes_the_other_a_single_chest_and_closes_the_window() {
    let mut world = world();
    world.place_double_chest();
    world.open_at(CHEST);
    world.server.handle(Inbound::Input {
        player: world.alice.id,
        input: PlayerInput::Dig {
            action: DigAction::Finish,
            pos: RIGHT_HALF,
            face: Direction::Up,
            sequence: 4,
        },
    });
    world.settle();

    assert!(closed(&world.alice, WINDOW));
    assert_eq!(
        chest_type_at(&world.alice, CHEST).as_deref(),
        Some("single")
    );
}

#[test]
fn a_blocked_partner_opens_only_the_clicked_half() {
    let mut world = world();
    world.place_double_chest();
    world.give("stone");
    world.select(0);
    world.sneak(true);
    world.use_on(RIGHT_HALF, Direction::Up);
    world.sneak(false);
    world.open_at(CHEST);

    assert_eq!(opened(&world.alice), Some((WINDOW, 2)));
    assert_eq!(content(&world.alice, WINDOW).expect("content").len(), 63);
}

#[test]
fn sneaking_keeps_two_neighbouring_chests_apart() {
    let mut world = world();
    world.give("chest");
    world.give("chest");
    world.select(0);
    world.sneak(true);
    world.use_on(FLOOR, Direction::Up);
    world.use_on(BlockPos::new(3, 63, 2), Direction::Up);
    world.sneak(false);

    assert_eq!(
        chest_type_at(&world.alice, CHEST).as_deref(),
        Some("single")
    );
    assert_eq!(
        chest_type_at(&world.alice, RIGHT_HALF).as_deref(),
        Some("single")
    );
}

fn carries_a_block_entity(player: &Player, pos: BlockPos, kind: &str) -> bool {
    player.log.iter().any(|event| match event {
        ClientEvent::Chunk { chunk, .. } => chunk
            .block_entity(pos)
            .is_some_and(|entity| entity.kind.to_string() == kind),
        _ => false,
    })
}

#[test]
fn a_container_placed_but_never_opened_is_in_the_chunk_sent_to_a_late_joiner() {
    let blocks = [
        ("chest", "minecraft:chest"),
        ("barrel", "minecraft:barrel"),
        ("furnace", "minecraft:furnace"),
        ("hopper", "minecraft:hopper"),
        ("shulker_box", "minecraft:shulker_box"),
    ];
    for (item, kind) in blocks {
        let mut world = world();
        world.place_storage(item);
        let mut carol = join(&mut world.server, "Carol");
        run_ticks(&mut world.server, &mut [&mut carol], 30);

        assert!(carries_a_block_entity(&carol, CHEST, kind), "{item}");
    }
}

#[test]
fn a_crafting_table_has_no_block_entity() {
    let mut world = world();
    world.place_storage("crafting_table");
    let mut carol = join(&mut world.server, "Carol");
    run_ticks(&mut world.server, &mut [&mut carol], 30);

    assert!(!carries_a_block_entity(
        &carol,
        CHEST,
        "minecraft:crafting_table"
    ));
}

fn ender_title(player: &Player) -> bool {
    player.log.iter().any(|event| {
        matches!(event, ClientEvent::OpenScreen { title, .. }
            if format!("{title:?}").contains("container.enderchest"))
    })
}

#[test]
fn an_ender_chest_opens_a_27_slot_window_with_its_own_title() {
    let mut world = world();
    world.place_storage("ender_chest");
    world.open_chest();

    assert_eq!(opened(&world.alice), Some((WINDOW, 2)));
    assert_eq!(content(&world.alice, WINDOW).expect("content").len(), 63);
    assert!(ender_title(&world.alice));
}

#[test]
fn the_ender_chest_items_belong_to_the_player_and_not_to_the_block() {
    let mut world = world();
    world.place_storage("ender_chest");
    world.give("stone");
    world.open_chest();
    world.click(WINDOW, HOTBAR_0, 0, ClickInput::Pickup);
    world.click(WINDOW, 4, 0, ClickInput::Pickup);
    world.close(WINDOW);
    world.open_chest_as_bob();

    let bob = content(&world.bob, WINDOW).expect("bob content");
    assert!(bob[..27].iter().all(Option::is_none));
    world.open_chest();
    let alice = content(&world.alice, 2).expect("alice content");
    assert_eq!(alice[4], Some((item("minecraft:stone"), 1)));
}

#[test]
fn the_ender_chest_lid_and_sound_follow_its_viewers() {
    let mut world = world();
    world.place_storage("ender_chest");
    world.open_chest();
    assert_eq!(lid_events(&world.bob), vec![1]);
    assert_eq!(
        sounds_heard(&world.bob, "minecraft:block.ender_chest.open"),
        1
    );

    world.close(WINDOW);
    assert_eq!(lid_events(&world.bob), vec![1, 0]);
    assert_eq!(
        sounds_heard(&world.bob, "minecraft:block.ender_chest.close"),
        1
    );
}

#[test]
fn a_solid_block_above_an_ender_chest_keeps_it_shut() {
    let mut world = world();
    world.place_storage("ender_chest");
    world.give("stone");
    world.select(0);
    world.sneak(true);
    world.use_on(CHEST, Direction::Up);
    world.sneak(false);
    world.open_chest();

    assert_eq!(opened(&world.alice), None);
}

#[test]
fn an_ender_chest_placed_but_never_opened_is_in_the_chunk_of_a_late_joiner() {
    let mut world = world();
    world.place_storage("ender_chest");
    let mut carol = join(&mut world.server, "Carol");
    run_ticks(&mut world.server, &mut [&mut carol], 30);

    assert!(carries_a_block_entity(
        &carol,
        CHEST,
        "minecraft:ender_chest"
    ));
}

const COPPER_CHESTS: [&str; 8] = [
    "copper_chest",
    "exposed_copper_chest",
    "weathered_copper_chest",
    "oxidized_copper_chest",
    "waxed_copper_chest",
    "waxed_exposed_copper_chest",
    "waxed_weathered_copper_chest",
    "waxed_oxidized_copper_chest",
];

fn kind_at(player: &Player, pos: BlockPos) -> Option<BlockKind> {
    last_update(player, pos).and_then(BlockStateId::kind)
}

impl World {
    fn place_pair_of(&mut self, first: &str, second: &str) {
        self.give(first);
        self.give(second);
        self.select(0);
        self.use_on(FLOOR, Direction::Up);
        self.select(1);
        self.use_on(BlockPos::new(3, 63, 2), Direction::Up);
    }
}

#[test]
fn every_copper_chest_opens_a_three_row_window() {
    for item in COPPER_CHESTS {
        let mut world = world();
        world.place_storage(item);
        world.open_chest();

        assert_eq!(opened(&world.alice), Some((WINDOW, 2)), "{item}");
        let slots = content(&world.alice, WINDOW).unwrap_or_else(|| panic!("{item} content"));
        assert_eq!(slots.len(), 63, "{item}");
    }
}

#[test]
fn copper_chest_items_are_kept_in_the_chest_block_entity() {
    let mut world = world();
    world.place_storage("copper_chest");
    world.give("stone");
    world.open_chest();
    world.click(WINDOW, 27 + 27, 0, ClickInput::Pickup);
    world.click(WINDOW, 4, 0, ClickInput::Pickup);
    world.close(WINDOW);
    world.open_chest();

    let slots = content(&world.alice, 2).expect("second window");
    assert_eq!(slots[4], Some((item("minecraft:stone"), 1)));
    let mut carol = join(&mut world.server, "Carol");
    run_ticks(&mut world.server, &mut [&mut carol], 30);
    assert!(carries_a_block_entity(&carol, CHEST, "minecraft:chest"));
}

#[test]
fn the_copper_chest_hinge_sound_follows_the_oxidation() {
    let sounds = [
        ("copper_chest", "copper_chest"),
        ("exposed_copper_chest", "copper_chest"),
        ("weathered_copper_chest", "copper_chest_weathered"),
        ("oxidized_copper_chest", "copper_chest_oxidized"),
        ("waxed_copper_chest", "copper_chest"),
        ("waxed_exposed_copper_chest", "copper_chest"),
        ("waxed_weathered_copper_chest", "copper_chest_weathered"),
        ("waxed_oxidized_copper_chest", "copper_chest_oxidized"),
    ];
    for (item, sound) in sounds {
        let mut world = world();
        world.place_storage(item);
        world.open_chest();
        let open = format!("minecraft:block.{sound}.open");
        let close = format!("minecraft:block.{sound}.close");
        assert_eq!(sounds_heard(&world.bob, &open), 1, "{item}");
        assert_eq!(
            sounds_heard(&world.bob, "minecraft:block.chest.open"),
            0,
            "{item}"
        );
        assert_eq!(lid_events(&world.bob), vec![1], "{item}");

        world.close(WINDOW);
        assert_eq!(sounds_heard(&world.bob, &close), 1, "{item}");
        assert_eq!(lid_events(&world.bob), vec![1, 0], "{item}");
    }
}

#[test]
fn a_solid_block_above_a_copper_chest_keeps_it_shut() {
    let mut world = world();
    world.place_storage("copper_chest");
    world.give("stone");
    world.select(0);
    world.sneak(true);
    world.use_on(CHEST, Direction::Up);
    world.sneak(false);
    assert!(last_update(&world.alice, ABOVE).is_some());
    world.open_chest();

    assert_eq!(opened(&world.alice), None);
}

#[test]
fn two_copper_chests_of_different_oxidation_make_one_double_chest_of_the_least_oxidized() {
    let mut world = world();
    world.place_pair_of("weathered_copper_chest", "exposed_copper_chest");

    for player in [&world.alice, &world.bob] {
        assert_eq!(kind_at(player, CHEST), Some(BlockKind::ExposedCopperChest));
        assert_eq!(
            kind_at(player, RIGHT_HALF),
            Some(BlockKind::ExposedCopperChest)
        );
        assert_eq!(chest_type_at(player, CHEST).as_deref(), Some("left"));
        assert_eq!(chest_type_at(player, RIGHT_HALF).as_deref(), Some("right"));
    }
    world.open_at(CHEST);
    assert_eq!(opened(&world.alice), Some((WINDOW, 5)));
    assert_eq!(content(&world.alice, WINDOW).expect("content").len(), 90);
}

#[test]
fn a_waxed_copper_chest_loses_its_wax_when_it_pairs_with_an_unwaxed_one() {
    let mut world = world();
    world.place_pair_of("waxed_copper_chest", "exposed_copper_chest");

    assert_eq!(kind_at(&world.alice, CHEST), Some(BlockKind::CopperChest));
    assert_eq!(
        kind_at(&world.alice, RIGHT_HALF),
        Some(BlockKind::CopperChest)
    );
    assert_eq!(chest_type_at(&world.alice, CHEST).as_deref(), Some("left"));
}

#[test]
fn two_waxed_copper_chests_stay_waxed() {
    let mut world = world();
    world.place_pair_of(
        "waxed_oxidized_copper_chest",
        "waxed_weathered_copper_chest",
    );

    assert_eq!(
        kind_at(&world.alice, CHEST),
        Some(BlockKind::WaxedWeatheredCopperChest)
    );
    assert_eq!(
        kind_at(&world.alice, RIGHT_HALF),
        Some(BlockKind::WaxedWeatheredCopperChest)
    );
}

#[test]
fn a_copper_chest_next_to_a_plain_chest_stays_a_single_chest() {
    let mut world = world();
    world.place_pair_of("chest", "copper_chest");

    assert_eq!(kind_at(&world.alice, CHEST), Some(BlockKind::Chest));
    assert_eq!(
        chest_type_at(&world.alice, CHEST).as_deref(),
        Some("single")
    );
    assert_eq!(
        chest_type_at(&world.alice, RIGHT_HALF).as_deref(),
        Some("single")
    );
}

#[test]
fn breaking_one_copper_half_makes_the_other_a_single_copper_chest() {
    let mut world = world();
    world.place_pair_of("copper_chest", "exposed_copper_chest");
    world.open_at(CHEST);
    world.server.handle(Inbound::Input {
        player: world.alice.id,
        input: PlayerInput::Dig {
            action: DigAction::Finish,
            pos: RIGHT_HALF,
            face: Direction::Up,
            sequence: 4,
        },
    });
    world.settle();

    assert!(closed(&world.alice, WINDOW));
    assert_eq!(kind_at(&world.alice, CHEST), Some(BlockKind::CopperChest));
    assert_eq!(
        chest_type_at(&world.alice, CHEST).as_deref(),
        Some("single")
    );
}
