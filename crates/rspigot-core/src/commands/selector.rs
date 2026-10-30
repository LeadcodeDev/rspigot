use rspigot_domain::text::Component;
use rspigot_entities::EntityKind;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    Arbitrary,
    Nearest,
    Furthest,
    Random,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeMatch {
    Kind(EntityKind),
    Tag(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectorBase {
    Name(String),
    Uuid(Uuid),
    Sender,
    Search,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub base: SelectorBase,
    pub limit: usize,
    pub sort: Sort,
    pub includes_entities: bool,
    pub alive_only: bool,
    pub types: Vec<(bool, TypeMatch)>,
    pub distance: Option<(Option<f64>, Option<f64>)>,
    pub names: Vec<(bool, String)>,
    pub tags: Vec<(bool, String)>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub dx: Option<f64>,
    pub dy: Option<f64>,
    pub dz: Option<f64>,
}

fn error(key: &str, args: Vec<Component>) -> Component {
    Component::translatable(key, args)
}

pub fn split_argument(input: &str) -> (&str, &str) {
    let input = input.trim_start();
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (index, c) in input.char_indices() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => quote = Some(c),
            '[' | '{' => depth += 1,
            ']' | '}' => depth -= 1,
            c if c.is_whitespace() && depth <= 0 => {
                return (&input[..index], &input[index..]);
            }
            _ => {}
        }
    }
    (input, "")
}

fn parse_uuid(text: &str) -> Option<Uuid> {
    if text.split('-').count() != 5 {
        return None;
    }
    Uuid::parse_str(text).ok()
}

fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 2
        && (bytes[0] == b'"' || bytes[0] == b'\'')
        && bytes[bytes.len() - 1] == bytes[0]
    {
        let inner = &trimmed[1..trimmed.len() - 1];
        let mut out = String::with_capacity(inner.len());
        let mut chars = inner.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            } else {
                out.push(c);
            }
        }
        out
    } else {
        trimmed.to_owned()
    }
}

fn parse_double(raw: &str) -> Result<f64, Component> {
    raw.trim()
        .parse::<f64>()
        .map_err(|_| error("parsing.double.invalid", vec![Component::text(raw.trim())]))
}

fn parse_range(raw: &str) -> Result<(Option<f64>, Option<f64>), Component> {
    let raw = raw.trim();
    let (min, max) = match raw.split_once("..") {
        Some((min, max)) => (min, max),
        None => (raw, raw),
    };
    let bound = |text: &str| -> Result<Option<f64>, Component> {
        if text.is_empty() {
            Ok(None)
        } else {
            parse_double(text).map(Some)
        }
    };
    let (min, max) = (bound(min)?, bound(max)?);
    if min.is_none() && max.is_none() {
        return Err(error("argument.range.empty", Vec::new()));
    }
    if let (Some(a), Some(b)) = (min, max)
        && a > b
    {
        return Err(error("argument.range.swapped", Vec::new()));
    }
    Ok((min, max))
}

fn split_options(body: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut quote: Option<char> = None;
    let mut start = 0;
    let mut escaped = false;
    for (index, c) in body.char_indices() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => quote = Some(c),
            ',' => {
                parts.push(&body[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(&body[start..]);
    parts
}

pub fn parse(input: &str) -> Result<Selector, Component> {
    let mut selector = Selector {
        base: SelectorBase::Search,
        limit: usize::MAX,
        sort: Sort::Arbitrary,
        includes_entities: true,
        alive_only: false,
        types: Vec::new(),
        distance: None,
        names: Vec::new(),
        tags: Vec::new(),
        x: None,
        y: None,
        z: None,
        dx: None,
        dy: None,
        dz: None,
    };
    let Some(rest) = input.strip_prefix('@') else {
        if let Some(uuid) = parse_uuid(input) {
            selector.base = SelectorBase::Uuid(uuid);
        } else {
            selector.base = SelectorBase::Name(input.to_owned());
            selector.includes_entities = false;
        }
        selector.limit = 1;
        return Ok(selector);
    };
    let mut chars = rest.chars();
    let Some(kind) = chars.next() else {
        return Err(error("argument.entity.selector.missing", Vec::new()));
    };
    let options = chars.as_str();
    let player = TypeMatch::Kind(EntityKind::Player);
    match kind {
        'a' => {
            selector.includes_entities = false;
            selector.types.push((false, player));
        }
        'e' => selector.alive_only = true,
        'n' => {
            selector.limit = 1;
            selector.sort = Sort::Nearest;
            selector.alive_only = true;
        }
        'p' => {
            selector.limit = 1;
            selector.sort = Sort::Nearest;
            selector.includes_entities = false;
            selector.types.push((false, player));
        }
        'r' => {
            selector.limit = 1;
            selector.sort = Sort::Random;
            selector.includes_entities = false;
            selector.types.push((false, player));
        }
        's' => {
            selector.limit = 1;
            selector.base = SelectorBase::Sender;
        }
        other => {
            return Err(error(
                "argument.entity.selector.unknown",
                vec![Component::text(format!("@{other}"))],
            ));
        }
    }
    if options.is_empty() {
        return Ok(selector);
    }
    let Some(body) = options
        .strip_prefix('[')
        .and_then(|body| body.strip_suffix(']'))
    else {
        return Err(error("argument.entity.options.unterminated", Vec::new()));
    };
    if body.trim().is_empty() {
        return Ok(selector);
    }
    for option in split_options(body) {
        let Some((name, value)) = option.split_once('=') else {
            let name = option.trim();
            if name.is_empty() {
                return Err(error("argument.entity.options.unterminated", Vec::new()));
            }
            return Err(error(
                "argument.entity.options.valueless",
                vec![Component::text(name)],
            ));
        };
        let name = name.trim();
        let value = value.trim();
        let (negated, plain) = match value.strip_prefix('!') {
            Some(rest) => (true, rest.trim()),
            None => (false, value),
        };
        let inapplicable = || {
            error(
                "argument.entity.options.inapplicable",
                vec![Component::text(name)],
            )
        };
        match name {
            "type" => {
                let kind_has_type = selector.types.iter().any(|(n, _)| !n);
                if !negated && kind_has_type {
                    return Err(inapplicable());
                }
                let matcher = if let Some(tag) = plain.strip_prefix('#') {
                    TypeMatch::Tag(tag.to_owned())
                } else {
                    let id = if plain.contains(':') {
                        plain.to_owned()
                    } else {
                        format!("minecraft:{plain}")
                    };
                    match EntityKind::from_name(&id) {
                        Some(kind) => TypeMatch::Kind(kind),
                        None => {
                            return Err(error(
                                "argument.entity.options.type.invalid",
                                vec![Component::text(id)],
                            ));
                        }
                    }
                };
                if !negated && matcher == TypeMatch::Kind(EntityKind::Player) {
                    selector.includes_entities = false;
                }
                selector.types.push((negated, matcher));
            }
            "distance" => {
                if selector.distance.is_some() {
                    return Err(inapplicable());
                }
                let range = parse_range(value)?;
                if range.0.is_some_and(|v| v < 0.0) || range.1.is_some_and(|v| v < 0.0) {
                    return Err(error(
                        "argument.entity.options.distance.negative",
                        Vec::new(),
                    ));
                }
                selector.distance = Some(range);
            }
            "limit" => {
                if matches!(selector.base, SelectorBase::Sender) {
                    return Err(inapplicable());
                }
                let parsed: i64 = value
                    .parse()
                    .map_err(|_| error("parsing.int.invalid", vec![Component::text(value)]))?;
                if parsed < 1 {
                    return Err(error("argument.entity.options.limit.toosmall", Vec::new()));
                }
                selector.limit = parsed as usize;
            }
            "sort" => {
                if matches!(selector.base, SelectorBase::Sender) {
                    return Err(inapplicable());
                }
                selector.sort = match value {
                    "nearest" => Sort::Nearest,
                    "furthest" => Sort::Furthest,
                    "random" => Sort::Random,
                    "arbitrary" => Sort::Arbitrary,
                    other => {
                        return Err(error(
                            "argument.entity.options.sort.irreversible",
                            vec![Component::text(other)],
                        ));
                    }
                };
            }
            "name" => {
                if !negated && selector.names.iter().any(|(n, _)| !n) {
                    return Err(inapplicable());
                }
                selector.names.push((negated, unquote(plain)));
            }
            "tag" => selector.tags.push((negated, unquote(plain))),
            "x" => selector.x = Some(parse_double(value)?),
            "y" => selector.y = Some(parse_double(value)?),
            "z" => selector.z = Some(parse_double(value)?),
            "dx" => selector.dx = Some(parse_double(value)?),
            "dy" => selector.dy = Some(parse_double(value)?),
            "dz" => selector.dz = Some(parse_double(value)?),
            other => {
                return Err(error(
                    "argument.entity.options.unknown",
                    vec![Component::text(other)],
                ));
            }
        }
    }
    Ok(selector)
}

impl Selector {
    pub fn single(&self) -> bool {
        self.limit <= 1
    }

    pub fn players_only(&self) -> bool {
        !self.includes_entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bases_follow_vanilla_defaults() {
        let e = parse("@e").unwrap();
        assert!(e.includes_entities && e.alive_only && e.limit == usize::MAX);
        let n = parse("@n").unwrap();
        assert_eq!((n.limit, n.sort), (1, Sort::Nearest));
        let p = parse("@p").unwrap();
        assert!(p.players_only() && !p.alive_only);
        assert_eq!(
            parse("Alice").unwrap().base,
            SelectorBase::Name("Alice".into())
        );
        assert!(parse("@x").is_err());
    }

    #[test]
    fn options_parse_with_negation_ranges_and_quotes() {
        let s = parse(r#"@e[type=cow, distance=..5,limit=2,sort=furthest,name="Big Bob",tag=!a,tag=,x=1.5,dz=-3]"#)
            .unwrap();
        assert_eq!(s.types, vec![(false, TypeMatch::Kind(EntityKind::Cow))]);
        assert_eq!(s.distance, Some((None, Some(5.0))));
        assert_eq!((s.limit, s.sort), (2, Sort::Furthest));
        assert_eq!(s.names, vec![(false, "Big Bob".into())]);
        assert_eq!(s.tags, vec![(true, "a".into()), (false, String::new())]);
        assert_eq!((s.x, s.dz), (Some(1.5), Some(-3.0)));
        assert!(parse("@e[type=player]").unwrap().players_only());
        assert!(parse("@e[type=nope]").is_err());
        assert!(parse("@e[limit=0]").is_err());
        assert!(parse("@e[distance=-1]").is_err());
        assert!(parse("@e[type=cow,type=pig]").is_err());
        assert!(parse("@e[gamemode=survival]").is_err());
        assert!(parse("@e[type=cow").is_err());
    }

    #[test]
    fn arguments_split_outside_brackets_and_quotes() {
        assert_eq!(
            split_argument("@e[type=cow, limit=1] rest"),
            ("@e[type=cow, limit=1]", " rest")
        );
        assert_eq!(split_argument("  Alice"), ("Alice", ""));
        assert_eq!(split_argument("{a:\"b c\"} x"), ("{a:\"b c\"}", " x"));
    }
}
