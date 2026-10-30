use crate::{AttributeOperation, AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Slowness;

impl MobEffect for Slowness {
    const NAME: &'static str = "minecraft:slowness";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.slowness";
    const ID: EffectId = EffectId(1);
    const CATEGORY: EffectCategory = EffectCategory::Harmful;
    const COLOR: u32 = 0x8BAFE0;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[AttributeTemplate { attribute: "minecraft:movement_speed", modifier: "minecraft:effect.slowness", amount: -0.15000000596046448, operation: AttributeOperation::AddMultipliedTotal }];
    type Behavior = crate::family::Plain;
}
