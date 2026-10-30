use crate::{AttributeOperation, AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct JumpBoost;

impl MobEffect for JumpBoost {
    const NAME: &'static str = "minecraft:jump_boost";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.jump_boost";
    const ID: EffectId = EffectId(7);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0xFDFF84;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[AttributeTemplate { attribute: "minecraft:safe_fall_distance", modifier: "minecraft:effect.jump_boost", amount: 1.0, operation: AttributeOperation::AddValue }];
    type Behavior = crate::family::Plain;
}
