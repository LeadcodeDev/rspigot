use std::sync::Mutex;

use rspigot_plugin::prelude::*;

const NAME: &str = "§6Guide";
const TITLE: &str = "§6Guide du serveur";
const INFO: u16 = 2;
const SPAWN: u16 = 4;
const CLOSE: u16 = 6;

#[derive(Plugin, Default)]
struct Npc {
    guide: Mutex<Option<i32>>,
    menu: Mutex<Option<CustomInventory>>,
}

impl Npc {
    fn is_guide(&self, entity: &Entity) -> bool {
        *self.guide.lock().unwrap() == Some(entity.entity_id)
    }

    fn menu_id(&self) -> Option<u64> {
        self.menu.lock().unwrap().as_ref().map(CustomInventory::id)
    }

    fn open_menu(&self, player: &Player) {
        if let Some(menu) = self.menu.lock().unwrap().as_ref() {
            player.play_sound(Sound::UiButtonClick, 1.0, 1.0);
            player.open_inventory(menu);
        }
    }

    async fn remove_old_guides(world: &World, at: Position) -> Result<(), Error> {
        for villager in world
            .entities
            .near(at, 8.0)
            .of(EntityType::Villager)
            .fetch()
            .await?
        {
            let state = villager.resolve().await?;
            if state.custom_name.as_deref() == Some(NAME) {
                villager.remove();
            }
        }
        Ok(())
    }

    async fn place_guide(&self, server: &Server) -> Result<(), Error> {
        let world = server.world();
        let spawn = world.resolve().await?.spawn;
        let at = Position {
            x: f64::from(spawn.x) + 2.5,
            y: f64::from(spawn.y),
            z: f64::from(spawn.z) + 0.5,
        };
        Self::remove_old_guides(&world, at).await?;
        let guide = world.spawn(EntityType::Villager, at).await?;
        guide.set_custom_name(NAME);
        guide.set_name_visible(true);
        guide.set_no_ai(true);
        guide.set_invulnerable(true);
        guide.set_silent(true);
        *self.guide.lock().unwrap() = Some(guide.entity_id);
        Ok(())
    }
}

#[listeners]
impl Npc {
    async fn on_enable(&self, server: &Server) {
        let menu = server.create_inventory(1, TITLE);
        menu.set(
            INFO,
            Some(
                ItemStack::new(Material::Book, 1)
                    .name("§eInformations")
                    .lore([
                        "§7Bienvenue sur le serveur !",
                        "§7Clique pour lire les règles.",
                    ]),
            ),
        );
        menu.set(
            SPAWN,
            Some(ItemStack::new(Material::Compass, 1).name("§aRetour au spawn")),
        );
        menu.set(
            CLOSE,
            Some(ItemStack::new(Material::Barrier, 1).name("§cFermer")),
        );
        *self.menu.lock().unwrap() = Some(menu);
        if let Err(error) = self.place_guide(server).await {
            tracing::warn!("The guide could not be placed: {error}");
        }
    }

    async fn on_entity_interact(&self, event: &EntityInteract) {
        if event.hand == Hand::Main && self.is_guide(&event.entity) {
            self.open_menu(&event.player);
        }
    }

    async fn on_inventory_click(&self, event: &InventoryClick) {
        if self.menu_id() != Some(event.inventory.id()) {
            return;
        }
        let ClickedSlot::Menu(slot) = event.slot else {
            return;
        };
        match slot {
            INFO => {
                event
                    .player
                    .send_message("§eBienvenue ! Respecte les autres joueurs.");
                event.player.close_inventory();
            }
            SPAWN => {
                event.player.close_inventory();
                if let Ok(world) = event.player.server().world().resolve().await {
                    event.player.teleport(Position {
                        x: f64::from(world.spawn.x) + 0.5,
                        y: f64::from(world.spawn.y),
                        z: f64::from(world.spawn.z) + 0.5,
                    });
                }
            }
            CLOSE => event.player.close_inventory(),
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt().init();
    rspigot_plugin::run(Npc::default()).await
}
