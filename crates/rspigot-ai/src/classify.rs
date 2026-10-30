use rspigot_blocks::{BlockKind, StateFlags, StateId};

use crate::PathType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockClass {
    Plain,
    Trapdoor,
    PowderSnow,
    Damaging,
    StickyHoney,
    Cocoa,
    DamageCautious,
    Burning,
    Campfire,
    WoodenDoor,
    IronDoor,
    Rail,
    Leaves,
    Fence,
    FenceGate,
    SnowLayer,
    Liquid,
    Unpathable,
}

pub(crate) fn path_type_of(state: Option<StateId>) -> PathType {
    let Some(state) = state else {
        return PathType::Blocked;
    };
    if state.is_air() {
        return PathType::Open;
    }
    let Some(kind) = state.kind() else {
        return PathType::Blocked;
    };
    let class = class_of(kind);
    match class {
        BlockClass::Trapdoor => return PathType::Trapdoor,
        BlockClass::PowderSnow => return PathType::PowderSnow,
        BlockClass::Damaging => return PathType::Damaging,
        BlockClass::StickyHoney => return PathType::StickyHoney,
        BlockClass::Cocoa => return PathType::Cocoa,
        BlockClass::DamageCautious => return PathType::DamageCautious,
        _ => {}
    }
    let flags = state.flags();
    if flags.contains(StateFlags::LAVA) {
        return PathType::Lava;
    }
    if is_burning(state, class) {
        return PathType::Fire;
    }
    match class {
        BlockClass::WoodenDoor if is_true(state, "open") => return PathType::DoorOpen,
        BlockClass::IronDoor if is_true(state, "open") => return PathType::DoorOpen,
        BlockClass::WoodenDoor => return PathType::DoorWoodClosed,
        BlockClass::IronDoor => return PathType::DoorIronClosed,
        BlockClass::Rail => return PathType::Rail,
        BlockClass::Leaves => return PathType::Leaves,
        BlockClass::Fence => return PathType::Fence,
        BlockClass::FenceGate if !is_true(state, "open") => return PathType::Fence,
        _ => {}
    }
    if !land_pathfindable(state, kind, class) {
        return PathType::Blocked;
    }
    if flags.contains(StateFlags::WATER) {
        return PathType::Water;
    }
    PathType::Open
}

pub(crate) fn is_pathfindable(state: StateId) -> bool {
    state
        .kind()
        .is_some_and(|kind| land_pathfindable(state, kind, class_of(kind)))
}

pub(crate) fn is_cauldron(state: StateId) -> bool {
    matches!(
        state.kind(),
        Some(
            BlockKind::Cauldron
                | BlockKind::WaterCauldron
                | BlockKind::LavaCauldron
                | BlockKind::PowderSnowCauldron
        )
    )
}

fn is_burning(state: StateId, class: BlockClass) -> bool {
    match class {
        BlockClass::Burning => true,
        BlockClass::Campfire => is_true(state, "lit"),
        _ => false,
    }
}

fn is_true(state: StateId, property: &str) -> bool {
    state.property(property) == Some("true")
}

fn land_pathfindable(state: StateId, kind: BlockKind, class: BlockClass) -> bool {
    let open_shape = !state.flags().contains(StateFlags::FULL_COLLISION_CUBE);
    match class {
        BlockClass::Unpathable | BlockClass::Fence | BlockClass::Campfire | BlockClass::Cocoa => {
            false
        }
        BlockClass::WoodenDoor | BlockClass::IronDoor | BlockClass::FenceGate => {
            is_true(state, "open")
        }
        BlockClass::Trapdoor => match state.property("open") {
            Some(open) => open == "true",
            None => open_shape,
        },
        BlockClass::PowderSnow => true,
        BlockClass::SnowLayer => state
            .property("layers")
            .and_then(|layers| layers.parse::<u8>().ok())
            .is_some_and(|layers| layers < 5),
        BlockClass::Liquid => kind != BlockKind::Lava,
        BlockClass::Damaging => kind != BlockKind::Cactus && open_shape,
        BlockClass::DamageCautious => kind == BlockKind::WitherRose && open_shape,
        BlockClass::Burning => kind != BlockKind::LavaCauldron && open_shape,
        BlockClass::Plain | BlockClass::StickyHoney | BlockClass::Rail | BlockClass::Leaves => {
            open_shape
        }
    }
}

pub(crate) const fn class_of(kind: BlockKind) -> BlockClass {
    match kind {
        BlockKind::OakTrapdoor
        | BlockKind::SpruceTrapdoor
        | BlockKind::BirchTrapdoor
        | BlockKind::JungleTrapdoor
        | BlockKind::AcaciaTrapdoor
        | BlockKind::CherryTrapdoor
        | BlockKind::DarkOakTrapdoor
        | BlockKind::PaleOakTrapdoor
        | BlockKind::MangroveTrapdoor
        | BlockKind::PoplarTrapdoor
        | BlockKind::BambooTrapdoor
        | BlockKind::LilyPad
        | BlockKind::IronTrapdoor
        | BlockKind::CrimsonTrapdoor
        | BlockKind::WarpedTrapdoor
        | BlockKind::CopperTrapdoor
        | BlockKind::ExposedCopperTrapdoor
        | BlockKind::WeatheredCopperTrapdoor
        | BlockKind::OxidizedCopperTrapdoor
        | BlockKind::WaxedCopperTrapdoor
        | BlockKind::WaxedExposedCopperTrapdoor
        | BlockKind::WaxedWeatheredCopperTrapdoor
        | BlockKind::WaxedOxidizedCopperTrapdoor
        | BlockKind::BigDripleaf => BlockClass::Trapdoor,
        BlockKind::PowderSnow => BlockClass::PowderSnow,
        BlockKind::Cactus | BlockKind::SweetBerryBush => BlockClass::Damaging,
        BlockKind::HoneyBlock => BlockClass::StickyHoney,
        BlockKind::Cocoa => BlockClass::Cocoa,
        BlockKind::WitherRose | BlockKind::PointedDripstone | BlockKind::SulfurSpike => {
            BlockClass::DamageCautious
        }
        BlockKind::Fire | BlockKind::SoulFire | BlockKind::LavaCauldron | BlockKind::MagmaBlock => {
            BlockClass::Burning
        }
        BlockKind::Campfire | BlockKind::SoulCampfire => BlockClass::Campfire,
        BlockKind::OakDoor
        | BlockKind::SpruceDoor
        | BlockKind::BirchDoor
        | BlockKind::JungleDoor
        | BlockKind::AcaciaDoor
        | BlockKind::CherryDoor
        | BlockKind::DarkOakDoor
        | BlockKind::PaleOakDoor
        | BlockKind::MangroveDoor
        | BlockKind::PoplarDoor
        | BlockKind::BambooDoor
        | BlockKind::CrimsonDoor
        | BlockKind::WarpedDoor
        | BlockKind::CopperDoor
        | BlockKind::ExposedCopperDoor
        | BlockKind::WeatheredCopperDoor
        | BlockKind::OxidizedCopperDoor
        | BlockKind::WaxedCopperDoor
        | BlockKind::WaxedExposedCopperDoor
        | BlockKind::WaxedWeatheredCopperDoor
        | BlockKind::WaxedOxidizedCopperDoor => BlockClass::WoodenDoor,
        BlockKind::IronDoor => BlockClass::IronDoor,
        BlockKind::PoweredRail
        | BlockKind::DetectorRail
        | BlockKind::Rail
        | BlockKind::ActivatorRail => BlockClass::Rail,
        BlockKind::OakLeaves
        | BlockKind::SpruceLeaves
        | BlockKind::BirchLeaves
        | BlockKind::JungleLeaves
        | BlockKind::AcaciaLeaves
        | BlockKind::CherryLeaves
        | BlockKind::DarkOakLeaves
        | BlockKind::PaleOakLeaves
        | BlockKind::MangroveLeaves
        | BlockKind::RedPoplarLeaves
        | BlockKind::OrangePoplarLeaves
        | BlockKind::YellowPoplarLeaves
        | BlockKind::AzaleaLeaves
        | BlockKind::FloweringAzaleaLeaves => BlockClass::Leaves,
        BlockKind::OakFence
        | BlockKind::ResinBrickWall
        | BlockKind::NetherBrickFence
        | BlockKind::CobblestoneWall
        | BlockKind::MossyCobblestoneWall
        | BlockKind::SpruceFence
        | BlockKind::BirchFence
        | BlockKind::JungleFence
        | BlockKind::AcaciaFence
        | BlockKind::CherryFence
        | BlockKind::DarkOakFence
        | BlockKind::PaleOakFence
        | BlockKind::MangroveFence
        | BlockKind::PoplarFence
        | BlockKind::BambooFence
        | BlockKind::BrickWall
        | BlockKind::PrismarineWall
        | BlockKind::RedSandstoneWall
        | BlockKind::MossyStoneBrickWall
        | BlockKind::GraniteWall
        | BlockKind::StoneBrickWall
        | BlockKind::MudBrickWall
        | BlockKind::NetherBrickWall
        | BlockKind::AndesiteWall
        | BlockKind::RedNetherBrickWall
        | BlockKind::SandstoneWall
        | BlockKind::EndStoneBrickWall
        | BlockKind::DioriteWall
        | BlockKind::CrimsonFence
        | BlockKind::WarpedFence
        | BlockKind::BlackstoneWall
        | BlockKind::PolishedBlackstoneBrickWall
        | BlockKind::PolishedBlackstoneWall
        | BlockKind::TuffWall
        | BlockKind::PolishedTuffWall
        | BlockKind::TuffBrickWall
        | BlockKind::SulfurWall
        | BlockKind::PolishedSulfurWall
        | BlockKind::SulfurBrickWall
        | BlockKind::CinnabarWall
        | BlockKind::PolishedCinnabarWall
        | BlockKind::CinnabarBrickWall
        | BlockKind::CobbledDeepslateWall
        | BlockKind::PolishedDeepslateWall
        | BlockKind::DeepslateTileWall
        | BlockKind::DeepslateBrickWall => BlockClass::Fence,
        BlockKind::OakFenceGate
        | BlockKind::SpruceFenceGate
        | BlockKind::BirchFenceGate
        | BlockKind::JungleFenceGate
        | BlockKind::AcaciaFenceGate
        | BlockKind::CherryFenceGate
        | BlockKind::DarkOakFenceGate
        | BlockKind::PaleOakFenceGate
        | BlockKind::MangroveFenceGate
        | BlockKind::PoplarFenceGate
        | BlockKind::BambooFenceGate
        | BlockKind::CrimsonFenceGate
        | BlockKind::WarpedFenceGate => BlockClass::FenceGate,
        BlockKind::Snow => BlockClass::SnowLayer,
        BlockKind::Water | BlockKind::Lava => BlockClass::Liquid,
        BlockKind::WhiteBed
        | BlockKind::OrangeBed
        | BlockKind::MagentaBed
        | BlockKind::LightBlueBed
        | BlockKind::YellowBed
        | BlockKind::LimeBed
        | BlockKind::PinkBed
        | BlockKind::GrayBed
        | BlockKind::LightGrayBed
        | BlockKind::CyanBed
        | BlockKind::PurpleBed
        | BlockKind::BlueBed
        | BlockKind::BrownBed
        | BlockKind::GreenBed
        | BlockKind::RedBed
        | BlockKind::BlackBed
        | BlockKind::StrawBed
        | BlockKind::StickyPiston
        | BlockKind::Piston
        | BlockKind::PistonHead
        | BlockKind::WhiteWoolStairs
        | BlockKind::OrangeWoolStairs
        | BlockKind::MagentaWoolStairs
        | BlockKind::LightBlueWoolStairs
        | BlockKind::YellowWoolStairs
        | BlockKind::LimeWoolStairs
        | BlockKind::PinkWoolStairs
        | BlockKind::GrayWoolStairs
        | BlockKind::LightGrayWoolStairs
        | BlockKind::CyanWoolStairs
        | BlockKind::PurpleWoolStairs
        | BlockKind::BlueWoolStairs
        | BlockKind::BrownWoolStairs
        | BlockKind::GreenWoolStairs
        | BlockKind::RedWoolStairs
        | BlockKind::BlackWoolStairs
        | BlockKind::WhiteWoolSlab
        | BlockKind::OrangeWoolSlab
        | BlockKind::MagentaWoolSlab
        | BlockKind::LightBlueWoolSlab
        | BlockKind::YellowWoolSlab
        | BlockKind::LimeWoolSlab
        | BlockKind::PinkWoolSlab
        | BlockKind::GrayWoolSlab
        | BlockKind::LightGrayWoolSlab
        | BlockKind::CyanWoolSlab
        | BlockKind::PurpleWoolSlab
        | BlockKind::BlueWoolSlab
        | BlockKind::BrownWoolSlab
        | BlockKind::GreenWoolSlab
        | BlockKind::RedWoolSlab
        | BlockKind::BlackWoolSlab
        | BlockKind::MovingPiston
        | BlockKind::AcaciaShelf
        | BlockKind::BambooShelf
        | BlockKind::BirchShelf
        | BlockKind::CherryShelf
        | BlockKind::CrimsonShelf
        | BlockKind::DarkOakShelf
        | BlockKind::JungleShelf
        | BlockKind::MangroveShelf
        | BlockKind::OakShelf
        | BlockKind::PaleOakShelf
        | BlockKind::PoplarShelf
        | BlockKind::SpruceShelf
        | BlockKind::WarpedShelf
        | BlockKind::OakStairs
        | BlockKind::Chest
        | BlockKind::Farmland
        | BlockKind::CobblestoneStairs
        | BlockKind::OakWallHangingSign
        | BlockKind::SpruceWallHangingSign
        | BlockKind::BirchWallHangingSign
        | BlockKind::AcaciaWallHangingSign
        | BlockKind::CherryWallHangingSign
        | BlockKind::JungleWallHangingSign
        | BlockKind::DarkOakWallHangingSign
        | BlockKind::PaleOakWallHangingSign
        | BlockKind::MangroveWallHangingSign
        | BlockKind::PoplarWallHangingSign
        | BlockKind::CrimsonWallHangingSign
        | BlockKind::WarpedWallHangingSign
        | BlockKind::BambooWallHangingSign
        | BlockKind::SoulSand
        | BlockKind::Cake
        | BlockKind::IronBars
        | BlockKind::CopperBars
        | BlockKind::ExposedCopperBars
        | BlockKind::WeatheredCopperBars
        | BlockKind::OxidizedCopperBars
        | BlockKind::WaxedCopperBars
        | BlockKind::WaxedExposedCopperBars
        | BlockKind::WaxedWeatheredCopperBars
        | BlockKind::WaxedOxidizedCopperBars
        | BlockKind::IronChain
        | BlockKind::CopperChain
        | BlockKind::ExposedCopperChain
        | BlockKind::WeatheredCopperChain
        | BlockKind::OxidizedCopperChain
        | BlockKind::WaxedCopperChain
        | BlockKind::WaxedExposedCopperChain
        | BlockKind::WaxedWeatheredCopperChain
        | BlockKind::WaxedOxidizedCopperChain
        | BlockKind::GlassPane
        | BlockKind::BrickStairs
        | BlockKind::StoneBrickStairs
        | BlockKind::MudBrickStairs
        | BlockKind::ResinBrickStairs
        | BlockKind::ResinBrickSlab
        | BlockKind::NetherBrickStairs
        | BlockKind::EnchantingTable
        | BlockKind::BrewingStand
        | BlockKind::Cauldron
        | BlockKind::WaterCauldron
        | BlockKind::PowderSnowCauldron
        | BlockKind::EndPortalFrame
        | BlockKind::DragonEgg
        | BlockKind::ShelfMushroom
        | BlockKind::SandstoneStairs
        | BlockKind::EnderChest
        | BlockKind::SpruceStairs
        | BlockKind::BirchStairs
        | BlockKind::JungleStairs
        | BlockKind::FlowerPot
        | BlockKind::PottedTorchflower
        | BlockKind::PottedOakSapling
        | BlockKind::PottedSpruceSapling
        | BlockKind::PottedBirchSapling
        | BlockKind::PottedJungleSapling
        | BlockKind::PottedAcaciaSapling
        | BlockKind::PottedCherrySapling
        | BlockKind::PottedDarkOakSapling
        | BlockKind::PottedPaleOakSapling
        | BlockKind::PottedPoplarSapling
        | BlockKind::PottedMangrovePropagule
        | BlockKind::PottedFern
        | BlockKind::PottedDandelion
        | BlockKind::PottedGoldenDandelion
        | BlockKind::PottedPoppy
        | BlockKind::PottedBlueOrchid
        | BlockKind::PottedAllium
        | BlockKind::PottedAzureBluet
        | BlockKind::PottedRedTulip
        | BlockKind::PottedOrangeTulip
        | BlockKind::PottedWhiteTulip
        | BlockKind::PottedPinkTulip
        | BlockKind::PottedOxeyeDaisy
        | BlockKind::PottedCornflower
        | BlockKind::PottedLilyOfTheValley
        | BlockKind::PottedWitherRose
        | BlockKind::PottedRedMushroom
        | BlockKind::PottedBrownMushroom
        | BlockKind::PottedDeadBush
        | BlockKind::PottedCactus
        | BlockKind::SkeletonSkull
        | BlockKind::SkeletonWallSkull
        | BlockKind::WitherSkeletonSkull
        | BlockKind::WitherSkeletonWallSkull
        | BlockKind::ZombieHead
        | BlockKind::ZombieWallHead
        | BlockKind::PlayerHead
        | BlockKind::PlayerWallHead
        | BlockKind::CreeperHead
        | BlockKind::CreeperWallHead
        | BlockKind::DragonHead
        | BlockKind::DragonWallHead
        | BlockKind::PiglinHead
        | BlockKind::PiglinWallHead
        | BlockKind::Anvil
        | BlockKind::ChippedAnvil
        | BlockKind::DamagedAnvil
        | BlockKind::TrappedChest
        | BlockKind::Hopper
        | BlockKind::QuartzStairs
        | BlockKind::WhiteStainedGlassPane
        | BlockKind::OrangeStainedGlassPane
        | BlockKind::MagentaStainedGlassPane
        | BlockKind::LightBlueStainedGlassPane
        | BlockKind::YellowStainedGlassPane
        | BlockKind::LimeStainedGlassPane
        | BlockKind::PinkStainedGlassPane
        | BlockKind::GrayStainedGlassPane
        | BlockKind::LightGrayStainedGlassPane
        | BlockKind::CyanStainedGlassPane
        | BlockKind::PurpleStainedGlassPane
        | BlockKind::BlueStainedGlassPane
        | BlockKind::BrownStainedGlassPane
        | BlockKind::GreenStainedGlassPane
        | BlockKind::RedStainedGlassPane
        | BlockKind::BlackStainedGlassPane
        | BlockKind::AcaciaStairs
        | BlockKind::CherryStairs
        | BlockKind::DarkOakStairs
        | BlockKind::PaleOakStairs
        | BlockKind::MangroveStairs
        | BlockKind::PoplarStairs
        | BlockKind::BambooStairs
        | BlockKind::BambooMosaicStairs
        | BlockKind::PrismarineStairs
        | BlockKind::PrismarineBrickStairs
        | BlockKind::DarkPrismarineStairs
        | BlockKind::PrismarineSlab
        | BlockKind::PrismarineBrickSlab
        | BlockKind::DarkPrismarineSlab
        | BlockKind::RedSandstoneStairs
        | BlockKind::OakSlab
        | BlockKind::SpruceSlab
        | BlockKind::BirchSlab
        | BlockKind::JungleSlab
        | BlockKind::AcaciaSlab
        | BlockKind::CherrySlab
        | BlockKind::DarkOakSlab
        | BlockKind::PaleOakSlab
        | BlockKind::MangroveSlab
        | BlockKind::PoplarSlab
        | BlockKind::BambooSlab
        | BlockKind::BambooMosaicSlab
        | BlockKind::StoneSlab
        | BlockKind::SandstoneSlab
        | BlockKind::CutSandstoneSlab
        | BlockKind::PetrifiedOakSlab
        | BlockKind::CobblestoneSlab
        | BlockKind::BrickSlab
        | BlockKind::StoneBrickSlab
        | BlockKind::MudBrickSlab
        | BlockKind::NetherBrickSlab
        | BlockKind::QuartzSlab
        | BlockKind::RedSandstoneSlab
        | BlockKind::CutRedSandstoneSlab
        | BlockKind::SmoothStoneSlab
        | BlockKind::EndRod
        | BlockKind::ChorusPlant
        | BlockKind::PurpurSlab
        | BlockKind::PurpurStairs
        | BlockKind::DirtPath
        | BlockKind::WhiteConcreteStairs
        | BlockKind::OrangeConcreteStairs
        | BlockKind::MagentaConcreteStairs
        | BlockKind::LightBlueConcreteStairs
        | BlockKind::YellowConcreteStairs
        | BlockKind::LimeConcreteStairs
        | BlockKind::PinkConcreteStairs
        | BlockKind::GrayConcreteStairs
        | BlockKind::LightGrayConcreteStairs
        | BlockKind::CyanConcreteStairs
        | BlockKind::PurpleConcreteStairs
        | BlockKind::BlueConcreteStairs
        | BlockKind::BrownConcreteStairs
        | BlockKind::GreenConcreteStairs
        | BlockKind::RedConcreteStairs
        | BlockKind::BlackConcreteStairs
        | BlockKind::WhiteConcreteSlab
        | BlockKind::OrangeConcreteSlab
        | BlockKind::MagentaConcreteSlab
        | BlockKind::LightBlueConcreteSlab
        | BlockKind::YellowConcreteSlab
        | BlockKind::LimeConcreteSlab
        | BlockKind::PinkConcreteSlab
        | BlockKind::GrayConcreteSlab
        | BlockKind::LightGrayConcreteSlab
        | BlockKind::CyanConcreteSlab
        | BlockKind::PurpleConcreteSlab
        | BlockKind::BlueConcreteSlab
        | BlockKind::BrownConcreteSlab
        | BlockKind::GreenConcreteSlab
        | BlockKind::RedConcreteSlab
        | BlockKind::BlackConcreteSlab
        | BlockKind::SnifferEgg
        | BlockKind::DriedGhast
        | BlockKind::SeaPickle
        | BlockKind::Conduit
        | BlockKind::Bamboo
        | BlockKind::PottedBamboo
        | BlockKind::PolishedGraniteStairs
        | BlockKind::SmoothRedSandstoneStairs
        | BlockKind::MossyStoneBrickStairs
        | BlockKind::PolishedDioriteStairs
        | BlockKind::MossyCobblestoneStairs
        | BlockKind::EndStoneBrickStairs
        | BlockKind::StoneStairs
        | BlockKind::SmoothSandstoneStairs
        | BlockKind::SmoothQuartzStairs
        | BlockKind::GraniteStairs
        | BlockKind::AndesiteStairs
        | BlockKind::RedNetherBrickStairs
        | BlockKind::PolishedAndesiteStairs
        | BlockKind::DioriteStairs
        | BlockKind::PolishedGraniteSlab
        | BlockKind::SmoothRedSandstoneSlab
        | BlockKind::MossyStoneBrickSlab
        | BlockKind::PolishedDioriteSlab
        | BlockKind::MossyCobblestoneSlab
        | BlockKind::EndStoneBrickSlab
        | BlockKind::SmoothSandstoneSlab
        | BlockKind::SmoothQuartzSlab
        | BlockKind::GraniteSlab
        | BlockKind::AndesiteSlab
        | BlockKind::RedNetherBrickSlab
        | BlockKind::PolishedAndesiteSlab
        | BlockKind::DioriteSlab
        | BlockKind::Grindstone
        | BlockKind::Lectern
        | BlockKind::Stonecutter
        | BlockKind::Bell
        | BlockKind::Lantern
        | BlockKind::SoulLantern
        | BlockKind::CopperLantern
        | BlockKind::ExposedCopperLantern
        | BlockKind::WeatheredCopperLantern
        | BlockKind::OxidizedCopperLantern
        | BlockKind::WaxedCopperLantern
        | BlockKind::WaxedExposedCopperLantern
        | BlockKind::WaxedWeatheredCopperLantern
        | BlockKind::WaxedOxidizedCopperLantern
        | BlockKind::CrimsonSlab
        | BlockKind::WarpedSlab
        | BlockKind::CrimsonStairs
        | BlockKind::WarpedStairs
        | BlockKind::Composter
        | BlockKind::RespawnAnchor
        | BlockKind::PottedCrimsonFungus
        | BlockKind::PottedWarpedFungus
        | BlockKind::PottedCrimsonRoots
        | BlockKind::PottedWarpedRoots
        | BlockKind::BlackstoneStairs
        | BlockKind::BlackstoneSlab
        | BlockKind::PolishedBlackstoneBrickSlab
        | BlockKind::PolishedBlackstoneBrickStairs
        | BlockKind::PolishedBlackstoneStairs
        | BlockKind::PolishedBlackstoneSlab
        | BlockKind::CandleCake
        | BlockKind::WhiteCandleCake
        | BlockKind::OrangeCandleCake
        | BlockKind::MagentaCandleCake
        | BlockKind::LightBlueCandleCake
        | BlockKind::YellowCandleCake
        | BlockKind::LimeCandleCake
        | BlockKind::PinkCandleCake
        | BlockKind::GrayCandleCake
        | BlockKind::LightGrayCandleCake
        | BlockKind::CyanCandleCake
        | BlockKind::PurpleCandleCake
        | BlockKind::BlueCandleCake
        | BlockKind::BrownCandleCake
        | BlockKind::GreenCandleCake
        | BlockKind::RedCandleCake
        | BlockKind::BlackCandleCake
        | BlockKind::TuffSlab
        | BlockKind::TuffStairs
        | BlockKind::PolishedTuffSlab
        | BlockKind::PolishedTuffStairs
        | BlockKind::TuffBrickSlab
        | BlockKind::TuffBrickStairs
        | BlockKind::SulfurSlab
        | BlockKind::SulfurStairs
        | BlockKind::PolishedSulfurSlab
        | BlockKind::PolishedSulfurStairs
        | BlockKind::SulfurBrickSlab
        | BlockKind::SulfurBrickStairs
        | BlockKind::CinnabarSlab
        | BlockKind::CinnabarStairs
        | BlockKind::PolishedCinnabarSlab
        | BlockKind::PolishedCinnabarStairs
        | BlockKind::CinnabarBrickSlab
        | BlockKind::CinnabarBrickStairs
        | BlockKind::SculkSensor
        | BlockKind::CalibratedSculkSensor
        | BlockKind::CutCopperStairs
        | BlockKind::ExposedCutCopperStairs
        | BlockKind::WeatheredCutCopperStairs
        | BlockKind::OxidizedCutCopperStairs
        | BlockKind::WaxedCutCopperStairs
        | BlockKind::WaxedExposedCutCopperStairs
        | BlockKind::WaxedWeatheredCutCopperStairs
        | BlockKind::WaxedOxidizedCutCopperStairs
        | BlockKind::CutCopperSlab
        | BlockKind::ExposedCutCopperSlab
        | BlockKind::WeatheredCutCopperSlab
        | BlockKind::OxidizedCutCopperSlab
        | BlockKind::WaxedCutCopperSlab
        | BlockKind::WaxedExposedCutCopperSlab
        | BlockKind::WaxedWeatheredCutCopperSlab
        | BlockKind::WaxedOxidizedCutCopperSlab
        | BlockKind::CopperChest
        | BlockKind::ExposedCopperChest
        | BlockKind::WeatheredCopperChest
        | BlockKind::OxidizedCopperChest
        | BlockKind::WaxedCopperChest
        | BlockKind::WaxedExposedCopperChest
        | BlockKind::WaxedWeatheredCopperChest
        | BlockKind::WaxedOxidizedCopperChest
        | BlockKind::CopperGolemStatue
        | BlockKind::ExposedCopperGolemStatue
        | BlockKind::WeatheredCopperGolemStatue
        | BlockKind::OxidizedCopperGolemStatue
        | BlockKind::WaxedCopperGolemStatue
        | BlockKind::WaxedExposedCopperGolemStatue
        | BlockKind::WaxedWeatheredCopperGolemStatue
        | BlockKind::WaxedOxidizedCopperGolemStatue
        | BlockKind::LightningRod
        | BlockKind::ExposedLightningRod
        | BlockKind::WeatheredLightningRod
        | BlockKind::OxidizedLightningRod
        | BlockKind::WaxedLightningRod
        | BlockKind::WaxedExposedLightningRod
        | BlockKind::WaxedWeatheredLightningRod
        | BlockKind::WaxedOxidizedLightningRod
        | BlockKind::Azalea
        | BlockKind::FloweringAzalea
        | BlockKind::Mud
        | BlockKind::CobbledDeepslateStairs
        | BlockKind::CobbledDeepslateSlab
        | BlockKind::PolishedDeepslateStairs
        | BlockKind::PolishedDeepslateSlab
        | BlockKind::DeepslateTileStairs
        | BlockKind::DeepslateTileSlab
        | BlockKind::DeepslateBrickStairs
        | BlockKind::DeepslateBrickSlab
        | BlockKind::PottedAzaleaBush
        | BlockKind::PottedFloweringAzaleaBush
        | BlockKind::DecoratedPot
        | BlockKind::HeavyCore
        | BlockKind::PottedOpenEyeblossom
        | BlockKind::PottedClosedEyeblossom => BlockClass::Unpathable,
        _ => BlockClass::Plain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(kind: BlockKind, properties: &[(&str, &str)]) -> Option<StateId> {
        kind.state_with(properties)
    }

    #[test]
    fn simple_blocks() {
        assert_eq!(path_type_of(Some(StateId::AIR)), PathType::Open);
        assert_eq!(path_type_of(None), PathType::Blocked);
        assert_eq!(
            path_type_of(state(BlockKind::Stone, &[])),
            PathType::Blocked
        );
        assert_eq!(
            path_type_of(state(BlockKind::ShortGrass, &[])),
            PathType::Open
        );
        assert_eq!(
            path_type_of(state(BlockKind::WhiteCarpet, &[])),
            PathType::Open
        );
        assert_eq!(path_type_of(state(BlockKind::Water, &[])), PathType::Water);
        assert_eq!(path_type_of(state(BlockKind::Lava, &[])), PathType::Lava);
    }

    #[test]
    fn damaging_and_burning_blocks() {
        assert_eq!(
            path_type_of(state(BlockKind::Cactus, &[])),
            PathType::Damaging
        );
        assert_eq!(
            path_type_of(state(BlockKind::SweetBerryBush, &[])),
            PathType::Damaging
        );
        assert_eq!(path_type_of(state(BlockKind::Fire, &[])), PathType::Fire);
        assert_eq!(
            path_type_of(state(BlockKind::SoulFire, &[])),
            PathType::Fire
        );
        assert_eq!(
            path_type_of(state(BlockKind::MagmaBlock, &[])),
            PathType::Fire
        );
        assert_eq!(
            path_type_of(state(BlockKind::LavaCauldron, &[])),
            PathType::Fire
        );
        assert_eq!(
            path_type_of(state(BlockKind::Campfire, &[("lit", "true")])),
            PathType::Fire
        );
        assert_eq!(
            path_type_of(state(BlockKind::Campfire, &[("lit", "false")])),
            PathType::Blocked
        );
        assert_eq!(
            path_type_of(state(BlockKind::WitherRose, &[])),
            PathType::DamageCautious
        );
        assert_eq!(
            path_type_of(state(BlockKind::PointedDripstone, &[])),
            PathType::DamageCautious
        );
        assert_eq!(
            path_type_of(state(BlockKind::HoneyBlock, &[])),
            PathType::StickyHoney
        );
        assert_eq!(path_type_of(state(BlockKind::Cocoa, &[])), PathType::Cocoa);
        assert_eq!(
            path_type_of(state(BlockKind::PowderSnow, &[])),
            PathType::PowderSnow
        );
    }

    #[test]
    fn doors_gates_and_trapdoors() {
        assert_eq!(
            path_type_of(state(BlockKind::OakDoor, &[("open", "false")])),
            PathType::DoorWoodClosed
        );
        assert_eq!(
            path_type_of(state(BlockKind::CopperDoor, &[("open", "false")])),
            PathType::DoorWoodClosed
        );
        assert_eq!(
            path_type_of(state(BlockKind::IronDoor, &[("open", "false")])),
            PathType::DoorIronClosed
        );
        assert_eq!(
            path_type_of(state(BlockKind::IronDoor, &[("open", "true")])),
            PathType::DoorOpen
        );
        assert_eq!(
            path_type_of(state(BlockKind::OakFenceGate, &[("open", "false")])),
            PathType::Fence
        );
        assert_eq!(
            path_type_of(state(BlockKind::OakFenceGate, &[("open", "true")])),
            PathType::Open
        );
        assert_eq!(
            path_type_of(state(BlockKind::IronTrapdoor, &[])),
            PathType::Trapdoor
        );
        assert_eq!(
            path_type_of(state(BlockKind::LilyPad, &[])),
            PathType::Trapdoor
        );
        assert_eq!(
            path_type_of(state(BlockKind::BigDripleaf, &[])),
            PathType::Trapdoor
        );
    }

    #[test]
    fn fences_walls_leaves_rails() {
        assert_eq!(
            path_type_of(state(BlockKind::OakFence, &[])),
            PathType::Fence
        );
        assert_eq!(
            path_type_of(state(BlockKind::NetherBrickFence, &[])),
            PathType::Fence
        );
        assert_eq!(
            path_type_of(state(BlockKind::CobblestoneWall, &[])),
            PathType::Fence
        );
        assert_eq!(
            path_type_of(state(BlockKind::OakLeaves, &[])),
            PathType::Leaves
        );
        assert_eq!(
            path_type_of(state(BlockKind::AzaleaLeaves, &[])),
            PathType::Leaves
        );
        assert_eq!(path_type_of(state(BlockKind::Rail, &[])), PathType::Rail);
        assert_eq!(
            path_type_of(state(BlockKind::PoweredRail, &[])),
            PathType::Rail
        );
    }

    #[test]
    fn unpathable_shapes_block() {
        for kind in [
            BlockKind::OakStairs,
            BlockKind::OakSlab,
            BlockKind::RedBed,
            BlockKind::Chest,
            BlockKind::Cauldron,
            BlockKind::FlowerPot,
            BlockKind::Farmland,
            BlockKind::DirtPath,
            BlockKind::SoulSand,
            BlockKind::Mud,
            BlockKind::GlassPane,
            BlockKind::IronBars,
            BlockKind::Anvil,
            BlockKind::Lantern,
        ] {
            assert_eq!(
                path_type_of(state(kind, &[])),
                PathType::Blocked,
                "{kind:?}"
            );
        }
        assert_eq!(
            path_type_of(state(BlockKind::OakSlab, &[("waterlogged", "true")])),
            PathType::Blocked
        );
        assert_eq!(
            path_type_of(state(BlockKind::Ladder, &[("waterlogged", "true")])),
            PathType::Water
        );
    }

    #[test]
    fn snow_layers_up_to_four_are_open() {
        assert_eq!(
            path_type_of(state(BlockKind::Snow, &[("layers", "4")])),
            PathType::Open
        );
        assert_eq!(
            path_type_of(state(BlockKind::Snow, &[("layers", "5")])),
            PathType::Blocked
        );
    }

    #[test]
    fn pathfindable_matches_vanilla_overrides() {
        let check = |kind: BlockKind, properties: &[(&str, &str)]| {
            state(kind, properties).is_some_and(is_pathfindable)
        };
        assert!(check(BlockKind::Air, &[]));
        assert!(!check(BlockKind::Stone, &[]));
        assert!(check(BlockKind::Water, &[]));
        assert!(!check(BlockKind::Lava, &[]));
        assert!(check(BlockKind::PowderSnow, &[]));
        assert!(!check(BlockKind::Cactus, &[]));
        assert!(check(BlockKind::SweetBerryBush, &[]));
        assert!(check(BlockKind::OakTrapdoor, &[("open", "true")]));
        assert!(!check(BlockKind::OakTrapdoor, &[("open", "false")]));
        assert!(check(BlockKind::LilyPad, &[]));
        assert!(!check(BlockKind::PointedDripstone, &[]));
        assert!(check(BlockKind::Fire, &[]));
        assert!(!check(BlockKind::LavaCauldron, &[]));
        assert!(is_cauldron(BlockKind::WaterCauldron.default_state()));
        assert!(!is_cauldron(BlockKind::Stone.default_state()));
    }
}
