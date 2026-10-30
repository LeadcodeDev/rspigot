#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectId(pub u16);

impl EffectId {
    pub fn from_name(name: &str) -> Option<Self> {
        let full = if name.contains(':') {
            name.to_owned()
        } else {
            format!("minecraft:{name}")
        };
        rspigot_data::registries::static_index(REGISTRY, &full)
            .and_then(|id| u16::try_from(id).ok())
            .map(Self)
    }

    pub fn name(self) -> Option<&'static str> {
        rspigot_data::registries::static_entries(REGISTRY)?
            .get(usize::from(self.0))
            .copied()
    }

    pub fn translation_key(self) -> Option<String> {
        let name = self.name()?;
        let (namespace, path) = name.split_once(':')?;
        Some(format!("effect.{namespace}.{path}"))
    }
}

const REGISTRY: &str = "minecraft:mob_effect";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectDuration {
    Ticks(u32),
    Infinite,
}

impl EffectDuration {
    pub const fn is_shorter_than(self, other: Self) -> bool {
        match (self, other) {
            (Self::Infinite, _) => false,
            (Self::Ticks(_), Self::Infinite) => true,
            (Self::Ticks(a), Self::Ticks(b)) => a < b,
        }
    }

    pub const fn wire_ticks(self) -> i32 {
        match self {
            Self::Infinite => -1,
            Self::Ticks(ticks) => {
                if ticks > i32::MAX as u32 {
                    i32::MAX
                } else {
                    ticks as i32
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectInstance {
    pub effect: EffectId,
    pub amplifier: u8,
    pub duration: EffectDuration,
    pub ambient: bool,
    pub visible: bool,
    pub show_icon: bool,
}

impl EffectInstance {
    pub const fn new(
        effect: EffectId,
        duration: EffectDuration,
        amplifier: u8,
        visible: bool,
    ) -> Self {
        Self {
            effect,
            amplifier,
            duration,
            ambient: false,
            visible,
            show_icon: visible,
        }
    }

    fn update(&mut self, other: &Self) -> bool {
        let mut changed = false;
        if other.amplifier > self.amplifier {
            self.amplifier = other.amplifier;
            self.duration = other.duration;
            changed = true;
        } else if self.duration.is_shorter_than(other.duration) && other.amplifier == self.amplifier
        {
            self.duration = other.duration;
            changed = true;
        }
        if (!other.ambient && self.ambient) || changed {
            self.ambient = other.ambient;
            changed = true;
        }
        if other.visible != self.visible {
            self.visible = other.visible;
            changed = true;
        }
        if other.show_icon != self.show_icon {
            self.show_icon = other.show_icon;
            changed = true;
        }
        changed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum ApplyOutcome {
    Added,
    Updated,
    Unchanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActiveEffects {
    effects: Vec<EffectInstance>,
}

impl ActiveEffects {
    pub fn apply(&mut self, instance: EffectInstance) -> ApplyOutcome {
        match self
            .effects
            .iter_mut()
            .find(|e| e.effect == instance.effect)
        {
            Some(existing) => {
                if existing.update(&instance) {
                    ApplyOutcome::Updated
                } else {
                    ApplyOutcome::Unchanged
                }
            }
            None => {
                self.effects.push(instance);
                ApplyOutcome::Added
            }
        }
    }

    pub fn get(&self, effect: EffectId) -> Option<&EffectInstance> {
        self.effects.iter().find(|e| e.effect == effect)
    }

    pub fn remove(&mut self, effect: EffectId) -> Option<EffectInstance> {
        let index = self.effects.iter().position(|e| e.effect == effect)?;
        Some(self.effects.remove(index))
    }

    pub fn clear(&mut self) -> Vec<EffectInstance> {
        std::mem::take(&mut self.effects)
    }

    pub fn tick(&mut self) -> Vec<EffectId> {
        let mut expired = Vec::new();
        self.effects
            .retain_mut(|effect| match &mut effect.duration {
                EffectDuration::Infinite => true,
                EffectDuration::Ticks(ticks) => {
                    *ticks = ticks.saturating_sub(1);
                    if *ticks == 0 {
                        expired.push(effect.effect);
                        false
                    } else {
                        true
                    }
                }
            });
        expired
    }

    pub fn iter(&self) -> impl Iterator<Item = &EffectInstance> {
        self.effects.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn has(&self, name: &str) -> bool {
        EffectId::from_name(name).is_some_and(|id| self.get(id).is_some())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeOperation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttributeTemplate {
    pub attribute: &'static str,
    pub modifier: &'static str,
    pub amount: f64,
    pub operation: AttributeOperation,
}

const fn template(
    attribute: &'static str,
    modifier: &'static str,
    amount: f64,
    operation: AttributeOperation,
) -> AttributeTemplate {
    AttributeTemplate {
        attribute,
        modifier,
        amount,
        operation,
    }
}

const EFFECT_MODIFIERS: [(&str, AttributeTemplate); 11] = [
    (
        "minecraft:speed",
        template(
            "minecraft:movement_speed",
            "minecraft:effect.speed",
            0.200_000_002_980_232_24,
            AttributeOperation::AddMultipliedTotal,
        ),
    ),
    (
        "minecraft:slowness",
        template(
            "minecraft:movement_speed",
            "minecraft:effect.slowness",
            -0.150_000_005_960_464_48,
            AttributeOperation::AddMultipliedTotal,
        ),
    ),
    (
        "minecraft:haste",
        template(
            "minecraft:attack_speed",
            "minecraft:effect.haste",
            0.100_000_001_490_116_12,
            AttributeOperation::AddMultipliedTotal,
        ),
    ),
    (
        "minecraft:mining_fatigue",
        template(
            "minecraft:attack_speed",
            "minecraft:effect.mining_fatigue",
            -0.100_000_001_490_116_12,
            AttributeOperation::AddMultipliedTotal,
        ),
    ),
    (
        "minecraft:strength",
        template(
            "minecraft:attack_damage",
            "minecraft:effect.strength",
            3.0,
            AttributeOperation::AddValue,
        ),
    ),
    (
        "minecraft:jump_boost",
        template(
            "minecraft:safe_fall_distance",
            "minecraft:effect.jump_boost",
            1.0,
            AttributeOperation::AddValue,
        ),
    ),
    (
        "minecraft:invisibility",
        template(
            "minecraft:waypoint_transmit_range",
            "minecraft:effect.waypoint_transmit_range_hide",
            -1.0,
            AttributeOperation::AddMultipliedTotal,
        ),
    ),
    (
        "minecraft:weakness",
        template(
            "minecraft:attack_damage",
            "minecraft:effect.weakness",
            -4.0,
            AttributeOperation::AddValue,
        ),
    ),
    (
        "minecraft:health_boost",
        template(
            "minecraft:max_health",
            "minecraft:effect.health_boost",
            4.0,
            AttributeOperation::AddValue,
        ),
    ),
    (
        "minecraft:absorption",
        template(
            "minecraft:max_absorption",
            "minecraft:effect.absorption",
            4.0,
            AttributeOperation::AddValue,
        ),
    ),
    (
        "minecraft:luck",
        template(
            "minecraft:luck",
            "minecraft:effect.luck",
            1.0,
            AttributeOperation::AddValue,
        ),
    ),
];

const UNLUCK: AttributeTemplate = template(
    "minecraft:luck",
    "minecraft:effect.unluck",
    -1.0,
    AttributeOperation::AddValue,
);

pub fn attribute_template(effect: &str) -> Option<AttributeTemplate> {
    if effect == "minecraft:unluck" {
        return Some(UNLUCK);
    }
    EFFECT_MODIFIERS
        .iter()
        .find(|(name, _)| *name == effect)
        .map(|(_, t)| *t)
}

pub const PLAYER_ATTRIBUTE_BASES: [(&str, f64); 8] = [
    ("minecraft:movement_speed", 0.100_000_001_490_116_12),
    ("minecraft:attack_speed", 4.0),
    ("minecraft:attack_damage", 1.0),
    ("minecraft:safe_fall_distance", 3.0),
    ("minecraft:waypoint_transmit_range", 6.0e7),
    ("minecraft:max_health", 20.0),
    ("minecraft:max_absorption", 0.0),
    ("minecraft:luck", 0.0),
];

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeModifier {
    pub id: &'static str,
    pub amount: f64,
    pub operation: AttributeOperation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeState {
    pub attribute: &'static str,
    pub base: f64,
    pub modifiers: Vec<AttributeModifier>,
}

pub fn effect_attributes(effects: &ActiveEffects) -> Vec<AttributeState> {
    effects
        .iter()
        .filter_map(|e| Some((e.amplifier, attribute_template(e.effect.name()?)?)))
        .fold(
            Vec::<AttributeState>::new(),
            |mut states, (amplifier, template)| {
                let modifier = AttributeModifier {
                    id: template.modifier,
                    amount: template.amount * (f64::from(amplifier) + 1.0),
                    operation: template.operation,
                };
                match states
                    .iter_mut()
                    .find(|s| s.attribute == template.attribute)
                {
                    Some(state) => state.modifiers.push(modifier),
                    None => states.push(AttributeState {
                        attribute: template.attribute,
                        base: player_base(template.attribute),
                        modifiers: vec![modifier],
                    }),
                }
                states
            },
        )
}

pub fn player_base(attribute: &str) -> f64 {
    PLAYER_ATTRIBUTE_BASES
        .iter()
        .find(|(name, _)| *name == attribute)
        .map_or(0.0, |(_, base)| *base)
}

pub fn affected_attribute(effect: EffectId) -> Option<&'static str> {
    effect
        .name()
        .and_then(attribute_template)
        .map(|t| t.attribute)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn speed() -> EffectId {
        EffectId::from_name("speed").unwrap()
    }

    #[test]
    fn effect_ids_follow_the_registry() {
        assert_eq!(speed(), EffectId(0));
        assert_eq!(EffectId::from_name("minecraft:slowness"), Some(EffectId(1)));
        assert_eq!(
            speed().translation_key().as_deref(),
            Some("effect.minecraft.speed")
        );
        assert_eq!(EffectId::from_name("not_an_effect"), None);
    }

    #[test]
    fn stronger_amplifier_replaces_and_weaker_is_refused() {
        let mut effects = ActiveEffects::default();
        let base = EffectInstance::new(speed(), EffectDuration::Ticks(600), 1, true);
        assert_eq!(effects.apply(base), ApplyOutcome::Added);
        let weaker = EffectInstance::new(speed(), EffectDuration::Ticks(6000), 0, true);
        assert_eq!(effects.apply(weaker), ApplyOutcome::Unchanged);
        let stronger = EffectInstance::new(speed(), EffectDuration::Ticks(20), 3, true);
        assert_eq!(effects.apply(stronger), ApplyOutcome::Updated);
        assert_eq!(
            effects.get(speed()).map(|e| (e.amplifier, e.duration)),
            Some((3, EffectDuration::Ticks(20)))
        );
    }

    #[test]
    fn same_amplifier_only_extends_duration() {
        let mut effects = ActiveEffects::default();
        let _ = effects.apply(EffectInstance::new(
            speed(),
            EffectDuration::Ticks(600),
            0,
            true,
        ));
        assert_eq!(
            effects.apply(EffectInstance::new(
                speed(),
                EffectDuration::Ticks(100),
                0,
                true
            )),
            ApplyOutcome::Unchanged
        );
        assert_eq!(
            effects.apply(EffectInstance::new(
                speed(),
                EffectDuration::Infinite,
                0,
                true
            )),
            ApplyOutcome::Updated
        );
        assert_eq!(
            effects.apply(EffectInstance::new(
                speed(),
                EffectDuration::Ticks(1_000_000),
                0,
                true
            )),
            ApplyOutcome::Unchanged
        );
    }

    #[test]
    fn visibility_change_counts_as_update() {
        let mut effects = ActiveEffects::default();
        let _ = effects.apply(EffectInstance::new(
            speed(),
            EffectDuration::Ticks(600),
            0,
            true,
        ));
        assert_eq!(
            effects.apply(EffectInstance::new(
                speed(),
                EffectDuration::Ticks(600),
                0,
                false
            )),
            ApplyOutcome::Updated
        );
        assert_eq!(
            effects.get(speed()).map(|e| (e.visible, e.show_icon)),
            Some((false, false))
        );
    }

    #[test]
    fn tick_expires_finite_effects_only() {
        let mut effects = ActiveEffects::default();
        let _ = effects.apply(EffectInstance::new(
            speed(),
            EffectDuration::Ticks(2),
            0,
            true,
        ));
        let _ = effects.apply(EffectInstance::new(
            EffectId(1),
            EffectDuration::Infinite,
            0,
            true,
        ));
        assert!(effects.tick().is_empty());
        assert_eq!(effects.tick(), vec![speed()]);
        assert!(effects.get(EffectId(1)).is_some());
        assert_eq!(EffectDuration::Infinite.wire_ticks(), -1);
    }

    #[test]
    fn speed_250_gives_vanilla_modifier() {
        let mut effects = ActiveEffects::default();
        let _ = effects.apply(EffectInstance::new(
            speed(),
            EffectDuration::Ticks(1_999_980),
            250,
            true,
        ));
        let states = effect_attributes(&effects);
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].attribute, "minecraft:movement_speed");
        assert!((states[0].base - 0.1).abs() < 1e-6);
        assert_eq!(states[0].modifiers[0].id, "minecraft:effect.speed");
        assert!((states[0].modifiers[0].amount - 50.2).abs() < 1e-5);
        assert_eq!(
            states[0].modifiers[0].operation,
            AttributeOperation::AddMultipliedTotal
        );
    }

    #[test]
    fn speed_and_slowness_share_one_attribute() {
        let mut effects = ActiveEffects::default();
        let _ = effects.apply(EffectInstance::new(
            speed(),
            EffectDuration::Ticks(10),
            0,
            true,
        ));
        let _ = effects.apply(EffectInstance::new(
            EffectId(1),
            EffectDuration::Ticks(10),
            0,
            true,
        ));
        let states = effect_attributes(&effects);
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].modifiers.len(), 2);
    }
}
