use rspigot_domain::world::Dimension;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DimensionKind {
    Overworld,
    Nether,
    End,
}

impl DimensionKind {
    pub(crate) fn of(dimension: &Dimension) -> Self {
        match dimension.dimension_type.path() {
            "the_nether" => Self::Nether,
            "the_end" => Self::End,
            _ => Self::Overworld,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_dimension_type() {
        assert_eq!(
            DimensionKind::of(&Dimension::overworld()),
            DimensionKind::Overworld
        );
        assert_eq!(
            DimensionKind::of(&Dimension::nether()),
            DimensionKind::Nether
        );
        assert_eq!(DimensionKind::of(&Dimension::end()), DimensionKind::End);
    }
}
