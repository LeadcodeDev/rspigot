use crate::{AttributeOperation, AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Haste;

impl MobEffect for Haste {
    const NAME: &'static str = "minecraft:haste";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.haste";
    const ID: EffectId = EffectId(2);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0xD9C043;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[AttributeTemplate { attribute: "minecraft:attack_speed", modifier: "minecraft:effect.haste", amount: 0.10000000149011612, operation: AttributeOperation::AddMultipliedTotal }];
    type Behavior = crate::family::Plain;
}
