use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct RaidOmen;

impl MobEffect for RaidOmen {
    const NAME: &'static str = "minecraft:raid_omen";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.raid_omen";
    const ID: EffectId = EffectId(34);
    const CATEGORY: EffectCategory = EffectCategory::Neutral;
    const COLOR: u32 = 0xDE4058;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::LastTick;
}
