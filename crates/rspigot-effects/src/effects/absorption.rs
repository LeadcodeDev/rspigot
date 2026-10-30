use crate::{AttributeOperation, AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Absorption;

impl MobEffect for Absorption {
    const NAME: &'static str = "minecraft:absorption";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.absorption";
    const ID: EffectId = EffectId(21);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0x2552A5;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[AttributeTemplate { attribute: "minecraft:max_absorption", modifier: "minecraft:effect.absorption", amount: 4.0, operation: AttributeOperation::AddValue }];
    type Behavior = crate::family::EveryTick;
}
