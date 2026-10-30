use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct HeroOfTheVillage;

impl MobEffect for HeroOfTheVillage {
    const NAME: &'static str = "minecraft:hero_of_the_village";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.hero_of_the_village";
    const ID: EffectId = EffectId(31);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0x44FF44;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
