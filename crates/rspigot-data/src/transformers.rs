use crate::generated::block_transformers::{AXE, HOE, SHOVEL};

#[derive(Debug, Clone, Copy)]
pub enum Condition {
    Block(&'static str),
    Tag {
        tag: &'static str,
        offset: (i32, i32, i32),
    },
    AllOf(&'static [Condition]),
}

#[derive(Debug, Clone, Copy)]
pub enum Then {
    Copy {
        to: &'static str,
        defaults: &'static [(&'static str, &'static str)],
    },
    Set {
        to: &'static str,
        properties: &'static [(&'static str, &'static str)],
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Rule {
    pub when: Condition,
    pub then: Then,
}

#[derive(Debug, Clone, Copy)]
pub struct Group {
    pub sound: &'static str,
    pub particle: Option<&'static str>,
    pub transform_type: Option<&'static str>,
    pub update_from_neighbors: bool,
    pub damage: u8,
    pub disallowed_faces: &'static [&'static str],
    pub drop_strategy: Option<&'static str>,
    pub loot: Option<&'static str>,
    pub rules: &'static [Rule],
}

pub fn axe() -> &'static [Group] {
    &AXE
}

pub fn hoe() -> &'static [Group] {
    &HOE
}

pub fn shovel() -> &'static [Group] {
    &SHOVEL
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_rule<'a>(group: &'a Group, name: &str) -> Option<&'a Rule> {
        group
            .rules
            .iter()
            .find(|rule| matches!(rule.when, Condition::Block(from) if from == name))
    }

    #[test]
    fn the_first_axe_group_strips_logs_keeping_the_axis_default() {
        let strip = &axe()[0];
        assert_eq!(strip.sound, "minecraft:item.axe.strip");
        assert!(strip.particle.is_none() && strip.transform_type.is_none());
        assert!(strip.update_from_neighbors);
        assert_eq!(strip.damage, 1);
        let rule = block_rule(strip, "minecraft:oak_log").expect("oak log rule");
        match rule.then {
            Then::Copy { to, defaults } => {
                assert_eq!(to, "minecraft:stripped_oak_log");
                assert_eq!(defaults, &[("axis", "y")]);
            }
            Then::Set { .. } => panic!("strip copies the properties"),
        }
    }

    #[test]
    fn every_strip_rule_targets_a_stripped_block() {
        let strip = &axe()[0];
        assert_eq!(strip.rules.len(), 25);
        for rule in strip.rules {
            let (Condition::Block(from), Then::Copy { to, .. }) = (rule.when, rule.then) else {
                panic!("strip rules match one block and copy its properties");
            };
            let source = from.strip_prefix("minecraft:").unwrap_or(from);
            let target = to.strip_prefix("minecraft:").unwrap_or(to);
            assert_eq!(target, format!("stripped_{source}"));
        }
    }

    #[test]
    fn the_hoe_tills_soil_with_air_above_and_never_from_below() {
        let till = &hoe()[0];
        assert_eq!(till.sound, "minecraft:item.hoe.till");
        assert_eq!(till.disallowed_faces, &["down"]);
        let Condition::AllOf(conditions) = till.rules[0].when else {
            panic!("tilling needs soil and air above");
        };
        assert!(matches!(
            conditions[0],
            Condition::Tag {
                tag: "minecraft:turns_into_farmland",
                offset: (0, 0, 0)
            }
        ));
        assert!(matches!(
            conditions[1],
            Condition::Tag {
                tag: "minecraft:air",
                offset: (0, 1, 0)
            }
        ));
        assert!(matches!(
            till.rules[0].then,
            Then::Set {
                to: "minecraft:farmland",
                properties: &[("moisture", "0")]
            }
        ));
    }

    #[test]
    fn rooted_dirt_drops_its_roots_from_the_clicked_face() {
        let rooted = &hoe()[1];
        assert_eq!(rooted.drop_strategy, Some("clicked_face"));
        assert_eq!(rooted.loot, Some("minecraft:till/rooted_dirt"));
        assert!(crate::loot::table("minecraft:till/rooted_dirt").is_some());
    }

    #[test]
    fn the_shovel_flattens_into_a_path() {
        let flatten = &shovel()[0];
        assert_eq!(flatten.sound, "minecraft:item.shovel.flatten");
        assert!(matches!(
            flatten.rules[0].then,
            Then::Set {
                to: "minecraft:dirt_path",
                properties: &[]
            }
        ));
    }

    #[test]
    fn a_block_that_is_not_strippable_has_no_rule() {
        assert!(block_rule(&axe()[0], "minecraft:stone").is_none());
    }
}
