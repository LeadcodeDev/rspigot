use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct ConduitPower;

impl MobEffect for ConduitPower {
    const NAME: &'static str = "minecraft:conduit_power";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.conduit_power";
    const ID: EffectId = EffectId(28);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0x1DC2D1;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
