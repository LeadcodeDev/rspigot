use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct TrialOmen;

impl MobEffect for TrialOmen {
    const NAME: &'static str = "minecraft:trial_omen";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.trial_omen";
    const ID: EffectId = EffectId(33);
    const CATEGORY: EffectCategory = EffectCategory::Neutral;
    const COLOR: u32 = 0x16A6A6;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
