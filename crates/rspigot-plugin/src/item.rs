use crate::{Enchantment, Material};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub material: Material,
    pub count: u8,
    pub name: Option<String>,
    pub lore: Vec<String>,
    pub enchantments: Vec<(Enchantment, u32)>,
    pub(crate) components: Vec<u8>,
}

impl ItemStack {
    pub const fn new(material: Material, count: u8) -> Self {
        Self {
            material,
            count,
            name: None,
            lore: Vec::new(),
            enchantments: Vec::new(),
            components: Vec::new(),
        }
    }

    pub fn name(self, text: impl Into<String>) -> Self {
        Self {
            name: Some(text.into()),
            ..self
        }
    }

    pub fn lore<I: IntoIterator<Item = S>, S: Into<String>>(self, lines: I) -> Self {
        Self {
            lore: lines.into_iter().map(Into::into).collect(),
            ..self
        }
    }

    pub fn enchant(mut self, enchantment: Enchantment, level: u32) -> Self {
        match self
            .enchantments
            .iter_mut()
            .find(|(known, _)| *known == enchantment)
        {
            Some((_, known)) => *known = level,
            None => self.enchantments.push((enchantment, level)),
        }
        self
    }
}
