use crate::{AttributeOperation, AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Invisibility;

impl MobEffect for Invisibility {
    const NAME: &'static str = "minecraft:invisibility";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.invisibility";
    const ID: EffectId = EffectId(13);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0xF6F6F6;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[AttributeTemplate { attribute: "minecraft:waypoint_transmit_range", modifier: "minecraft:effect.waypoint_transmit_range_hide", amount: -1.0, operation: AttributeOperation::AddMultipliedTotal }];
    type Behavior = crate::family::Plain;
}
