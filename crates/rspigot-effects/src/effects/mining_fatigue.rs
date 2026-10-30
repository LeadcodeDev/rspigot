use crate::{AttributeOperation, AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct MiningFatigue;

impl MobEffect for MiningFatigue {
    const NAME: &'static str = "minecraft:mining_fatigue";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.mining_fatigue";
    const ID: EffectId = EffectId(3);
    const CATEGORY: EffectCategory = EffectCategory::Harmful;
    const COLOR: u32 = 0x4A4217;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[AttributeTemplate { attribute: "minecraft:attack_speed", modifier: "minecraft:effect.mining_fatigue", amount: -0.10000000149011612, operation: AttributeOperation::AddMultipliedTotal }];
    type Behavior = crate::family::Plain;
}
