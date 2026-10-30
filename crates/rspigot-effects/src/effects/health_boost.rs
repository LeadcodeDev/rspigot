use crate::{AttributeOperation, AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct HealthBoost;

impl MobEffect for HealthBoost {
    const NAME: &'static str = "minecraft:health_boost";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.health_boost";
    const ID: EffectId = EffectId(20);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0xF87D23;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[AttributeTemplate { attribute: "minecraft:max_health", modifier: "minecraft:effect.health_boost", amount: 4.0, operation: AttributeOperation::AddValue }];
    type Behavior = crate::family::Plain;
}
