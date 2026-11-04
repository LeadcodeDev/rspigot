use rspigot_plugin::prelude::*;
use rspigot_plugin::{Access, HandlerSpec};

#[derive(Plugin)]
struct Everything;

#[listeners]
impl Everything {
    async fn observe_area_effect_cloud_apply(&self, event: &rspigot_plugin::AreaEffectCloudApply) {
        let _ = event.id();
    }

    async fn observe_arrow_body_count_change(&self, event: &rspigot_plugin::ArrowBodyCountChange) {
        let _ = event.id();
    }

    async fn observe_async_player_chat(&self, event: &rspigot_plugin::AsyncPlayerChat) {
        let _ = event.id();
    }

    async fn observe_async_player_pre_login(&self, event: &rspigot_plugin::AsyncPlayerPreLogin) {
        let _ = event.id();
    }

    async fn observe_async_structure_generate(
        &self,
        event: &rspigot_plugin::AsyncStructureGenerate,
    ) {
        let _ = event.id();
    }

    async fn observe_async_structure_spawn(&self, event: &rspigot_plugin::AsyncStructureSpawn) {
        let _ = event.id();
    }

    async fn observe_bat_toggle_sleep(&self, event: &rspigot_plugin::BatToggleSleep) {
        let _ = event.id();
    }

    async fn observe_bell_resonate(&self, event: &rspigot_plugin::BellResonate) {
        let _ = event.id();
    }

    async fn observe_bell_ring(&self, event: &rspigot_plugin::BellRing) {
        let _ = event.id();
    }

    async fn observe_block_break(&self, event: &rspigot_plugin::BlockBreak) {
        let _ = event.id();
    }

    async fn observe_block_brush(&self, event: &rspigot_plugin::BlockBrush) {
        let _ = event.id();
    }

    async fn observe_block_burn(&self, event: &rspigot_plugin::BlockBurn) {
        let _ = event.id();
    }

    async fn observe_block_can_build(&self, event: &rspigot_plugin::BlockCanBuild) {
        let _ = event.id();
    }

    async fn observe_block_cook(&self, event: &rspigot_plugin::BlockCook) {
        let _ = event.id();
    }

    async fn observe_block_damage(&self, event: &rspigot_plugin::BlockDamage) {
        let _ = event.id();
    }

    async fn observe_block_damage_abort(&self, event: &rspigot_plugin::BlockDamageAbort) {
        let _ = event.id();
    }

    async fn observe_block_dispense(&self, event: &rspigot_plugin::BlockDispense) {
        let _ = event.id();
    }

    async fn observe_block_dispense_armor(&self, event: &rspigot_plugin::BlockDispenseArmor) {
        let _ = event.id();
    }

    async fn observe_block_dispense_loot(&self, event: &rspigot_plugin::BlockDispenseLoot) {
        let _ = event.id();
    }

    async fn observe_block_drop_item(&self, event: &rspigot_plugin::BlockDropItem) {
        let _ = event.id();
    }

    async fn observe_block_exp(&self, event: &rspigot_plugin::BlockExp) {
        let _ = event.id();
    }

    async fn observe_block_explode(&self, event: &rspigot_plugin::BlockExplode) {
        let _ = event.id();
    }

    async fn observe_block_fade(&self, event: &rspigot_plugin::BlockFade) {
        let _ = event.id();
    }

    async fn observe_block_fertilize(&self, event: &rspigot_plugin::BlockFertilize) {
        let _ = event.id();
    }

    async fn observe_block_form(&self, event: &rspigot_plugin::BlockForm) {
        let _ = event.id();
    }

    async fn observe_block_from_to(&self, event: &rspigot_plugin::BlockFromTo) {
        let _ = event.id();
    }

    async fn observe_block_grow(&self, event: &rspigot_plugin::BlockGrow) {
        let _ = event.id();
    }

    async fn observe_block_ignite(&self, event: &rspigot_plugin::BlockIgnite) {
        let _ = event.id();
    }

    async fn observe_block_multi_place(&self, event: &rspigot_plugin::BlockMultiPlace) {
        let _ = event.id();
    }

    async fn observe_block_physics(&self, event: &rspigot_plugin::BlockPhysics) {
        let _ = event.id();
    }

    async fn observe_block_piston_extend(&self, event: &rspigot_plugin::BlockPistonExtend) {
        let _ = event.id();
    }

    async fn observe_block_piston_retract(&self, event: &rspigot_plugin::BlockPistonRetract) {
        let _ = event.id();
    }

    async fn observe_block_place(&self, event: &rspigot_plugin::BlockPlace) {
        let _ = event.id();
    }

    async fn observe_block_receive_game(&self, event: &rspigot_plugin::BlockReceiveGame) {
        let _ = event.id();
    }

    async fn observe_block_redstone(&self, event: &rspigot_plugin::BlockRedstone) {
        let _ = event.id();
    }

    async fn observe_block_shear_entity(&self, event: &rspigot_plugin::BlockShearEntity) {
        let _ = event.id();
    }

    async fn observe_block_spread(&self, event: &rspigot_plugin::BlockSpread) {
        let _ = event.id();
    }

    async fn observe_brew(&self, event: &rspigot_plugin::Brew) {
        let _ = event.id();
    }

    async fn observe_brewing_stand_fuel(&self, event: &rspigot_plugin::BrewingStandFuel) {
        let _ = event.id();
    }

    async fn observe_brewing_start(&self, event: &rspigot_plugin::BrewingStart) {
        let _ = event.id();
    }

    async fn observe_broadcast_message(&self, event: &rspigot_plugin::BroadcastMessage) {
        let _ = event.id();
    }

    async fn observe_campfire_start(&self, event: &rspigot_plugin::CampfireStart) {
        let _ = event.id();
    }

    async fn observe_cauldron_level_change(&self, event: &rspigot_plugin::CauldronLevelChange) {
        let _ = event.id();
    }

    async fn observe_chunk_load(&self, event: &rspigot_plugin::ChunkLoad) {
        let _ = event.id();
    }

    async fn observe_chunk_populate(&self, event: &rspigot_plugin::ChunkPopulate) {
        let _ = event.id();
    }

    async fn observe_chunk_unload(&self, event: &rspigot_plugin::ChunkUnload) {
        let _ = event.id();
    }

    async fn observe_craft_item(&self, event: &rspigot_plugin::CraftItem) {
        let _ = event.id();
    }

    async fn observe_crafter_craft(&self, event: &rspigot_plugin::CrafterCraft) {
        let _ = event.id();
    }

    async fn observe_creature_spawn(&self, event: &rspigot_plugin::CreatureSpawn) {
        let _ = event.id();
    }

    async fn observe_creeper_power(&self, event: &rspigot_plugin::CreeperPower) {
        let _ = event.id();
    }

    async fn observe_cube_mob_split(&self, event: &rspigot_plugin::CubeMobSplit) {
        let _ = event.id();
    }

    async fn observe_enchant_item(&self, event: &rspigot_plugin::EnchantItem) {
        let _ = event.id();
    }

    async fn observe_ender_dragon_change_phase(
        &self,
        event: &rspigot_plugin::EnderDragonChangePhase,
    ) {
        let _ = event.id();
    }

    async fn observe_entities_load(&self, event: &rspigot_plugin::EntitiesLoad) {
        let _ = event.id();
    }

    async fn observe_entities_unload(&self, event: &rspigot_plugin::EntitiesUnload) {
        let _ = event.id();
    }

    async fn observe_entity_air_change(&self, event: &rspigot_plugin::EntityAirChange) {
        let _ = event.id();
    }

    async fn observe_entity_block_form(&self, event: &rspigot_plugin::EntityBlockForm) {
        let _ = event.id();
    }

    async fn observe_entity_break_door(&self, event: &rspigot_plugin::EntityBreakDoor) {
        let _ = event.id();
    }

    async fn observe_entity_breed(&self, event: &rspigot_plugin::EntityBreed) {
        let _ = event.id();
    }

    async fn observe_entity_change_block(&self, event: &rspigot_plugin::EntityChangeBlock) {
        let _ = event.id();
    }

    async fn observe_entity_combust(&self, event: &rspigot_plugin::EntityCombust) {
        let _ = event.id();
    }

    async fn observe_entity_combust_by_block(&self, event: &rspigot_plugin::EntityCombustByBlock) {
        let _ = event.id();
    }

    async fn observe_entity_combust_by_entity(
        &self,
        event: &rspigot_plugin::EntityCombustByEntity,
    ) {
        let _ = event.id();
    }

    async fn observe_entity_damage(&self, event: &rspigot_plugin::EntityDamage) {
        let _ = event.id();
    }

    async fn observe_entity_damage_by_block(&self, event: &rspigot_plugin::EntityDamageByBlock) {
        let _ = event.id();
    }

    async fn observe_entity_damage_by_entity(&self, event: &rspigot_plugin::EntityDamageByEntity) {
        let _ = event.id();
    }

    async fn observe_entity_death(&self, event: &rspigot_plugin::EntityDeath) {
        let _ = event.id();
    }

    async fn observe_entity_dismount(&self, event: &rspigot_plugin::EntityDismount) {
        let _ = event.id();
    }

    async fn observe_entity_drop_item(&self, event: &rspigot_plugin::EntityDropItem) {
        let _ = event.id();
    }

    async fn observe_entity_enter_block(&self, event: &rspigot_plugin::EntityEnterBlock) {
        let _ = event.id();
    }

    async fn observe_entity_enter_love_mode(&self, event: &rspigot_plugin::EntityEnterLoveMode) {
        let _ = event.id();
    }

    async fn observe_entity_exhaustion(&self, event: &rspigot_plugin::EntityExhaustion) {
        let _ = event.id();
    }

    async fn observe_entity_explode(&self, event: &rspigot_plugin::EntityExplode) {
        let _ = event.id();
    }

    async fn observe_entity_interact(&self, event: &rspigot_plugin::EntityInteract) {
        let _ = event.id();
    }

    async fn observe_entity_knockback(&self, event: &rspigot_plugin::EntityKnockback) {
        let _ = event.id();
    }

    async fn observe_entity_knockback_by_entity(
        &self,
        event: &rspigot_plugin::EntityKnockbackByEntity,
    ) {
        let _ = event.id();
    }

    async fn observe_entity_mount(&self, event: &rspigot_plugin::EntityMount) {
        let _ = event.id();
    }

    async fn observe_entity_pickup_item(&self, event: &rspigot_plugin::EntityPickupItem) {
        let _ = event.id();
    }

    async fn observe_entity_place(&self, event: &rspigot_plugin::EntityPlace) {
        let _ = event.id();
    }

    async fn observe_entity_portal(&self, event: &rspigot_plugin::EntityPortal) {
        let _ = event.id();
    }

    async fn observe_entity_portal_enter(&self, event: &rspigot_plugin::EntityPortalEnter) {
        let _ = event.id();
    }

    async fn observe_entity_portal_exit(&self, event: &rspigot_plugin::EntityPortalExit) {
        let _ = event.id();
    }

    async fn observe_entity_pose_change(&self, event: &rspigot_plugin::EntityPoseChange) {
        let _ = event.id();
    }

    async fn observe_entity_potion_effect(&self, event: &rspigot_plugin::EntityPotionEffect) {
        let _ = event.id();
    }

    async fn observe_entity_regain_health(&self, event: &rspigot_plugin::EntityRegainHealth) {
        let _ = event.id();
    }

    async fn observe_entity_remove(&self, event: &rspigot_plugin::EntityRemove) {
        let _ = event.id();
    }

    async fn observe_entity_resurrect(&self, event: &rspigot_plugin::EntityResurrect) {
        let _ = event.id();
    }

    async fn observe_entity_shoot_bow(&self, event: &rspigot_plugin::EntityShootBow) {
        let _ = event.id();
    }

    async fn observe_entity_spawn(&self, event: &rspigot_plugin::EntitySpawn) {
        let _ = event.id();
    }

    async fn observe_entity_spell_cast(&self, event: &rspigot_plugin::EntitySpellCast) {
        let _ = event.id();
    }

    async fn observe_entity_tame(&self, event: &rspigot_plugin::EntityTame) {
        let _ = event.id();
    }

    async fn observe_entity_target(&self, event: &rspigot_plugin::EntityTarget) {
        let _ = event.id();
    }

    async fn observe_entity_target_block(&self, event: &rspigot_plugin::EntityTargetBlock) {
        let _ = event.id();
    }

    async fn observe_entity_target_living_entity(
        &self,
        event: &rspigot_plugin::EntityTargetLivingEntity,
    ) {
        let _ = event.id();
    }

    async fn observe_entity_teleport(&self, event: &rspigot_plugin::EntityTeleport) {
        let _ = event.id();
    }

    async fn observe_entity_toggle_glide(&self, event: &rspigot_plugin::EntityToggleGlide) {
        let _ = event.id();
    }

    async fn observe_entity_toggle_swim(&self, event: &rspigot_plugin::EntityToggleSwim) {
        let _ = event.id();
    }

    async fn observe_entity_transform(&self, event: &rspigot_plugin::EntityTransform) {
        let _ = event.id();
    }

    async fn observe_entity_unleash(&self, event: &rspigot_plugin::EntityUnleash) {
        let _ = event.id();
    }

    async fn observe_exp_bottle(&self, event: &rspigot_plugin::ExpBottle) {
        let _ = event.id();
    }

    async fn observe_explosion_prime(&self, event: &rspigot_plugin::ExplosionPrime) {
        let _ = event.id();
    }

    async fn observe_firework_explode(&self, event: &rspigot_plugin::FireworkExplode) {
        let _ = event.id();
    }

    async fn observe_fluid_level_change(&self, event: &rspigot_plugin::FluidLevelChange) {
        let _ = event.id();
    }

    async fn observe_food_level_change(&self, event: &rspigot_plugin::FoodLevelChange) {
        let _ = event.id();
    }

    async fn observe_furnace_burn(&self, event: &rspigot_plugin::FurnaceBurn) {
        let _ = event.id();
    }

    async fn observe_furnace_extract(&self, event: &rspigot_plugin::FurnaceExtract) {
        let _ = event.id();
    }

    async fn observe_furnace_smelt(&self, event: &rspigot_plugin::FurnaceSmelt) {
        let _ = event.id();
    }

    async fn observe_furnace_start_smelt(&self, event: &rspigot_plugin::FurnaceStartSmelt) {
        let _ = event.id();
    }

    async fn observe_generic_game(&self, event: &rspigot_plugin::GenericGame) {
        let _ = event.id();
    }

    async fn observe_hanging_break(&self, event: &rspigot_plugin::HangingBreak) {
        let _ = event.id();
    }

    async fn observe_hanging_break_by_entity(&self, event: &rspigot_plugin::HangingBreakByEntity) {
        let _ = event.id();
    }

    async fn observe_hanging_place(&self, event: &rspigot_plugin::HangingPlace) {
        let _ = event.id();
    }

    async fn observe_hopper_inventory_search(&self, event: &rspigot_plugin::HopperInventorySearch) {
        let _ = event.id();
    }

    async fn observe_horse_jump(&self, event: &rspigot_plugin::HorseJump) {
        let _ = event.id();
    }

    async fn observe_inventory_click(&self, event: &rspigot_plugin::InventoryClick) {
        let _ = event.id();
    }

    async fn observe_inventory_close(&self, event: &rspigot_plugin::InventoryClose) {
        let _ = event.id();
    }

    async fn observe_inventory_creative(&self, event: &rspigot_plugin::InventoryCreative) {
        let _ = event.id();
    }

    async fn observe_inventory_drag(&self, event: &rspigot_plugin::InventoryDrag) {
        let _ = event.id();
    }

    async fn observe_inventory_move_item(&self, event: &rspigot_plugin::InventoryMoveItem) {
        let _ = event.id();
    }

    async fn observe_inventory_open(&self, event: &rspigot_plugin::InventoryOpen) {
        let _ = event.id();
    }

    async fn observe_inventory_pickup_item(&self, event: &rspigot_plugin::InventoryPickupItem) {
        let _ = event.id();
    }

    async fn observe_item_despawn(&self, event: &rspigot_plugin::ItemDespawn) {
        let _ = event.id();
    }

    async fn observe_item_merge(&self, event: &rspigot_plugin::ItemMerge) {
        let _ = event.id();
    }

    async fn observe_item_spawn(&self, event: &rspigot_plugin::ItemSpawn) {
        let _ = event.id();
    }

    async fn observe_leaves_decay(&self, event: &rspigot_plugin::LeavesDecay) {
        let _ = event.id();
    }

    async fn observe_lightning_strike(&self, event: &rspigot_plugin::LightningStrike) {
        let _ = event.id();
    }

    async fn observe_lingering_potion_splash(&self, event: &rspigot_plugin::LingeringPotionSplash) {
        let _ = event.id();
    }

    async fn observe_loot_generate(&self, event: &rspigot_plugin::LootGenerate) {
        let _ = event.id();
    }

    async fn observe_map_initialize(&self, event: &rspigot_plugin::MapInitialize) {
        let _ = event.id();
    }

    async fn observe_moisture_change(&self, event: &rspigot_plugin::MoistureChange) {
        let _ = event.id();
    }

    async fn observe_note_play(&self, event: &rspigot_plugin::NotePlay) {
        let _ = event.id();
    }

    async fn observe_pig_zap(&self, event: &rspigot_plugin::PigZap) {
        let _ = event.id();
    }

    async fn observe_pig_zombie_anger(&self, event: &rspigot_plugin::PigZombieAnger) {
        let _ = event.id();
    }

    async fn observe_piglin_barter(&self, event: &rspigot_plugin::PiglinBarter) {
        let _ = event.id();
    }

    async fn observe_player_advancement_criterion_grant(
        &self,
        event: &rspigot_plugin::PlayerAdvancementCriterionGrant,
    ) {
        let _ = event.id();
    }

    async fn observe_player_advancement_done(&self, event: &rspigot_plugin::PlayerAdvancementDone) {
        let _ = event.id();
    }

    async fn observe_player_animation(&self, event: &rspigot_plugin::PlayerAnimation) {
        let _ = event.id();
    }

    async fn observe_player_armor_stand_manipulate(
        &self,
        event: &rspigot_plugin::PlayerArmorStandManipulate,
    ) {
        let _ = event.id();
    }

    async fn observe_player_bed_enter(&self, event: &rspigot_plugin::PlayerBedEnter) {
        let _ = event.id();
    }

    async fn observe_player_bed_leave(&self, event: &rspigot_plugin::PlayerBedLeave) {
        let _ = event.id();
    }

    async fn observe_player_bucket_empty(&self, event: &rspigot_plugin::PlayerBucketEmpty) {
        let _ = event.id();
    }

    async fn observe_player_bucket_entity(&self, event: &rspigot_plugin::PlayerBucketEntity) {
        let _ = event.id();
    }

    async fn observe_player_bucket_fill(&self, event: &rspigot_plugin::PlayerBucketFill) {
        let _ = event.id();
    }

    async fn observe_player_changed_main_hand(
        &self,
        event: &rspigot_plugin::PlayerChangedMainHand,
    ) {
        let _ = event.id();
    }

    async fn observe_player_changed_world(&self, event: &rspigot_plugin::PlayerChangedWorld) {
        let _ = event.id();
    }

    async fn observe_player_command_preprocess(
        &self,
        event: &rspigot_plugin::PlayerCommandPreprocess,
    ) {
        let _ = event.id();
    }

    async fn observe_player_command_send(&self, event: &rspigot_plugin::PlayerCommandSend) {
        let _ = event.id();
    }

    async fn observe_player_custom_click(&self, event: &rspigot_plugin::PlayerCustomClick) {
        let _ = event.id();
    }

    async fn observe_player_death(&self, event: &rspigot_plugin::PlayerDeath) {
        let _ = event.id();
    }

    async fn observe_player_drop_item(&self, event: &rspigot_plugin::PlayerDropItem) {
        let _ = event.id();
    }

    async fn observe_player_edit_book(&self, event: &rspigot_plugin::PlayerEditBook) {
        let _ = event.id();
    }

    async fn observe_player_egg_throw(&self, event: &rspigot_plugin::PlayerEggThrow) {
        let _ = event.id();
    }

    async fn observe_player_exp_change(&self, event: &rspigot_plugin::PlayerExpChange) {
        let _ = event.id();
    }

    async fn observe_player_exp_cooldown_change(
        &self,
        event: &rspigot_plugin::PlayerExpCooldownChange,
    ) {
        let _ = event.id();
    }

    async fn observe_player_fish(&self, event: &rspigot_plugin::PlayerFish) {
        let _ = event.id();
    }

    async fn observe_player_game_mode_change(&self, event: &rspigot_plugin::PlayerGameModeChange) {
        let _ = event.id();
    }

    async fn observe_player_harvest_block(&self, event: &rspigot_plugin::PlayerHarvestBlock) {
        let _ = event.id();
    }

    async fn observe_player_hide_entity(&self, event: &rspigot_plugin::PlayerHideEntity) {
        let _ = event.id();
    }

    async fn observe_player_input(&self, event: &rspigot_plugin::PlayerInput) {
        let _ = event.id();
    }

    async fn observe_player_interact(&self, event: &rspigot_plugin::PlayerInteract) {
        let _ = event.id();
    }

    async fn observe_player_interact_at_entity(
        &self,
        event: &rspigot_plugin::PlayerInteractAtEntity,
    ) {
        let _ = event.id();
    }

    async fn observe_player_interact_entity(&self, event: &rspigot_plugin::PlayerInteractEntity) {
        let _ = event.id();
    }

    async fn observe_player_item_break(&self, event: &rspigot_plugin::PlayerItemBreak) {
        let _ = event.id();
    }

    async fn observe_player_item_consume(&self, event: &rspigot_plugin::PlayerItemConsume) {
        let _ = event.id();
    }

    async fn observe_player_item_damage(&self, event: &rspigot_plugin::PlayerItemDamage) {
        let _ = event.id();
    }

    async fn observe_player_item_held(&self, event: &rspigot_plugin::PlayerItemHeld) {
        let _ = event.id();
    }

    async fn observe_player_item_mend(&self, event: &rspigot_plugin::PlayerItemMend) {
        let _ = event.id();
    }

    async fn observe_player_join(&self, event: &rspigot_plugin::PlayerJoin) {
        let _ = event.id();
    }

    async fn observe_player_kick(&self, event: &rspigot_plugin::PlayerKick) {
        let _ = event.id();
    }

    async fn observe_player_leash_entity(&self, event: &rspigot_plugin::PlayerLeashEntity) {
        let _ = event.id();
    }

    async fn observe_player_leave(&self, event: &rspigot_plugin::PlayerLeave) {
        let _ = event.id();
    }

    async fn observe_player_level_change(&self, event: &rspigot_plugin::PlayerLevelChange) {
        let _ = event.id();
    }

    async fn observe_player_links_send(&self, event: &rspigot_plugin::PlayerLinksSend) {
        let _ = event.id();
    }

    async fn observe_player_locale_change(&self, event: &rspigot_plugin::PlayerLocaleChange) {
        let _ = event.id();
    }

    async fn observe_player_login(&self, event: &rspigot_plugin::PlayerLogin) {
        let _ = event.id();
    }

    async fn observe_player_move(&self, event: &rspigot_plugin::PlayerMove) {
        let _ = event.id();
    }

    async fn observe_player_pickup_arrow(&self, event: &rspigot_plugin::PlayerPickupArrow) {
        let _ = event.id();
    }

    async fn observe_player_portal(&self, event: &rspigot_plugin::PlayerPortal) {
        let _ = event.id();
    }

    async fn observe_player_recipe_book_click(
        &self,
        event: &rspigot_plugin::PlayerRecipeBookClick,
    ) {
        let _ = event.id();
    }

    async fn observe_player_recipe_book_settings_change(
        &self,
        event: &rspigot_plugin::PlayerRecipeBookSettingsChange,
    ) {
        let _ = event.id();
    }

    async fn observe_player_recipe_discover(&self, event: &rspigot_plugin::PlayerRecipeDiscover) {
        let _ = event.id();
    }

    async fn observe_player_resource_pack_status(
        &self,
        event: &rspigot_plugin::PlayerResourcePackStatus,
    ) {
        let _ = event.id();
    }

    async fn observe_player_respawn(&self, event: &rspigot_plugin::PlayerRespawn) {
        let _ = event.id();
    }

    async fn observe_player_riptide(&self, event: &rspigot_plugin::PlayerRiptide) {
        let _ = event.id();
    }

    async fn observe_player_shear_entity(&self, event: &rspigot_plugin::PlayerShearEntity) {
        let _ = event.id();
    }

    async fn observe_player_show_entity(&self, event: &rspigot_plugin::PlayerShowEntity) {
        let _ = event.id();
    }

    async fn observe_player_sign_open(&self, event: &rspigot_plugin::PlayerSignOpen) {
        let _ = event.id();
    }

    async fn observe_player_spawn_change(&self, event: &rspigot_plugin::PlayerSpawnChange) {
        let _ = event.id();
    }

    async fn observe_player_statistic_increment(
        &self,
        event: &rspigot_plugin::PlayerStatisticIncrement,
    ) {
        let _ = event.id();
    }

    async fn observe_player_swap_hand_items(&self, event: &rspigot_plugin::PlayerSwapHandItems) {
        let _ = event.id();
    }

    async fn observe_player_take_lectern_book(
        &self,
        event: &rspigot_plugin::PlayerTakeLecternBook,
    ) {
        let _ = event.id();
    }

    async fn observe_player_teleport(&self, event: &rspigot_plugin::PlayerTeleport) {
        let _ = event.id();
    }

    async fn observe_player_toggle_flight(&self, event: &rspigot_plugin::PlayerToggleFlight) {
        let _ = event.id();
    }

    async fn observe_player_toggle_sneak(&self, event: &rspigot_plugin::PlayerToggleSneak) {
        let _ = event.id();
    }

    async fn observe_player_toggle_sprint(&self, event: &rspigot_plugin::PlayerToggleSprint) {
        let _ = event.id();
    }

    async fn observe_player_unleash_entity(&self, event: &rspigot_plugin::PlayerUnleashEntity) {
        let _ = event.id();
    }

    async fn observe_player_velocity(&self, event: &rspigot_plugin::PlayerVelocity) {
        let _ = event.id();
    }

    async fn observe_plugin_disable(&self, event: &rspigot_plugin::PluginDisable) {
        let _ = event.id();
    }

    async fn observe_plugin_enable(&self, event: &rspigot_plugin::PluginEnable) {
        let _ = event.id();
    }

    async fn observe_portal_create(&self, event: &rspigot_plugin::PortalCreate) {
        let _ = event.id();
    }

    async fn observe_potion_splash(&self, event: &rspigot_plugin::PotionSplash) {
        let _ = event.id();
    }

    async fn observe_prepare_anvil(&self, event: &rspigot_plugin::PrepareAnvil) {
        let _ = event.id();
    }

    async fn observe_prepare_grindstone(&self, event: &rspigot_plugin::PrepareGrindstone) {
        let _ = event.id();
    }

    async fn observe_prepare_item_craft(&self, event: &rspigot_plugin::PrepareItemCraft) {
        let _ = event.id();
    }

    async fn observe_prepare_item_enchant(&self, event: &rspigot_plugin::PrepareItemEnchant) {
        let _ = event.id();
    }

    async fn observe_prepare_smithing(&self, event: &rspigot_plugin::PrepareSmithing) {
        let _ = event.id();
    }

    async fn observe_projectile_hit(&self, event: &rspigot_plugin::ProjectileHit) {
        let _ = event.id();
    }

    async fn observe_projectile_launch(&self, event: &rspigot_plugin::ProjectileLaunch) {
        let _ = event.id();
    }

    async fn observe_raid_finish(&self, event: &rspigot_plugin::RaidFinish) {
        let _ = event.id();
    }

    async fn observe_raid_spawn_wave(&self, event: &rspigot_plugin::RaidSpawnWave) {
        let _ = event.id();
    }

    async fn observe_raid_stop(&self, event: &rspigot_plugin::RaidStop) {
        let _ = event.id();
    }

    async fn observe_raid_trigger(&self, event: &rspigot_plugin::RaidTrigger) {
        let _ = event.id();
    }

    async fn observe_remote_server_command(&self, event: &rspigot_plugin::RemoteServerCommand) {
        let _ = event.id();
    }

    async fn observe_sculk_bloom(&self, event: &rspigot_plugin::SculkBloom) {
        let _ = event.id();
    }

    async fn observe_server_command(&self, event: &rspigot_plugin::ServerCommand) {
        let _ = event.id();
    }

    async fn observe_server_list_ping(&self, event: &rspigot_plugin::ServerListPing) {
        let _ = event.id();
    }

    async fn observe_server_load(&self, event: &rspigot_plugin::ServerLoad) {
        let _ = event.id();
    }

    async fn observe_sheep_dye_wool(&self, event: &rspigot_plugin::SheepDyeWool) {
        let _ = event.id();
    }

    async fn observe_sheep_regrow_wool(&self, event: &rspigot_plugin::SheepRegrowWool) {
        let _ = event.id();
    }

    async fn observe_sign_change(&self, event: &rspigot_plugin::SignChange) {
        let _ = event.id();
    }

    async fn observe_smith_item(&self, event: &rspigot_plugin::SmithItem) {
        let _ = event.id();
    }

    async fn observe_spawn_change(&self, event: &rspigot_plugin::SpawnChange) {
        let _ = event.id();
    }

    async fn observe_spawner_spawn(&self, event: &rspigot_plugin::SpawnerSpawn) {
        let _ = event.id();
    }

    async fn observe_sponge_absorb(&self, event: &rspigot_plugin::SpongeAbsorb) {
        let _ = event.id();
    }

    async fn observe_strider_temperature_change(
        &self,
        event: &rspigot_plugin::StriderTemperatureChange,
    ) {
        let _ = event.id();
    }

    async fn observe_structure_grow(&self, event: &rspigot_plugin::StructureGrow) {
        let _ = event.id();
    }

    async fn observe_tnt_prime(&self, event: &rspigot_plugin::TNTPrime) {
        let _ = event.id();
    }

    async fn observe_tab_complete(&self, event: &rspigot_plugin::TabComplete) {
        let _ = event.id();
    }

    async fn observe_thunder_change(&self, event: &rspigot_plugin::ThunderChange) {
        let _ = event.id();
    }

    async fn observe_time_skip(&self, event: &rspigot_plugin::TimeSkip) {
        let _ = event.id();
    }

    async fn observe_trade_select(&self, event: &rspigot_plugin::TradeSelect) {
        let _ = event.id();
    }

    async fn observe_trial_spawner_spawn(&self, event: &rspigot_plugin::TrialSpawnerSpawn) {
        let _ = event.id();
    }

    async fn observe_vault_display_item(&self, event: &rspigot_plugin::VaultDisplayItem) {
        let _ = event.id();
    }

    async fn observe_vehicle_block_collision(&self, event: &rspigot_plugin::VehicleBlockCollision) {
        let _ = event.id();
    }

    async fn observe_vehicle_create(&self, event: &rspigot_plugin::VehicleCreate) {
        let _ = event.id();
    }

    async fn observe_vehicle_damage(&self, event: &rspigot_plugin::VehicleDamage) {
        let _ = event.id();
    }

    async fn observe_vehicle_destroy(&self, event: &rspigot_plugin::VehicleDestroy) {
        let _ = event.id();
    }

    async fn observe_vehicle_enter(&self, event: &rspigot_plugin::VehicleEnter) {
        let _ = event.id();
    }

    async fn observe_vehicle_entity_collision(
        &self,
        event: &rspigot_plugin::VehicleEntityCollision,
    ) {
        let _ = event.id();
    }

    async fn observe_vehicle_exit(&self, event: &rspigot_plugin::VehicleExit) {
        let _ = event.id();
    }

    async fn observe_vehicle_move(&self, event: &rspigot_plugin::VehicleMove) {
        let _ = event.id();
    }

    async fn observe_vehicle_update(&self, event: &rspigot_plugin::VehicleUpdate) {
        let _ = event.id();
    }

    async fn observe_villager_acquire_trade(&self, event: &rspigot_plugin::VillagerAcquireTrade) {
        let _ = event.id();
    }

    async fn observe_villager_career_change(&self, event: &rspigot_plugin::VillagerCareerChange) {
        let _ = event.id();
    }

    async fn observe_villager_replenish_trade(
        &self,
        event: &rspigot_plugin::VillagerReplenishTrade,
    ) {
        let _ = event.id();
    }

    async fn observe_villager_reputation_change(
        &self,
        event: &rspigot_plugin::VillagerReputationChange,
    ) {
        let _ = event.id();
    }

    async fn observe_weather_change(&self, event: &rspigot_plugin::WeatherChange) {
        let _ = event.id();
    }

    async fn observe_world_init(&self, event: &rspigot_plugin::WorldInit) {
        let _ = event.id();
    }

    async fn observe_world_load(&self, event: &rspigot_plugin::WorldLoad) {
        let _ = event.id();
    }

    async fn observe_world_save(&self, event: &rspigot_plugin::WorldSave) {
        let _ = event.id();
    }

    async fn observe_world_unload(&self, event: &rspigot_plugin::WorldUnload) {
        let _ = event.id();
    }

    async fn decide_area_effect_cloud_apply(
        &self,
        event: &mut rspigot_plugin::AreaEffectCloudApply,
    ) {
        let _ = event.id();
    }

    async fn decide_arrow_body_count_change(
        &self,
        event: &mut rspigot_plugin::ArrowBodyCountChange,
    ) {
        let _ = event.id();
    }

    async fn decide_async_player_chat(&self, event: &mut rspigot_plugin::AsyncPlayerChat) {
        let _ = event.id();
    }

    async fn decide_async_player_pre_login(&self, event: &mut rspigot_plugin::AsyncPlayerPreLogin) {
        let _ = event.id();
    }

    async fn decide_async_structure_spawn(&self, event: &mut rspigot_plugin::AsyncStructureSpawn) {
        let _ = event.id();
    }

    async fn decide_bat_toggle_sleep(&self, event: &mut rspigot_plugin::BatToggleSleep) {
        let _ = event.id();
    }

    async fn decide_bell_ring(&self, event: &mut rspigot_plugin::BellRing) {
        let _ = event.id();
    }

    async fn decide_block_break(&self, event: &mut rspigot_plugin::BlockBreak) {
        let _ = event.id();
    }

    async fn decide_block_brush(&self, event: &mut rspigot_plugin::BlockBrush) {
        let _ = event.id();
    }

    async fn decide_block_burn(&self, event: &mut rspigot_plugin::BlockBurn) {
        let _ = event.id();
    }

    async fn decide_block_can_build(&self, event: &mut rspigot_plugin::BlockCanBuild) {
        let _ = event.id();
    }

    async fn decide_block_cook(&self, event: &mut rspigot_plugin::BlockCook) {
        let _ = event.id();
    }

    async fn decide_block_damage(&self, event: &mut rspigot_plugin::BlockDamage) {
        let _ = event.id();
    }

    async fn decide_block_dispense(&self, event: &mut rspigot_plugin::BlockDispense) {
        let _ = event.id();
    }

    async fn decide_block_dispense_armor(&self, event: &mut rspigot_plugin::BlockDispenseArmor) {
        let _ = event.id();
    }

    async fn decide_block_dispense_loot(&self, event: &mut rspigot_plugin::BlockDispenseLoot) {
        let _ = event.id();
    }

    async fn decide_block_drop_item(&self, event: &mut rspigot_plugin::BlockDropItem) {
        let _ = event.id();
    }

    async fn decide_block_exp(&self, event: &mut rspigot_plugin::BlockExp) {
        let _ = event.id();
    }

    async fn decide_block_explode(&self, event: &mut rspigot_plugin::BlockExplode) {
        let _ = event.id();
    }

    async fn decide_block_fade(&self, event: &mut rspigot_plugin::BlockFade) {
        let _ = event.id();
    }

    async fn decide_block_fertilize(&self, event: &mut rspigot_plugin::BlockFertilize) {
        let _ = event.id();
    }

    async fn decide_block_form(&self, event: &mut rspigot_plugin::BlockForm) {
        let _ = event.id();
    }

    async fn decide_block_from_to(&self, event: &mut rspigot_plugin::BlockFromTo) {
        let _ = event.id();
    }

    async fn decide_block_grow(&self, event: &mut rspigot_plugin::BlockGrow) {
        let _ = event.id();
    }

    async fn decide_block_ignite(&self, event: &mut rspigot_plugin::BlockIgnite) {
        let _ = event.id();
    }

    async fn decide_block_multi_place(&self, event: &mut rspigot_plugin::BlockMultiPlace) {
        let _ = event.id();
    }

    async fn decide_block_physics(&self, event: &mut rspigot_plugin::BlockPhysics) {
        let _ = event.id();
    }

    async fn decide_block_piston_extend(&self, event: &mut rspigot_plugin::BlockPistonExtend) {
        let _ = event.id();
    }

    async fn decide_block_piston_retract(&self, event: &mut rspigot_plugin::BlockPistonRetract) {
        let _ = event.id();
    }

    async fn decide_block_place(&self, event: &mut rspigot_plugin::BlockPlace) {
        let _ = event.id();
    }

    async fn decide_block_receive_game(&self, event: &mut rspigot_plugin::BlockReceiveGame) {
        let _ = event.id();
    }

    async fn decide_block_redstone(&self, event: &mut rspigot_plugin::BlockRedstone) {
        let _ = event.id();
    }

    async fn decide_block_shear_entity(&self, event: &mut rspigot_plugin::BlockShearEntity) {
        let _ = event.id();
    }

    async fn decide_block_spread(&self, event: &mut rspigot_plugin::BlockSpread) {
        let _ = event.id();
    }

    async fn decide_brew(&self, event: &mut rspigot_plugin::Brew) {
        let _ = event.id();
    }

    async fn decide_brewing_stand_fuel(&self, event: &mut rspigot_plugin::BrewingStandFuel) {
        let _ = event.id();
    }

    async fn decide_brewing_start(&self, event: &mut rspigot_plugin::BrewingStart) {
        let _ = event.id();
    }

    async fn decide_broadcast_message(&self, event: &mut rspigot_plugin::BroadcastMessage) {
        let _ = event.id();
    }

    async fn decide_campfire_start(&self, event: &mut rspigot_plugin::CampfireStart) {
        let _ = event.id();
    }

    async fn decide_cauldron_level_change(&self, event: &mut rspigot_plugin::CauldronLevelChange) {
        let _ = event.id();
    }

    async fn decide_chunk_unload(&self, event: &mut rspigot_plugin::ChunkUnload) {
        let _ = event.id();
    }

    async fn decide_craft_item(&self, event: &mut rspigot_plugin::CraftItem) {
        let _ = event.id();
    }

    async fn decide_crafter_craft(&self, event: &mut rspigot_plugin::CrafterCraft) {
        let _ = event.id();
    }

    async fn decide_creature_spawn(&self, event: &mut rspigot_plugin::CreatureSpawn) {
        let _ = event.id();
    }

    async fn decide_creeper_power(&self, event: &mut rspigot_plugin::CreeperPower) {
        let _ = event.id();
    }

    async fn decide_cube_mob_split(&self, event: &mut rspigot_plugin::CubeMobSplit) {
        let _ = event.id();
    }

    async fn decide_enchant_item(&self, event: &mut rspigot_plugin::EnchantItem) {
        let _ = event.id();
    }

    async fn decide_ender_dragon_change_phase(
        &self,
        event: &mut rspigot_plugin::EnderDragonChangePhase,
    ) {
        let _ = event.id();
    }

    async fn decide_entity_air_change(&self, event: &mut rspigot_plugin::EntityAirChange) {
        let _ = event.id();
    }

    async fn decide_entity_block_form(&self, event: &mut rspigot_plugin::EntityBlockForm) {
        let _ = event.id();
    }

    async fn decide_entity_break_door(&self, event: &mut rspigot_plugin::EntityBreakDoor) {
        let _ = event.id();
    }

    async fn decide_entity_breed(&self, event: &mut rspigot_plugin::EntityBreed) {
        let _ = event.id();
    }

    async fn decide_entity_change_block(&self, event: &mut rspigot_plugin::EntityChangeBlock) {
        let _ = event.id();
    }

    async fn decide_entity_combust(&self, event: &mut rspigot_plugin::EntityCombust) {
        let _ = event.id();
    }

    async fn decide_entity_combust_by_block(
        &self,
        event: &mut rspigot_plugin::EntityCombustByBlock,
    ) {
        let _ = event.id();
    }

    async fn decide_entity_combust_by_entity(
        &self,
        event: &mut rspigot_plugin::EntityCombustByEntity,
    ) {
        let _ = event.id();
    }

    async fn decide_entity_damage(&self, event: &mut rspigot_plugin::EntityDamage) {
        let _ = event.id();
    }

    async fn decide_entity_damage_by_block(&self, event: &mut rspigot_plugin::EntityDamageByBlock) {
        let _ = event.id();
    }

    async fn decide_entity_damage_by_entity(
        &self,
        event: &mut rspigot_plugin::EntityDamageByEntity,
    ) {
        let _ = event.id();
    }

    async fn decide_entity_death(&self, event: &mut rspigot_plugin::EntityDeath) {
        let _ = event.id();
    }

    async fn decide_entity_dismount(&self, event: &mut rspigot_plugin::EntityDismount) {
        let _ = event.id();
    }

    async fn decide_entity_drop_item(&self, event: &mut rspigot_plugin::EntityDropItem) {
        let _ = event.id();
    }

    async fn decide_entity_enter_block(&self, event: &mut rspigot_plugin::EntityEnterBlock) {
        let _ = event.id();
    }

    async fn decide_entity_enter_love_mode(&self, event: &mut rspigot_plugin::EntityEnterLoveMode) {
        let _ = event.id();
    }

    async fn decide_entity_exhaustion(&self, event: &mut rspigot_plugin::EntityExhaustion) {
        let _ = event.id();
    }

    async fn decide_entity_explode(&self, event: &mut rspigot_plugin::EntityExplode) {
        let _ = event.id();
    }

    async fn decide_entity_interact(&self, event: &mut rspigot_plugin::EntityInteract) {
        let _ = event.id();
    }

    async fn decide_entity_knockback(&self, event: &mut rspigot_plugin::EntityKnockback) {
        let _ = event.id();
    }

    async fn decide_entity_knockback_by_entity(
        &self,
        event: &mut rspigot_plugin::EntityKnockbackByEntity,
    ) {
        let _ = event.id();
    }

    async fn decide_entity_mount(&self, event: &mut rspigot_plugin::EntityMount) {
        let _ = event.id();
    }

    async fn decide_entity_pickup_item(&self, event: &mut rspigot_plugin::EntityPickupItem) {
        let _ = event.id();
    }

    async fn decide_entity_place(&self, event: &mut rspigot_plugin::EntityPlace) {
        let _ = event.id();
    }

    async fn decide_entity_portal(&self, event: &mut rspigot_plugin::EntityPortal) {
        let _ = event.id();
    }

    async fn decide_entity_portal_exit(&self, event: &mut rspigot_plugin::EntityPortalExit) {
        let _ = event.id();
    }

    async fn decide_entity_potion_effect(&self, event: &mut rspigot_plugin::EntityPotionEffect) {
        let _ = event.id();
    }

    async fn decide_entity_regain_health(&self, event: &mut rspigot_plugin::EntityRegainHealth) {
        let _ = event.id();
    }

    async fn decide_entity_resurrect(&self, event: &mut rspigot_plugin::EntityResurrect) {
        let _ = event.id();
    }

    async fn decide_entity_shoot_bow(&self, event: &mut rspigot_plugin::EntityShootBow) {
        let _ = event.id();
    }

    async fn decide_entity_spawn(&self, event: &mut rspigot_plugin::EntitySpawn) {
        let _ = event.id();
    }

    async fn decide_entity_spell_cast(&self, event: &mut rspigot_plugin::EntitySpellCast) {
        let _ = event.id();
    }

    async fn decide_entity_tame(&self, event: &mut rspigot_plugin::EntityTame) {
        let _ = event.id();
    }

    async fn decide_entity_target(&self, event: &mut rspigot_plugin::EntityTarget) {
        let _ = event.id();
    }

    async fn decide_entity_target_block(&self, event: &mut rspigot_plugin::EntityTargetBlock) {
        let _ = event.id();
    }

    async fn decide_entity_target_living_entity(
        &self,
        event: &mut rspigot_plugin::EntityTargetLivingEntity,
    ) {
        let _ = event.id();
    }

    async fn decide_entity_teleport(&self, event: &mut rspigot_plugin::EntityTeleport) {
        let _ = event.id();
    }

    async fn decide_entity_toggle_glide(&self, event: &mut rspigot_plugin::EntityToggleGlide) {
        let _ = event.id();
    }

    async fn decide_entity_toggle_swim(&self, event: &mut rspigot_plugin::EntityToggleSwim) {
        let _ = event.id();
    }

    async fn decide_entity_transform(&self, event: &mut rspigot_plugin::EntityTransform) {
        let _ = event.id();
    }

    async fn decide_exp_bottle(&self, event: &mut rspigot_plugin::ExpBottle) {
        let _ = event.id();
    }

    async fn decide_explosion_prime(&self, event: &mut rspigot_plugin::ExplosionPrime) {
        let _ = event.id();
    }

    async fn decide_firework_explode(&self, event: &mut rspigot_plugin::FireworkExplode) {
        let _ = event.id();
    }

    async fn decide_fluid_level_change(&self, event: &mut rspigot_plugin::FluidLevelChange) {
        let _ = event.id();
    }

    async fn decide_food_level_change(&self, event: &mut rspigot_plugin::FoodLevelChange) {
        let _ = event.id();
    }

    async fn decide_furnace_burn(&self, event: &mut rspigot_plugin::FurnaceBurn) {
        let _ = event.id();
    }

    async fn decide_furnace_extract(&self, event: &mut rspigot_plugin::FurnaceExtract) {
        let _ = event.id();
    }

    async fn decide_furnace_smelt(&self, event: &mut rspigot_plugin::FurnaceSmelt) {
        let _ = event.id();
    }

    async fn decide_furnace_start_smelt(&self, event: &mut rspigot_plugin::FurnaceStartSmelt) {
        let _ = event.id();
    }

    async fn decide_generic_game(&self, event: &mut rspigot_plugin::GenericGame) {
        let _ = event.id();
    }

    async fn decide_hanging_break(&self, event: &mut rspigot_plugin::HangingBreak) {
        let _ = event.id();
    }

    async fn decide_hanging_break_by_entity(
        &self,
        event: &mut rspigot_plugin::HangingBreakByEntity,
    ) {
        let _ = event.id();
    }

    async fn decide_hanging_place(&self, event: &mut rspigot_plugin::HangingPlace) {
        let _ = event.id();
    }

    async fn decide_hopper_inventory_search(
        &self,
        event: &mut rspigot_plugin::HopperInventorySearch,
    ) {
        let _ = event.id();
    }

    async fn decide_horse_jump(&self, event: &mut rspigot_plugin::HorseJump) {
        let _ = event.id();
    }

    async fn decide_inventory_click(&self, event: &mut rspigot_plugin::InventoryClick) {
        let _ = event.id();
    }

    async fn decide_inventory_creative(&self, event: &mut rspigot_plugin::InventoryCreative) {
        let _ = event.id();
    }

    async fn decide_inventory_drag(&self, event: &mut rspigot_plugin::InventoryDrag) {
        let _ = event.id();
    }

    async fn decide_inventory_move_item(&self, event: &mut rspigot_plugin::InventoryMoveItem) {
        let _ = event.id();
    }

    async fn decide_inventory_open(&self, event: &mut rspigot_plugin::InventoryOpen) {
        let _ = event.id();
    }

    async fn decide_inventory_pickup_item(&self, event: &mut rspigot_plugin::InventoryPickupItem) {
        let _ = event.id();
    }

    async fn decide_item_despawn(&self, event: &mut rspigot_plugin::ItemDespawn) {
        let _ = event.id();
    }

    async fn decide_item_merge(&self, event: &mut rspigot_plugin::ItemMerge) {
        let _ = event.id();
    }

    async fn decide_item_spawn(&self, event: &mut rspigot_plugin::ItemSpawn) {
        let _ = event.id();
    }

    async fn decide_leaves_decay(&self, event: &mut rspigot_plugin::LeavesDecay) {
        let _ = event.id();
    }

    async fn decide_lightning_strike(&self, event: &mut rspigot_plugin::LightningStrike) {
        let _ = event.id();
    }

    async fn decide_lingering_potion_splash(
        &self,
        event: &mut rspigot_plugin::LingeringPotionSplash,
    ) {
        let _ = event.id();
    }

    async fn decide_loot_generate(&self, event: &mut rspigot_plugin::LootGenerate) {
        let _ = event.id();
    }

    async fn decide_moisture_change(&self, event: &mut rspigot_plugin::MoistureChange) {
        let _ = event.id();
    }

    async fn decide_note_play(&self, event: &mut rspigot_plugin::NotePlay) {
        let _ = event.id();
    }

    async fn decide_pig_zap(&self, event: &mut rspigot_plugin::PigZap) {
        let _ = event.id();
    }

    async fn decide_pig_zombie_anger(&self, event: &mut rspigot_plugin::PigZombieAnger) {
        let _ = event.id();
    }

    async fn decide_piglin_barter(&self, event: &mut rspigot_plugin::PiglinBarter) {
        let _ = event.id();
    }

    async fn decide_player_advancement_criterion_grant(
        &self,
        event: &mut rspigot_plugin::PlayerAdvancementCriterionGrant,
    ) {
        let _ = event.id();
    }

    async fn decide_player_animation(&self, event: &mut rspigot_plugin::PlayerAnimation) {
        let _ = event.id();
    }

    async fn decide_player_armor_stand_manipulate(
        &self,
        event: &mut rspigot_plugin::PlayerArmorStandManipulate,
    ) {
        let _ = event.id();
    }

    async fn decide_player_bed_enter(&self, event: &mut rspigot_plugin::PlayerBedEnter) {
        let _ = event.id();
    }

    async fn decide_player_bed_leave(&self, event: &mut rspigot_plugin::PlayerBedLeave) {
        let _ = event.id();
    }

    async fn decide_player_bucket_empty(&self, event: &mut rspigot_plugin::PlayerBucketEmpty) {
        let _ = event.id();
    }

    async fn decide_player_bucket_entity(&self, event: &mut rspigot_plugin::PlayerBucketEntity) {
        let _ = event.id();
    }

    async fn decide_player_bucket_fill(&self, event: &mut rspigot_plugin::PlayerBucketFill) {
        let _ = event.id();
    }

    async fn decide_player_command_preprocess(
        &self,
        event: &mut rspigot_plugin::PlayerCommandPreprocess,
    ) {
        let _ = event.id();
    }

    async fn decide_player_command_send(&self, event: &mut rspigot_plugin::PlayerCommandSend) {
        let _ = event.id();
    }

    async fn decide_player_death(&self, event: &mut rspigot_plugin::PlayerDeath) {
        let _ = event.id();
    }

    async fn decide_player_drop_item(&self, event: &mut rspigot_plugin::PlayerDropItem) {
        let _ = event.id();
    }

    async fn decide_player_edit_book(&self, event: &mut rspigot_plugin::PlayerEditBook) {
        let _ = event.id();
    }

    async fn decide_player_egg_throw(&self, event: &mut rspigot_plugin::PlayerEggThrow) {
        let _ = event.id();
    }

    async fn decide_player_exp_change(&self, event: &mut rspigot_plugin::PlayerExpChange) {
        let _ = event.id();
    }

    async fn decide_player_exp_cooldown_change(
        &self,
        event: &mut rspigot_plugin::PlayerExpCooldownChange,
    ) {
        let _ = event.id();
    }

    async fn decide_player_fish(&self, event: &mut rspigot_plugin::PlayerFish) {
        let _ = event.id();
    }

    async fn decide_player_game_mode_change(
        &self,
        event: &mut rspigot_plugin::PlayerGameModeChange,
    ) {
        let _ = event.id();
    }

    async fn decide_player_harvest_block(&self, event: &mut rspigot_plugin::PlayerHarvestBlock) {
        let _ = event.id();
    }

    async fn decide_player_interact(&self, event: &mut rspigot_plugin::PlayerInteract) {
        let _ = event.id();
    }

    async fn decide_player_interact_at_entity(
        &self,
        event: &mut rspigot_plugin::PlayerInteractAtEntity,
    ) {
        let _ = event.id();
    }

    async fn decide_player_interact_entity(
        &self,
        event: &mut rspigot_plugin::PlayerInteractEntity,
    ) {
        let _ = event.id();
    }

    async fn decide_player_item_consume(&self, event: &mut rspigot_plugin::PlayerItemConsume) {
        let _ = event.id();
    }

    async fn decide_player_item_damage(&self, event: &mut rspigot_plugin::PlayerItemDamage) {
        let _ = event.id();
    }

    async fn decide_player_item_held(&self, event: &mut rspigot_plugin::PlayerItemHeld) {
        let _ = event.id();
    }

    async fn decide_player_item_mend(&self, event: &mut rspigot_plugin::PlayerItemMend) {
        let _ = event.id();
    }

    async fn decide_player_join(&self, event: &mut rspigot_plugin::PlayerJoin) {
        let _ = event.id();
    }

    async fn decide_player_kick(&self, event: &mut rspigot_plugin::PlayerKick) {
        let _ = event.id();
    }

    async fn decide_player_leash_entity(&self, event: &mut rspigot_plugin::PlayerLeashEntity) {
        let _ = event.id();
    }

    async fn decide_player_leave(&self, event: &mut rspigot_plugin::PlayerLeave) {
        let _ = event.id();
    }

    async fn decide_player_login(&self, event: &mut rspigot_plugin::PlayerLogin) {
        let _ = event.id();
    }

    async fn decide_player_move(&self, event: &mut rspigot_plugin::PlayerMove) {
        let _ = event.id();
    }

    async fn decide_player_pickup_arrow(&self, event: &mut rspigot_plugin::PlayerPickupArrow) {
        let _ = event.id();
    }

    async fn decide_player_portal(&self, event: &mut rspigot_plugin::PlayerPortal) {
        let _ = event.id();
    }

    async fn decide_player_recipe_book_click(
        &self,
        event: &mut rspigot_plugin::PlayerRecipeBookClick,
    ) {
        let _ = event.id();
    }

    async fn decide_player_recipe_discover(
        &self,
        event: &mut rspigot_plugin::PlayerRecipeDiscover,
    ) {
        let _ = event.id();
    }

    async fn decide_player_respawn(&self, event: &mut rspigot_plugin::PlayerRespawn) {
        let _ = event.id();
    }

    async fn decide_player_shear_entity(&self, event: &mut rspigot_plugin::PlayerShearEntity) {
        let _ = event.id();
    }

    async fn decide_player_sign_open(&self, event: &mut rspigot_plugin::PlayerSignOpen) {
        let _ = event.id();
    }

    async fn decide_player_spawn_change(&self, event: &mut rspigot_plugin::PlayerSpawnChange) {
        let _ = event.id();
    }

    async fn decide_player_statistic_increment(
        &self,
        event: &mut rspigot_plugin::PlayerStatisticIncrement,
    ) {
        let _ = event.id();
    }

    async fn decide_player_swap_hand_items(&self, event: &mut rspigot_plugin::PlayerSwapHandItems) {
        let _ = event.id();
    }

    async fn decide_player_take_lectern_book(
        &self,
        event: &mut rspigot_plugin::PlayerTakeLecternBook,
    ) {
        let _ = event.id();
    }

    async fn decide_player_teleport(&self, event: &mut rspigot_plugin::PlayerTeleport) {
        let _ = event.id();
    }

    async fn decide_player_toggle_flight(&self, event: &mut rspigot_plugin::PlayerToggleFlight) {
        let _ = event.id();
    }

    async fn decide_player_toggle_sneak(&self, event: &mut rspigot_plugin::PlayerToggleSneak) {
        let _ = event.id();
    }

    async fn decide_player_toggle_sprint(&self, event: &mut rspigot_plugin::PlayerToggleSprint) {
        let _ = event.id();
    }

    async fn decide_player_unleash_entity(&self, event: &mut rspigot_plugin::PlayerUnleashEntity) {
        let _ = event.id();
    }

    async fn decide_player_velocity(&self, event: &mut rspigot_plugin::PlayerVelocity) {
        let _ = event.id();
    }

    async fn decide_portal_create(&self, event: &mut rspigot_plugin::PortalCreate) {
        let _ = event.id();
    }

    async fn decide_potion_splash(&self, event: &mut rspigot_plugin::PotionSplash) {
        let _ = event.id();
    }

    async fn decide_prepare_anvil(&self, event: &mut rspigot_plugin::PrepareAnvil) {
        let _ = event.id();
    }

    async fn decide_prepare_grindstone(&self, event: &mut rspigot_plugin::PrepareGrindstone) {
        let _ = event.id();
    }

    async fn decide_prepare_item_craft(&self, event: &mut rspigot_plugin::PrepareItemCraft) {
        let _ = event.id();
    }

    async fn decide_prepare_item_enchant(&self, event: &mut rspigot_plugin::PrepareItemEnchant) {
        let _ = event.id();
    }

    async fn decide_prepare_smithing(&self, event: &mut rspigot_plugin::PrepareSmithing) {
        let _ = event.id();
    }

    async fn decide_projectile_hit(&self, event: &mut rspigot_plugin::ProjectileHit) {
        let _ = event.id();
    }

    async fn decide_projectile_launch(&self, event: &mut rspigot_plugin::ProjectileLaunch) {
        let _ = event.id();
    }

    async fn decide_raid_trigger(&self, event: &mut rspigot_plugin::RaidTrigger) {
        let _ = event.id();
    }

    async fn decide_remote_server_command(&self, event: &mut rspigot_plugin::RemoteServerCommand) {
        let _ = event.id();
    }

    async fn decide_sculk_bloom(&self, event: &mut rspigot_plugin::SculkBloom) {
        let _ = event.id();
    }

    async fn decide_server_command(&self, event: &mut rspigot_plugin::ServerCommand) {
        let _ = event.id();
    }

    async fn decide_server_list_ping(&self, event: &mut rspigot_plugin::ServerListPing) {
        let _ = event.id();
    }

    async fn decide_sheep_dye_wool(&self, event: &mut rspigot_plugin::SheepDyeWool) {
        let _ = event.id();
    }

    async fn decide_sheep_regrow_wool(&self, event: &mut rspigot_plugin::SheepRegrowWool) {
        let _ = event.id();
    }

    async fn decide_sign_change(&self, event: &mut rspigot_plugin::SignChange) {
        let _ = event.id();
    }

    async fn decide_smith_item(&self, event: &mut rspigot_plugin::SmithItem) {
        let _ = event.id();
    }

    async fn decide_spawner_spawn(&self, event: &mut rspigot_plugin::SpawnerSpawn) {
        let _ = event.id();
    }

    async fn decide_sponge_absorb(&self, event: &mut rspigot_plugin::SpongeAbsorb) {
        let _ = event.id();
    }

    async fn decide_strider_temperature_change(
        &self,
        event: &mut rspigot_plugin::StriderTemperatureChange,
    ) {
        let _ = event.id();
    }

    async fn decide_structure_grow(&self, event: &mut rspigot_plugin::StructureGrow) {
        let _ = event.id();
    }

    async fn decide_tnt_prime(&self, event: &mut rspigot_plugin::TNTPrime) {
        let _ = event.id();
    }

    async fn decide_tab_complete(&self, event: &mut rspigot_plugin::TabComplete) {
        let _ = event.id();
    }

    async fn decide_thunder_change(&self, event: &mut rspigot_plugin::ThunderChange) {
        let _ = event.id();
    }

    async fn decide_time_skip(&self, event: &mut rspigot_plugin::TimeSkip) {
        let _ = event.id();
    }

    async fn decide_trade_select(&self, event: &mut rspigot_plugin::TradeSelect) {
        let _ = event.id();
    }

    async fn decide_trial_spawner_spawn(&self, event: &mut rspigot_plugin::TrialSpawnerSpawn) {
        let _ = event.id();
    }

    async fn decide_vault_display_item(&self, event: &mut rspigot_plugin::VaultDisplayItem) {
        let _ = event.id();
    }

    async fn decide_vehicle_create(&self, event: &mut rspigot_plugin::VehicleCreate) {
        let _ = event.id();
    }

    async fn decide_vehicle_damage(&self, event: &mut rspigot_plugin::VehicleDamage) {
        let _ = event.id();
    }

    async fn decide_vehicle_destroy(&self, event: &mut rspigot_plugin::VehicleDestroy) {
        let _ = event.id();
    }

    async fn decide_vehicle_enter(&self, event: &mut rspigot_plugin::VehicleEnter) {
        let _ = event.id();
    }

    async fn decide_vehicle_entity_collision(
        &self,
        event: &mut rspigot_plugin::VehicleEntityCollision,
    ) {
        let _ = event.id();
    }

    async fn decide_vehicle_exit(&self, event: &mut rspigot_plugin::VehicleExit) {
        let _ = event.id();
    }

    async fn decide_villager_acquire_trade(
        &self,
        event: &mut rspigot_plugin::VillagerAcquireTrade,
    ) {
        let _ = event.id();
    }

    async fn decide_villager_career_change(
        &self,
        event: &mut rspigot_plugin::VillagerCareerChange,
    ) {
        let _ = event.id();
    }

    async fn decide_villager_replenish_trade(
        &self,
        event: &mut rspigot_plugin::VillagerReplenishTrade,
    ) {
        let _ = event.id();
    }

    async fn decide_villager_reputation_change(
        &self,
        event: &mut rspigot_plugin::VillagerReputationChange,
    ) {
        let _ = event.id();
    }

    async fn decide_weather_change(&self, event: &mut rspigot_plugin::WeatherChange) {
        let _ = event.id();
    }

    async fn decide_world_unload(&self, event: &mut rspigot_plugin::WorldUnload) {
        let _ = event.id();
    }
}

#[test]
fn every_event_takes_an_observer_and_every_decidable_one_a_decider() {
    let handlers: Vec<HandlerSpec> = <Everything as Listeners>::handlers();
    assert_eq!(handlers.len(), 462);
    let deciding = handlers
        .iter()
        .filter(|spec| spec.access == Access::Decide)
        .count();
    assert_eq!(deciding, 211);
}
