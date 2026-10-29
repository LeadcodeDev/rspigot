use crate::generated::static_registries::{ENTITY_TYPE, STATIC};

pub fn id(name: &str) -> Option<u16> {
    STATIC[ENTITY_TYPE].index_of(name)
}

pub fn name(id: u16) -> Option<&'static str> {
    STATIC[ENTITY_TYPE].name_of(id)
}
