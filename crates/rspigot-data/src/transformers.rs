use crate::generated::block_transformers::AXE;

#[derive(Debug, Clone, Copy)]
pub struct Rule {
    pub from: &'static str,
    pub to: &'static str,
    pub defaults: &'static [(&'static str, &'static str)],
}

#[derive(Debug, Clone, Copy)]
pub struct Group {
    pub sound: &'static str,
    pub particle: Option<&'static str>,
    pub transform_type: Option<&'static str>,
    pub update_from_neighbors: bool,
    pub damage: u8,
    pub rules: &'static [Rule],
}

impl Group {
    pub fn rule_for(&self, block: &str) -> Option<&'static Rule> {
        let block = path(block);
        self.rules.iter().find(|rule| path(rule.from) == block)
    }
}

fn path(name: &str) -> &str {
    name.strip_prefix("minecraft:").unwrap_or(name)
}

pub fn axe() -> &'static [Group] {
    &AXE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_axe_group_strips_logs_keeping_the_axis_default() {
        let strip = &axe()[0];
        assert_eq!(strip.sound, "minecraft:item.axe.strip");
        assert!(strip.particle.is_none() && strip.transform_type.is_none());
        assert!(strip.update_from_neighbors);
        assert_eq!(strip.damage, 1);
        let rule = strip.rule_for("minecraft:oak_log").expect("oak log rule");
        assert_eq!(rule.to, "minecraft:stripped_oak_log");
        assert_eq!(rule.defaults, &[("axis", "y")]);
    }

    #[test]
    fn every_strip_rule_targets_a_stripped_block() {
        let strip = &axe()[0];
        assert_eq!(strip.rules.len(), 25);
        for rule in strip.rules {
            let source = rule.from.strip_prefix("minecraft:").unwrap_or(rule.from);
            let target = rule.to.strip_prefix("minecraft:").unwrap_or(rule.to);
            assert_eq!(target, format!("stripped_{source}"));
        }
    }

    #[test]
    fn a_block_that_is_not_strippable_has_no_rule() {
        assert!(axe()[0].rule_for("minecraft:stone").is_none());
    }
}
