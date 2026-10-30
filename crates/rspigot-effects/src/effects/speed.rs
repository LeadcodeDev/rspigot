use crate::{AttributeOperation, AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Speed;

impl MobEffect for Speed {
    const NAME: &'static str = "minecraft:speed";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.speed";
    const ID: EffectId = EffectId(0);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0x33EBFF;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[AttributeTemplate { attribute: "minecraft:movement_speed", modifier: "minecraft:effect.speed", amount: 0.20000000298023224, operation: AttributeOperation::AddMultipliedTotal }];
    type Behavior = crate::family::Plain;
}
