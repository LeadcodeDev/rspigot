use crate::Packet;
use crate::codec::{Decode, DecodeError, Encode, MAX_LIST_LEN, decode_len, encode_len};
use crate::ids::play::clientbound as id;
use crate::play::types::{decode_var_ids, encode_var_ids};
use crate::play::wire::{
    List, Opt, Plain, UNBOUNDED, Var, read_var, wire_enum, wire_struct, write_var,
};
use crate::types::Text;

pub const MASK_NODE_TYPE: u8 = 0x03;
pub const FLAG_EXECUTABLE: u8 = 0x04;
pub const FLAG_REDIRECT: u8 = 0x08;
pub const FLAG_CUSTOM_SUGGESTIONS: u8 = 0x10;
pub const FLAG_RESTRICTED: u8 = 0x20;
pub const NODE_TYPE_ROOT: u8 = 0;
pub const NODE_TYPE_LITERAL: u8 = 1;
pub const NODE_TYPE_ARGUMENT: u8 = 2;

pub mod parser {
    pub const BOOL: i32 = 0;
    pub const FLOAT: i32 = 1;
    pub const DOUBLE: i32 = 2;
    pub const INTEGER: i32 = 3;
    pub const LONG: i32 = 4;
    pub const STRING: i32 = 5;
    pub const ENTITY: i32 = 6;
    pub const GAME_PROFILE: i32 = 7;
    pub const BLOCK_POS: i32 = 8;
    pub const COLUMN_POS: i32 = 9;
    pub const VEC3: i32 = 10;
    pub const VEC2: i32 = 11;
    pub const BLOCK_STATE: i32 = 12;
    pub const BLOCK_PREDICATE: i32 = 13;
    pub const ITEM_STACK: i32 = 14;
    pub const ITEM_PREDICATE: i32 = 15;
    pub const TEAM_COLOR: i32 = 16;
    pub const HEX_COLOR: i32 = 17;
    pub const COMPONENT: i32 = 18;
    pub const STYLE: i32 = 19;
    pub const MESSAGE: i32 = 20;
    pub const NBT_COMPOUND_TAG: i32 = 21;
    pub const NBT_TAG: i32 = 22;
    pub const NBT_PATH: i32 = 23;
    pub const OBJECTIVE: i32 = 24;
    pub const OBJECTIVE_CRITERIA: i32 = 25;
    pub const OPERATION: i32 = 26;
    pub const PARTICLE: i32 = 27;
    pub const ANGLE: i32 = 28;
    pub const ROTATION: i32 = 29;
    pub const SCOREBOARD_SLOT: i32 = 30;
    pub const SCORE_HOLDER: i32 = 31;
    pub const SWIZZLE: i32 = 32;
    pub const TEAM: i32 = 33;
    pub const ITEM_SLOT: i32 = 34;
    pub const ITEM_SLOTS: i32 = 35;
    pub const RESOURCE_LOCATION: i32 = 36;
    pub const FUNCTION: i32 = 37;
    pub const ENTITY_ANCHOR: i32 = 38;
    pub const INT_RANGE: i32 = 39;
    pub const FLOAT_RANGE: i32 = 40;
    pub const DIMENSION: i32 = 41;
    pub const GAMEMODE: i32 = 42;
    pub const TIME: i32 = 43;
    pub const RESOURCE_OR_TAG: i32 = 44;
    pub const RESOURCE_OR_TAG_KEY: i32 = 45;
    pub const RESOURCE: i32 = 46;
    pub const RESOURCE_KEY: i32 = 47;
    pub const RESOURCE_SELECTOR: i32 = 48;
    pub const TEMPLATE_MIRROR: i32 = 49;
    pub const TEMPLATE_ROTATION: i32 = 50;
    pub const HEIGHTMAP: i32 = 51;
    pub const LOOT_TABLE: i32 = 52;
    pub const LOOT_PREDICATE: i32 = 53;
    pub const LOOT_MODIFIER: i32 = 54;
    pub const CONTEXT_FLOAT_PROVIDER: i32 = 55;
    pub const CONTEXT_INT_PROVIDER: i32 = 56;
    pub const SLOT_SOURCE: i32 = 57;
    pub const DIALOG: i32 = 58;
    pub const FEATURE: i32 = 59;
    pub const SWING_ANIMATION: i32 = 60;
    pub const UUID: i32 = 61;
    pub const COUNT: i32 = 62;
}

wire_enum! {
    pub enum StringKind: Strict { SingleWord, QuotablePhrase, GreedyPhrase }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimpleParser(i32);

impl SimpleParser {
    pub const fn new(parser_id: i32) -> Option<Self> {
        match parser_id {
            parser::FLOAT
            | parser::DOUBLE
            | parser::INTEGER
            | parser::LONG
            | parser::STRING
            | parser::ENTITY
            | parser::SCORE_HOLDER
            | parser::TIME
            | parser::RESOURCE_OR_TAG
            | parser::RESOURCE_OR_TAG_KEY
            | parser::RESOURCE
            | parser::RESOURCE_KEY
            | parser::RESOURCE_SELECTOR => None,
            id if id >= 0 && id < parser::COUNT => Some(Self(id)),
            _ => None,
        }
    }

    pub const fn id(self) -> i32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentParser {
    Float { min: Option<f32>, max: Option<f32> },
    Double { min: Option<f64>, max: Option<f64> },
    Integer { min: Option<i32>, max: Option<i32> },
    Long { min: Option<i64>, max: Option<i64> },
    String(StringKind),
    Entity { single: bool, players_only: bool },
    ScoreHolder { multiple: bool },
    Time { min: i32 },
    ResourceOrTag { registry: String },
    ResourceOrTagKey { registry: String },
    Resource { registry: String },
    ResourceKey { registry: String },
    ResourceSelector { registry: String },
    Simple(SimpleParser),
}

fn range_flags<T>(min: &Option<T>, max: &Option<T>) -> u8 {
    u8::from(min.is_some()) | (u8::from(max.is_some()) << 1)
}

fn write_range<T: Encode>(min: &Option<T>, max: &Option<T>, out: &mut Vec<u8>) {
    out.push(range_flags(min, max));
    if let Some(min) = min {
        min.encode(out);
    }
    if let Some(max) = max {
        max.encode(out);
    }
}

type Range<T> = (Option<T>, Option<T>);

fn read_range<T: Decode>(input: &mut &[u8]) -> Result<Range<T>, DecodeError> {
    let flags = u8::decode(input)?;
    let min = if flags & 1 != 0 {
        Some(T::decode(input)?)
    } else {
        None
    };
    let max = if flags & 2 != 0 {
        Some(T::decode(input)?)
    } else {
        None
    };
    Ok((min, max))
}

impl ArgumentParser {
    pub const fn id(&self) -> i32 {
        match self {
            Self::Float { .. } => parser::FLOAT,
            Self::Double { .. } => parser::DOUBLE,
            Self::Integer { .. } => parser::INTEGER,
            Self::Long { .. } => parser::LONG,
            Self::String(_) => parser::STRING,
            Self::Entity { .. } => parser::ENTITY,
            Self::ScoreHolder { .. } => parser::SCORE_HOLDER,
            Self::Time { .. } => parser::TIME,
            Self::ResourceOrTag { .. } => parser::RESOURCE_OR_TAG,
            Self::ResourceOrTagKey { .. } => parser::RESOURCE_OR_TAG_KEY,
            Self::Resource { .. } => parser::RESOURCE,
            Self::ResourceKey { .. } => parser::RESOURCE_KEY,
            Self::ResourceSelector { .. } => parser::RESOURCE_SELECTOR,
            Self::Simple(simple) => simple.id(),
        }
    }
}

impl Encode for ArgumentParser {
    fn encode(&self, out: &mut Vec<u8>) {
        write_var(self.id(), out);
        match self {
            Self::Float { min, max } => write_range(min, max, out),
            Self::Double { min, max } => write_range(min, max, out),
            Self::Integer { min, max } => write_range(min, max, out),
            Self::Long { min, max } => write_range(min, max, out),
            Self::String(kind) => kind.encode(out),
            Self::Entity {
                single,
                players_only,
            } => out.push(u8::from(*single) | (u8::from(*players_only) << 1)),
            Self::ScoreHolder { multiple } => out.push(u8::from(*multiple)),
            Self::Time { min } => min.encode(out),
            Self::ResourceOrTag { registry }
            | Self::ResourceOrTagKey { registry }
            | Self::Resource { registry }
            | Self::ResourceKey { registry }
            | Self::ResourceSelector { registry } => registry.encode(out),
            Self::Simple(_) => {}
        }
    }
}

impl Decode for ArgumentParser {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let parser_id = read_var(input)?;
        Ok(match parser_id {
            parser::FLOAT => {
                let (min, max) = read_range(input)?;
                Self::Float { min, max }
            }
            parser::DOUBLE => {
                let (min, max) = read_range(input)?;
                Self::Double { min, max }
            }
            parser::INTEGER => {
                let (min, max) = read_range(input)?;
                Self::Integer { min, max }
            }
            parser::LONG => {
                let (min, max) = read_range(input)?;
                Self::Long { min, max }
            }
            parser::STRING => Self::String(StringKind::decode(input)?),
            parser::ENTITY => {
                let flags = u8::decode(input)?;
                Self::Entity {
                    single: flags & 1 != 0,
                    players_only: flags & 2 != 0,
                }
            }
            parser::SCORE_HOLDER => Self::ScoreHolder {
                multiple: u8::decode(input)? & 1 != 0,
            },
            parser::TIME => Self::Time {
                min: i32::decode(input)?,
            },
            parser::RESOURCE_OR_TAG => Self::ResourceOrTag {
                registry: String::decode(input)?,
            },
            parser::RESOURCE_OR_TAG_KEY => Self::ResourceOrTagKey {
                registry: String::decode(input)?,
            },
            parser::RESOURCE => Self::Resource {
                registry: String::decode(input)?,
            },
            parser::RESOURCE_KEY => Self::ResourceKey {
                registry: String::decode(input)?,
            },
            parser::RESOURCE_SELECTOR => Self::ResourceSelector {
                registry: String::decode(input)?,
            },
            other => Self::Simple(SimpleParser::new(other).ok_or(DecodeError::InvalidValue {
                field: "command argument type",
                value: i64::from(other),
            })?),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandNodeKind {
    Root,
    Literal {
        name: String,
    },
    Argument {
        name: String,
        parser: ArgumentParser,
        suggestions: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandNode {
    pub kind: CommandNodeKind,
    pub executable: bool,
    pub restricted: bool,
    pub children: Vec<i32>,
    pub redirect: Option<i32>,
}

impl CommandNode {
    pub fn flags(&self) -> u8 {
        let mut flags = match &self.kind {
            CommandNodeKind::Root => NODE_TYPE_ROOT,
            CommandNodeKind::Literal { .. } => NODE_TYPE_LITERAL,
            CommandNodeKind::Argument { suggestions, .. } => {
                NODE_TYPE_ARGUMENT
                    | if suggestions.is_some() {
                        FLAG_CUSTOM_SUGGESTIONS
                    } else {
                        0
                    }
            }
        };
        if self.executable {
            flags |= FLAG_EXECUTABLE;
        }
        if self.redirect.is_some() {
            flags |= FLAG_REDIRECT;
        }
        if self.restricted {
            flags |= FLAG_RESTRICTED;
        }
        flags
    }
}

impl Encode for CommandNode {
    fn encode(&self, out: &mut Vec<u8>) {
        out.push(self.flags());
        encode_var_ids(&self.children, out);
        if let Some(redirect) = self.redirect {
            write_var(redirect, out);
        }
        match &self.kind {
            CommandNodeKind::Root => {}
            CommandNodeKind::Literal { name } => name.encode(out),
            CommandNodeKind::Argument {
                name,
                parser,
                suggestions,
            } => {
                name.encode(out);
                parser.encode(out);
                if let Some(suggestions) = suggestions {
                    suggestions.encode(out);
                }
            }
        }
    }
}

impl Decode for CommandNode {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let flags = u8::decode(input)?;
        let children = decode_var_ids(input, MAX_LIST_LEN)?;
        let redirect = if flags & FLAG_REDIRECT != 0 {
            Some(read_var(input)?)
        } else {
            None
        };
        let kind = match flags & MASK_NODE_TYPE {
            NODE_TYPE_ROOT => CommandNodeKind::Root,
            NODE_TYPE_LITERAL => CommandNodeKind::Literal {
                name: String::decode(input)?,
            },
            NODE_TYPE_ARGUMENT => {
                let name = String::decode(input)?;
                let parser = ArgumentParser::decode(input)?;
                let suggestions = if flags & FLAG_CUSTOM_SUGGESTIONS != 0 {
                    Some(String::decode(input)?)
                } else {
                    None
                };
                CommandNodeKind::Argument {
                    name,
                    parser,
                    suggestions,
                }
            }
            other => {
                return Err(DecodeError::InvalidValue {
                    field: "command node type",
                    value: i64::from(other),
                });
            }
        };
        Ok(Self {
            kind,
            executable: flags & FLAG_EXECUTABLE != 0,
            restricted: flags & FLAG_RESTRICTED != 0,
            children,
            redirect,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Commands {
    pub nodes: Vec<CommandNode>,
    pub root_index: i32,
}

impl Encode for Commands {
    fn encode(&self, out: &mut Vec<u8>) {
        encode_len(self.nodes.len(), out);
        for node in &self.nodes {
            node.encode(out);
        }
        write_var(self.root_index, out);
    }
}

impl Decode for Commands {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let count = decode_len(input, MAX_LIST_LEN)?;
        let mut nodes = Vec::with_capacity(count.min(input.len()));
        for _ in 0..count {
            nodes.push(CommandNode::decode(input)?);
        }
        let root_index = read_var(input)?;
        Ok(Self { nodes, root_index })
    }
}

impl Packet for Commands {
    const ID: i32 = id::COMMANDS;
}

wire_struct! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct Suggestion {
        pub text: String,
        pub tooltip: Option<Text> as Opt<Plain>,
    }
}

wire_struct! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct CommandSuggestions = id::COMMAND_SUGGESTIONS {
        pub id: i32 as Var,
        pub start: i32 as Var,
        pub length: i32 as Var,
        pub suggestions: Vec<Suggestion> as List<Plain, UNBOUNDED>,
    }
}

#[cfg(test)]
mod tests {
    use rspigot_nbt::Tag;

    use super::*;
    use crate::codec::decode_exact;
    use crate::play::wire::testing::roundtrip;

    fn literal(name: &str, children: Vec<i32>, executable: bool) -> CommandNode {
        CommandNode {
            kind: CommandNodeKind::Literal { name: name.into() },
            executable,
            restricted: false,
            children,
            redirect: None,
        }
    }

    #[test]
    fn small_tree_golden() {
        let commands = Commands {
            nodes: vec![
                CommandNode {
                    kind: CommandNodeKind::Root,
                    executable: false,
                    restricted: false,
                    children: vec![1],
                    redirect: None,
                },
                literal("tp", vec![2], false),
                CommandNode {
                    kind: CommandNodeKind::Argument {
                        name: "target".into(),
                        parser: ArgumentParser::Entity {
                            single: true,
                            players_only: false,
                        },
                        suggestions: None,
                    },
                    executable: true,
                    restricted: true,
                    children: vec![],
                    redirect: None,
                },
            ],
            root_index: 0,
        };
        let bytes = roundtrip(&commands);
        let mut expected = vec![3, 0x00, 1, 1];
        expected.extend([0x01, 1, 2, 2, b't', b'p']);
        expected.extend([0x26, 0, 6, b't', b'a', b'r', b'g', b'e', b't', 6, 0x01]);
        expected.push(0);
        assert_eq!(bytes, expected);
    }

    #[test]
    fn redirect_and_suggestions_flags() {
        let node = CommandNode {
            kind: CommandNodeKind::Argument {
                name: "n".into(),
                parser: ArgumentParser::Integer {
                    min: Some(0),
                    max: None,
                },
                suggestions: Some("minecraft:ask_server".into()),
            },
            executable: false,
            restricted: false,
            children: vec![],
            redirect: Some(5),
        };
        assert_eq!(node.flags(), 0x02 | 0x08 | 0x10);
        let bytes = roundtrip(&node);
        assert_eq!(&bytes[..6], &[0x1A, 0, 5, 1, b'n', 3]);
        assert_eq!(&bytes[6..11], &[0x01, 0, 0, 0, 0]);
        assert_eq!(bytes[11], 20);
    }

    #[test]
    fn every_parser_with_properties_roundtrips() {
        let parsers = vec![
            ArgumentParser::Float {
                min: Some(-1.0),
                max: Some(1.0),
            },
            ArgumentParser::Double {
                min: None,
                max: Some(2.0),
            },
            ArgumentParser::Integer {
                min: None,
                max: None,
            },
            ArgumentParser::Long {
                min: Some(-5),
                max: None,
            },
            ArgumentParser::String(StringKind::GreedyPhrase),
            ArgumentParser::Entity {
                single: false,
                players_only: true,
            },
            ArgumentParser::ScoreHolder { multiple: true },
            ArgumentParser::Time { min: 0 },
            ArgumentParser::ResourceOrTag {
                registry: "minecraft:worldgen/biome".into(),
            },
            ArgumentParser::ResourceOrTagKey {
                registry: "minecraft:item".into(),
            },
            ArgumentParser::Resource {
                registry: "minecraft:enchantment".into(),
            },
            ArgumentParser::ResourceKey {
                registry: "minecraft:dimension".into(),
            },
            ArgumentParser::ResourceSelector {
                registry: "minecraft:recipe".into(),
            },
            ArgumentParser::Simple(SimpleParser::new(parser::BOOL).unwrap()),
            ArgumentParser::Simple(SimpleParser::new(parser::UUID).unwrap()),
        ];
        for parser in parsers {
            roundtrip(&parser);
        }
        assert_eq!(
            roundtrip(&ArgumentParser::String(StringKind::SingleWord)),
            vec![5, 0]
        );
        assert!(SimpleParser::new(parser::INTEGER).is_none());
        assert!(SimpleParser::new(parser::COUNT).is_none());
        assert!(decode_exact::<ArgumentParser>(&[62]).is_err());
    }

    #[test]
    fn suggestions_packet_roundtrip() {
        let packet = CommandSuggestions {
            id: 3,
            start: 1,
            length: 2,
            suggestions: vec![
                Suggestion {
                    text: "help".into(),
                    tooltip: None,
                },
                Suggestion {
                    text: "home".into(),
                    tooltip: Some(Text(Tag::String("go home".into()))),
                },
            ],
        };
        let bytes = roundtrip(&packet);
        assert_eq!(&bytes[..10], &[3, 1, 2, 2, 4, b'h', b'e', b'l', b'p', 0]);
    }

    #[test]
    fn invalid_node_type_is_rejected() {
        assert!(decode_exact::<CommandNode>(&[0x03, 0]).is_err());
    }
}
