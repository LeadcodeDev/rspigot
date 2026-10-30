use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct DolphinsGrace;

impl MobEffect for DolphinsGrace {
    const NAME: &'static str = "minecraft:dolphins_grace";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.dolphins_grace";
    const ID: EffectId = EffectId(29);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0x88A3BE;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
