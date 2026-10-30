use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct BadOmen;

impl MobEffect for BadOmen {
    const NAME: &'static str = "minecraft:bad_omen";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.bad_omen";
    const ID: EffectId = EffectId(30);
    const CATEGORY: EffectCategory = EffectCategory::Neutral;
    const COLOR: u32 = 0x0B6138;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::EveryTick;
}
