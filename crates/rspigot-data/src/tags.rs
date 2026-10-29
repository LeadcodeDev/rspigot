pub use crate::generated::tags::TAGS;

pub struct RegistryTags {
    pub registry: &'static str,
    pub tags: &'static [(&'static str, &'static [u32])],
}
