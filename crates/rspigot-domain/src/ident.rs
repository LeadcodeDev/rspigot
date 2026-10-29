use std::fmt;
use std::sync::Arc;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("invalid identifier `{0}`")]
pub struct InvalidIdentifier(pub String);

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier(Arc<str>);

impl Identifier {
    pub const DEFAULT_NAMESPACE: &'static str = "minecraft";

    pub fn parse(raw: &str) -> Result<Self, InvalidIdentifier> {
        let (namespace, path) = match raw.split_once(':') {
            Some((ns, path)) => (
                if ns.is_empty() {
                    Self::DEFAULT_NAMESPACE
                } else {
                    ns
                },
                path,
            ),
            None => (Self::DEFAULT_NAMESPACE, raw),
        };
        let namespace_ok = namespace
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.'));
        let path_ok = !path.is_empty()
            && path
                .bytes()
                .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'/'));
        if !namespace_ok || !path_ok {
            return Err(InvalidIdentifier(raw.to_owned()));
        }
        Ok(Self(Arc::from(format!("{namespace}:{path}"))))
    }

    pub fn minecraft(path: &str) -> Self {
        Self(Arc::from(format!("{}:{path}", Self::DEFAULT_NAMESPACE)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn namespace(&self) -> &str {
        self.0
            .split_once(':')
            .map_or(Self::DEFAULT_NAMESPACE, |(ns, _)| ns)
    }

    pub fn path(&self) -> &str {
        self.0.split_once(':').map_or(&*self.0, |(_, p)| p)
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Identifier({})", self.0)
    }
}

impl TryFrom<&str> for Identifier {
    type Error = InvalidIdentifier;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_adds_default_namespace() {
        let id = Identifier::parse("stone").unwrap();
        assert_eq!(id.as_str(), "minecraft:stone");
        assert_eq!(id.namespace(), "minecraft");
        assert_eq!(id.path(), "stone");
    }

    #[test]
    fn parse_rejects_uppercase() {
        assert!(Identifier::parse("minecraft:Stone").is_err());
    }
}
