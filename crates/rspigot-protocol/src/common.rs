use std::fmt;

use rspigot_nbt::Tag;
use uuid::Uuid;

use crate::codec::{
    Decode, DecodeError, Encode, MAX_STRING_CHARS, NetworkNbt, VarInt, decode_len, decode_string,
    encode_len, take,
};
use crate::types::Text;

pub const UNBOUNDED: usize = i32::MAX as usize;
pub const MAX_CLIENTBOUND_PAYLOAD_LEN: usize = 1_048_576;
pub const MAX_SERVERBOUND_PAYLOAD_LEN: usize = 32_767;
pub const MAX_COOKIE_LEN: usize = 5_120;
pub const MAX_CLICK_ACTION_PAYLOAD_LEN: usize = 65_536;
pub const MAX_LANGUAGE_CHARS: usize = 16;
pub const MAX_RESOURCE_PACK_HASH_CHARS: usize = 40;
pub const MAX_REPORT_DETAILS: usize = 32;
pub const MAX_REPORT_DETAIL_KEY_CHARS: usize = 128;
pub const MAX_REPORT_DETAIL_VALUE_CHARS: usize = 4096;

const DEFAULT_NAMESPACE: &str = "minecraft";

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier(String);

impl Identifier {
    pub fn parse(text: &str) -> Result<Self, DecodeError> {
        let (namespace, path) = match text.split_once(':') {
            Some(("", path)) => (DEFAULT_NAMESPACE, path),
            Some((namespace, path)) => (namespace, path),
            None => (DEFAULT_NAMESPACE, text),
        };
        if namespace == ".." || !namespace.chars().all(is_namespace_char) {
            return Err(invalid_identifier(namespace));
        }
        if !path.chars().all(is_path_char) {
            return Err(invalid_identifier(path));
        }
        Ok(Self(format!("{namespace}:{path}")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn namespace(&self) -> &str {
        self.0
            .split_once(':')
            .map_or(DEFAULT_NAMESPACE, |(ns, _)| ns)
    }

    pub fn path(&self) -> &str {
        self.0
            .split_once(':')
            .map_or(self.0.as_str(), |(_, path)| path)
    }
}

fn invalid_identifier(part: &str) -> DecodeError {
    DecodeError::InvalidValue {
        field: "identifier",
        value: part.len() as i64,
    }
}

fn is_namespace_char(c: char) -> bool {
    matches!(c, '_' | '-' | 'a'..='z' | '0'..='9' | '.')
}

fn is_path_char(c: char) -> bool {
    is_namespace_char(c) || c == '/'
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Encode for Identifier {
    fn encode(&self, out: &mut Vec<u8>) {
        self.0.encode(out);
    }
}

impl Decode for Identifier {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Self::parse(&decode_string(input, MAX_STRING_CHARS)?)
    }
}

pub fn encode_bytes(bytes: &[u8], out: &mut Vec<u8>) {
    encode_len(bytes.len(), out);
    out.extend_from_slice(bytes);
}

pub fn decode_bytes(input: &mut &[u8], max: usize) -> Result<Vec<u8>, DecodeError> {
    let len = decode_len(input, max)?;
    Ok(take(input, len)?.to_vec())
}

pub fn decode_remaining(input: &mut &[u8], max: usize) -> Result<Vec<u8>, DecodeError> {
    if input.len() > max {
        return Err(DecodeError::TooLong {
            len: input.len(),
            max,
        });
    }
    Ok(std::mem::take(input).to_vec())
}

pub fn encode_tag(tag: &Tag, out: &mut Vec<u8>) {
    rspigot_nbt::write_network(out, tag);
}

pub fn decode_tag(input: &mut &[u8]) -> Result<Tag, DecodeError> {
    NetworkNbt::decode(input)?
        .0
        .ok_or(DecodeError::InvalidValue {
            field: "nbt",
            value: 0,
        })
}

pub fn decode_list<T: Decode>(input: &mut &[u8], max: usize) -> Result<Vec<T>, DecodeError> {
    decode_list_with(input, max, T::decode)
}

pub fn decode_list_with<T>(
    input: &mut &[u8],
    max: usize,
    mut decode_item: impl FnMut(&mut &[u8]) -> Result<T, DecodeError>,
) -> Result<Vec<T>, DecodeError> {
    let len = decode_len(input, max)?;
    let mut items = Vec::with_capacity(len.min(input.len()));
    for _ in 0..len {
        items.push(decode_item(input)?);
    }
    Ok(items)
}

fn wrapped_id(id: i32, count: i32) -> usize {
    id.rem_euclid(count) as usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeepAlive {
    pub id: i64,
}

impl Encode for KeepAlive {
    fn encode(&self, out: &mut Vec<u8>) {
        self.id.encode(out);
    }
}

impl Decode for KeepAlive {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            id: i64::decode(input)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ping {
    pub id: i32,
}

impl Encode for Ping {
    fn encode(&self, out: &mut Vec<u8>) {
        self.id.encode(out);
    }
}

impl Decode for Ping {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            id: i32::decode(input)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pong {
    pub id: i32,
}

impl Encode for Pong {
    fn encode(&self, out: &mut Vec<u8>) {
        self.id.encode(out);
    }
}

impl Decode for Pong {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            id: i32::decode(input)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomPayload {
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl Encode for ClientboundCustomPayload {
    fn encode(&self, out: &mut Vec<u8>) {
        self.channel.encode(out);
        out.extend_from_slice(&self.data);
    }
}

impl Decode for ClientboundCustomPayload {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            channel: Identifier::decode(input)?,
            data: decode_remaining(input, MAX_CLIENTBOUND_PAYLOAD_LEN)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCustomPayload {
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl Encode for ServerboundCustomPayload {
    fn encode(&self, out: &mut Vec<u8>) {
        self.channel.encode(out);
        out.extend_from_slice(&self.data);
    }
}

impl Decode for ServerboundCustomPayload {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            channel: Identifier::decode(input)?,
            data: decode_remaining(input, MAX_SERVERBOUND_PAYLOAD_LEN)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Disconnect {
    pub reason: Text,
}

impl Encode for Disconnect {
    fn encode(&self, out: &mut Vec<u8>) {
        self.reason.encode(out);
    }
}

impl Decode for Disconnect {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            reason: Text::decode(input)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

impl ChatVisibility {
    const ALL: [Self; 3] = [Self::Full, Self::System, Self::Hidden];

    pub const fn id(self) -> i32 {
        self as i32
    }

    pub fn from_id(id: i32) -> Self {
        Self::ALL[wrapped_id(id, Self::ALL.len() as i32)]
    }
}

impl Encode for ChatVisibility {
    fn encode(&self, out: &mut Vec<u8>) {
        VarInt(self.id()).encode(out);
    }
}

impl Decode for ChatVisibility {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_id(VarInt::decode(input)?.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainHand {
    Left,
    Right,
}

impl MainHand {
    pub const fn id(self) -> i32 {
        self as i32
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Right,
            _ => Self::Left,
        }
    }
}

impl Encode for MainHand {
    fn encode(&self, out: &mut Vec<u8>) {
        VarInt(self.id()).encode(out);
    }
}

impl Decode for MainHand {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_id(VarInt::decode(input)?.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleStatus {
    All,
    Decreased,
    Minimal,
}

impl ParticleStatus {
    const ALL: [Self; 3] = [Self::All, Self::Decreased, Self::Minimal];

    pub const fn id(self) -> i32 {
        self as i32
    }

    pub fn from_id(id: i32) -> Self {
        Self::ALL[wrapped_id(id, Self::ALL.len() as i32)]
    }
}

impl Encode for ParticleStatus {
    fn encode(&self, out: &mut Vec<u8>) {
        VarInt(self.id()).encode(out);
    }
}

impl Decode for ParticleStatus {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self::from_id(VarInt::decode(input)?.0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientInformation {
    pub language: String,
    pub view_distance: i8,
    pub chat_visibility: ChatVisibility,
    pub chat_colors: bool,
    pub model_customisation: u8,
    pub main_hand: MainHand,
    pub text_filtering_enabled: bool,
    pub allows_listing: bool,
    pub particle_status: ParticleStatus,
}

impl Encode for ClientInformation {
    fn encode(&self, out: &mut Vec<u8>) {
        self.language.encode(out);
        self.view_distance.encode(out);
        self.chat_visibility.encode(out);
        self.chat_colors.encode(out);
        self.model_customisation.encode(out);
        self.main_hand.encode(out);
        self.text_filtering_enabled.encode(out);
        self.allows_listing.encode(out);
        self.particle_status.encode(out);
    }
}

impl Decode for ClientInformation {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            language: decode_string(input, MAX_LANGUAGE_CHARS)?,
            view_distance: i8::decode(input)?,
            chat_visibility: ChatVisibility::decode(input)?,
            chat_colors: bool::decode(input)?,
            model_customisation: u8::decode(input)?,
            main_hand: MainHand::decode(input)?,
            text_filtering_enabled: bool::decode(input)?,
            allows_listing: bool::decode(input)?,
            particle_status: ParticleStatus::decode(input)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookieRequest {
    pub key: Identifier,
}

impl Encode for CookieRequest {
    fn encode(&self, out: &mut Vec<u8>) {
        self.key.encode(out);
    }
}

impl Decode for CookieRequest {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            key: Identifier::decode(input)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookieResponse {
    pub key: Identifier,
    pub payload: Option<Vec<u8>>,
}

impl Encode for CookieResponse {
    fn encode(&self, out: &mut Vec<u8>) {
        self.key.encode(out);
        match &self.payload {
            Some(payload) => {
                true.encode(out);
                encode_bytes(payload, out);
            }
            None => false.encode(out),
        }
    }
}

impl Decode for CookieResponse {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let key = Identifier::decode(input)?;
        let payload = if bool::decode(input)? {
            Some(decode_bytes(input, MAX_COOKIE_LEN)?)
        } else {
            None
        };
        Ok(Self { key, payload })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreCookie {
    pub key: Identifier,
    pub payload: Vec<u8>,
}

impl Encode for StoreCookie {
    fn encode(&self, out: &mut Vec<u8>) {
        self.key.encode(out);
        encode_bytes(&self.payload, out);
    }
}

impl Decode for StoreCookie {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            key: Identifier::decode(input)?,
            payload: decode_bytes(input, MAX_COOKIE_LEN)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    pub host: String,
    pub port: i32,
}

impl Encode for Transfer {
    fn encode(&self, out: &mut Vec<u8>) {
        self.host.encode(out);
        VarInt(self.port).encode(out);
    }
}

impl Decode for Transfer {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            host: String::decode(input)?,
            port: VarInt::decode(input)?.0,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResourcePackPush {
    pub id: Uuid,
    pub url: String,
    pub hash: String,
    pub required: bool,
    pub prompt: Option<Text>,
}

impl Encode for ResourcePackPush {
    fn encode(&self, out: &mut Vec<u8>) {
        self.id.encode(out);
        self.url.encode(out);
        self.hash.encode(out);
        self.required.encode(out);
        self.prompt.encode(out);
    }
}

impl Decode for ResourcePackPush {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            id: Uuid::decode(input)?,
            url: String::decode(input)?,
            hash: decode_string(input, MAX_RESOURCE_PACK_HASH_CHARS)?,
            required: bool::decode(input)?,
            prompt: Option::<Text>::decode(input)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourcePackPop {
    pub id: Option<Uuid>,
}

impl Encode for ResourcePackPop {
    fn encode(&self, out: &mut Vec<u8>) {
        self.id.encode(out);
    }
}

impl Decode for ResourcePackPop {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            id: Option::<Uuid>::decode(input)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourcePackAction {
    SuccessfullyLoaded,
    Declined,
    FailedDownload,
    Accepted,
    Downloaded,
    InvalidUrl,
    FailedReload,
    Discarded,
}

impl ResourcePackAction {
    const ALL: [Self; 8] = [
        Self::SuccessfullyLoaded,
        Self::Declined,
        Self::FailedDownload,
        Self::Accepted,
        Self::Downloaded,
        Self::InvalidUrl,
        Self::FailedReload,
        Self::Discarded,
    ];

    pub const fn id(self) -> i32 {
        self as i32
    }

    pub fn from_id(id: i32) -> Result<Self, DecodeError> {
        usize::try_from(id)
            .ok()
            .and_then(|index| Self::ALL.get(index).copied())
            .ok_or(DecodeError::InvalidValue {
                field: "resource pack action",
                value: i64::from(id),
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourcePackResponse {
    pub id: Uuid,
    pub action: ResourcePackAction,
}

impl Encode for ResourcePackResponse {
    fn encode(&self, out: &mut Vec<u8>) {
        self.id.encode(out);
        VarInt(self.action.id()).encode(out);
    }
}

impl Decode for ResourcePackResponse {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            id: Uuid::decode(input)?,
            action: ResourcePackAction::from_id(VarInt::decode(input)?.0)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkTag {
    pub name: Identifier,
    pub entries: Vec<i32>,
}

impl Encode for NetworkTag {
    fn encode(&self, out: &mut Vec<u8>) {
        self.name.encode(out);
        encode_len(self.entries.len(), out);
        for entry in &self.entries {
            VarInt(*entry).encode(out);
        }
    }
}

impl Decode for NetworkTag {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            name: Identifier::decode(input)?,
            entries: decode_list_with(input, UNBOUNDED, |input| Ok(VarInt::decode(input)?.0))?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryTags {
    pub registry: Identifier,
    pub tags: Vec<NetworkTag>,
}

impl Encode for RegistryTags {
    fn encode(&self, out: &mut Vec<u8>) {
        self.registry.encode(out);
        self.tags.encode(out);
    }
}

impl Decode for RegistryTags {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            registry: Identifier::decode(input)?,
            tags: decode_list(input, UNBOUNDED)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateTags {
    pub registries: Vec<RegistryTags>,
}

impl Encode for UpdateTags {
    fn encode(&self, out: &mut Vec<u8>) {
        self.registries.encode(out);
    }
}

impl Decode for UpdateTags {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            registries: decode_list(input, UNBOUNDED)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomReportDetails {
    pub details: Vec<(String, String)>,
}

impl Encode for CustomReportDetails {
    fn encode(&self, out: &mut Vec<u8>) {
        encode_len(self.details.len(), out);
        for (key, value) in &self.details {
            key.encode(out);
            value.encode(out);
        }
    }
}

impl Decode for CustomReportDetails {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            details: decode_list_with(input, MAX_REPORT_DETAILS, |input| {
                Ok((
                    decode_string(input, MAX_REPORT_DETAIL_KEY_CHARS)?,
                    decode_string(input, MAX_REPORT_DETAIL_VALUE_CHARS)?,
                ))
            })?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownLinkType {
    BugReport,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
}

impl KnownLinkType {
    const ALL: [Self; 10] = [
        Self::BugReport,
        Self::CommunityGuidelines,
        Self::Support,
        Self::Status,
        Self::Feedback,
        Self::Community,
        Self::Website,
        Self::Forums,
        Self::News,
        Self::Announcements,
    ];

    pub const fn id(self) -> i32 {
        self as i32
    }

    pub fn from_id(id: i32) -> Self {
        usize::try_from(id)
            .ok()
            .and_then(|index| Self::ALL.get(index).copied())
            .unwrap_or(Self::BugReport)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerLinkLabel {
    Known(KnownLinkType),
    Custom(Text),
}

impl Encode for ServerLinkLabel {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Known(kind) => {
                true.encode(out);
                VarInt(kind.id()).encode(out);
            }
            Self::Custom(text) => {
                false.encode(out);
                text.encode(out);
            }
        }
    }
}

impl Decode for ServerLinkLabel {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        if bool::decode(input)? {
            Ok(Self::Known(KnownLinkType::from_id(
                VarInt::decode(input)?.0,
            )))
        } else {
            Ok(Self::Custom(Text::decode(input)?))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerLink {
    pub label: ServerLinkLabel,
    pub url: String,
}

impl Encode for ServerLink {
    fn encode(&self, out: &mut Vec<u8>) {
        self.label.encode(out);
        self.url.encode(out);
    }
}

impl Decode for ServerLink {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            label: ServerLinkLabel::decode(input)?,
            url: String::decode(input)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerLinks {
    pub links: Vec<ServerLink>,
}

impl Encode for ServerLinks {
    fn encode(&self, out: &mut Vec<u8>) {
        self.links.encode(out);
    }
}

impl Decode for ServerLinks {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            links: decode_list(input, UNBOUNDED)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShowDialog {
    pub dialog: Tag,
}

impl Encode for ShowDialog {
    fn encode(&self, out: &mut Vec<u8>) {
        encode_tag(&self.dialog, out);
    }
}

impl Decode for ShowDialog {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            dialog: decode_tag(input)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DialogHolder {
    Registry(i32),
    Direct(Tag),
}

impl Encode for DialogHolder {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Registry(id) => VarInt(id.wrapping_add(1)).encode(out),
            Self::Direct(dialog) => {
                VarInt(0).encode(out);
                encode_tag(dialog, out);
            }
        }
    }
}

impl Decode for DialogHolder {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        match VarInt::decode(input)?.0 {
            0 => Ok(Self::Direct(decode_tag(input)?)),
            id => Ok(Self::Registry(id.wrapping_sub(1))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShowDialogHolder {
    pub dialog: DialogHolder,
}

impl Encode for ShowDialogHolder {
    fn encode(&self, out: &mut Vec<u8>) {
        self.dialog.encode(out);
    }
}

impl Decode for ShowDialogHolder {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            dialog: DialogHolder::decode(input)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClearDialog;

impl Encode for ClearDialog {
    fn encode(&self, _out: &mut Vec<u8>) {}
}

impl Decode for ClearDialog {
    fn decode(_input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostEffects {
    pub effects: Vec<Identifier>,
}

impl Encode for PostEffects {
    fn encode(&self, out: &mut Vec<u8>) {
        self.effects.encode(out);
    }
}

impl Decode for PostEffects {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            effects: decode_list(input, UNBOUNDED)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomClickAction {
    pub id: Identifier,
    pub payload: Option<Tag>,
}

impl Encode for CustomClickAction {
    fn encode(&self, out: &mut Vec<u8>) {
        self.id.encode(out);
        let mut payload = Vec::new();
        match &self.payload {
            Some(tag) => encode_tag(tag, &mut payload),
            None => payload.push(0),
        }
        encode_bytes(&payload, out);
    }
}

impl Decode for CustomClickAction {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let id = Identifier::decode(input)?;
        let len = decode_len(input, MAX_CLICK_ACTION_PAYLOAD_LEN)?;
        let mut payload = take(input, len)?;
        Ok(Self {
            id,
            payload: NetworkNbt::decode(&mut payload)?.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use rspigot_nbt::{Compound, Tag};
    use uuid::Uuid;

    use super::*;
    use crate::codec::{Decode, DecodeError, Encode, decode_exact};
    use crate::types::Text;

    fn bytes<T: Encode>(value: &T) -> Vec<u8> {
        let mut out = Vec::new();
        value.encode(&mut out);
        out
    }

    fn roundtrip<T: Encode + Decode + PartialEq + std::fmt::Debug>(value: &T) -> Vec<u8> {
        let out = bytes(value);
        assert_eq!(&decode_exact::<T>(&out).unwrap(), value);
        out
    }

    fn string(text: &str) -> Vec<u8> {
        let mut out = vec![text.len() as u8];
        out.extend_from_slice(text.as_bytes());
        out
    }

    fn id(text: &str) -> Identifier {
        Identifier::parse(text).unwrap()
    }

    fn uuid() -> Uuid {
        Uuid::from_u128(0x0011_2233_4455_6677_8899_AABB_CCDD_EEFF)
    }

    #[test]
    fn identifier_parsing_follows_vanilla_rules() {
        assert_eq!(id("brand").as_str(), "minecraft:brand");
        assert_eq!(id(":brand").as_str(), "minecraft:brand");
        assert_eq!(id("rspigot:a/b.c-d_e").as_str(), "rspigot:a/b.c-d_e");
        assert_eq!(id("rspigot:a/b").namespace(), "rspigot");
        assert_eq!(id("rspigot:a/b").path(), "a/b");
        assert_eq!(id("minecraft:").path(), "");
        for invalid in ["Upper:case", "a:b:c", "..:x", "a b", "ns/x:y", "x:é"] {
            assert!(Identifier::parse(invalid).is_err(), "{invalid}");
        }
        assert_eq!(id("dot.ns:x").namespace(), "dot.ns");
    }

    #[test]
    fn identifier_decodes_and_normalizes() {
        assert_eq!(
            decode_exact::<Identifier>(&string("brand")).unwrap(),
            id("minecraft:brand")
        );
        assert_eq!(roundtrip(&id("minecraft:brand")), string("minecraft:brand"));
        assert!(decode_exact::<Identifier>(&string("BAD")).is_err());
    }

    #[test]
    fn keep_alive_ping_pong_layouts() {
        assert_eq!(
            roundtrip(&KeepAlive { id: 1 }),
            vec![0, 0, 0, 0, 0, 0, 0, 1]
        );
        assert_eq!(roundtrip(&Ping { id: -2 }), vec![0xFF, 0xFF, 0xFF, 0xFE]);
        assert_eq!(roundtrip(&Pong { id: 258 }), vec![0, 0, 1, 2]);
    }

    #[test]
    fn custom_payload_carries_raw_remaining_bytes() {
        let clientbound = ClientboundCustomPayload {
            channel: id("minecraft:brand"),
            data: string("vanilla"),
        };
        let mut expected = string("minecraft:brand");
        expected.extend(string("vanilla"));
        assert_eq!(roundtrip(&clientbound), expected);
        let serverbound = ServerboundCustomPayload {
            channel: id("minecraft:brand"),
            data: string("vanilla"),
        };
        assert_eq!(roundtrip(&serverbound), expected);
    }

    #[test]
    fn custom_payload_size_limits_differ_per_direction() {
        let mut body = string("minecraft:x");
        body.extend(vec![0u8; MAX_SERVERBOUND_PAYLOAD_LEN + 1]);
        assert!(decode_exact::<ServerboundCustomPayload>(&body).is_err());
        assert!(decode_exact::<ClientboundCustomPayload>(&body).is_ok());
        let mut huge = string("minecraft:x");
        huge.extend(vec![0u8; MAX_CLIENTBOUND_PAYLOAD_LEN + 1]);
        assert!(decode_exact::<ClientboundCustomPayload>(&huge).is_err());
    }

    #[test]
    fn disconnect_is_nbt_text() {
        let packet = Disconnect {
            reason: Text(Tag::String(String::from("bye"))),
        };
        assert_eq!(roundtrip(&packet), vec![0x08, 0x00, 0x03, b'b', b'y', b'e']);
    }

    #[test]
    fn client_information_layout() {
        let info = ClientInformation {
            language: String::from("en_us"),
            view_distance: 10,
            chat_visibility: ChatVisibility::Full,
            chat_colors: true,
            model_customisation: 0x7F,
            main_hand: MainHand::Right,
            text_filtering_enabled: false,
            allows_listing: true,
            particle_status: ParticleStatus::Minimal,
        };
        let mut expected = string("en_us");
        expected.extend([0x0A, 0x00, 0x01, 0x7F, 0x01, 0x00, 0x01, 0x02]);
        assert_eq!(roundtrip(&info), expected);
    }

    #[test]
    fn client_information_enums_map_out_of_range_like_vanilla() {
        let mut body = string("en_us");
        body.extend([0x0A, 0x04, 0x01, 0x7F, 0x05, 0x00, 0x01]);
        body.extend([0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        let info = decode_exact::<ClientInformation>(&body).unwrap();
        assert_eq!(info.chat_visibility, ChatVisibility::System);
        assert_eq!(info.main_hand, MainHand::Left);
        assert_eq!(info.particle_status, ParticleStatus::Minimal);
        let mut long_language = string("abcdefghijklmnopq");
        long_language.extend([0x0A, 0x00, 0x01, 0x7F, 0x01, 0x00, 0x01, 0x00]);
        assert!(decode_exact::<ClientInformation>(&long_language).is_err());
    }

    #[test]
    fn cookie_packets_layout() {
        let key = id("rspigot:session");
        let mut key_bytes = string("rspigot:session");
        assert_eq!(roundtrip(&CookieRequest { key: key.clone() }), key_bytes);
        let mut absent = key_bytes.clone();
        absent.push(0x00);
        assert_eq!(
            roundtrip(&CookieResponse {
                key: key.clone(),
                payload: None
            }),
            absent
        );
        let mut present = key_bytes.clone();
        present.extend([0x01, 0x02, 0xAA, 0xBB]);
        assert_eq!(
            roundtrip(&CookieResponse {
                key: key.clone(),
                payload: Some(vec![0xAA, 0xBB])
            }),
            present
        );
        key_bytes.extend([0x02, 0xAA, 0xBB]);
        assert_eq!(
            roundtrip(&StoreCookie {
                key,
                payload: vec![0xAA, 0xBB]
            }),
            key_bytes
        );
    }

    #[test]
    fn cookie_payload_is_limited_to_5120_bytes() {
        let mut body = string("rspigot:c");
        body.push(0x01);
        crate::codec::VarInt(5121).encode(&mut body);
        body.extend(vec![0u8; 5121]);
        assert!(matches!(
            decode_exact::<CookieResponse>(&body),
            Err(DecodeError::TooLong {
                len: 5121,
                max: 5120
            })
        ));
    }

    #[test]
    fn transfer_layout() {
        let mut expected = string("example.org");
        expected.extend([0xDD, 0xC7, 0x01]);
        assert_eq!(
            roundtrip(&Transfer {
                host: String::from("example.org"),
                port: 25565
            }),
            expected
        );
    }

    #[test]
    fn resource_pack_packets_layout() {
        let hash = "0123456789abcdef0123456789abcdef01234567";
        let push = ResourcePackPush {
            id: uuid(),
            url: String::from("https://x/p.zip"),
            hash: String::from(hash),
            required: true,
            prompt: Some(Text(Tag::String(String::from("hi")))),
        };
        let mut expected = uuid().as_bytes().to_vec();
        expected.extend(string("https://x/p.zip"));
        expected.extend(string(hash));
        expected.extend([0x01, 0x01, 0x08, 0x00, 0x02, b'h', b'i']);
        assert_eq!(roundtrip(&push), expected);
        roundtrip(&ResourcePackPush {
            prompt: None,
            required: false,
            ..push
        });
        let mut pop = vec![0x01];
        pop.extend_from_slice(uuid().as_bytes());
        assert_eq!(roundtrip(&ResourcePackPop { id: Some(uuid()) }), pop);
        assert_eq!(roundtrip(&ResourcePackPop { id: None }), vec![0x00]);
        let mut response = uuid().as_bytes().to_vec();
        response.push(0x07);
        assert_eq!(
            roundtrip(&ResourcePackResponse {
                id: uuid(),
                action: ResourcePackAction::Discarded
            }),
            response
        );
        response.pop();
        response.push(0x08);
        assert!(decode_exact::<ResourcePackResponse>(&response).is_err());
    }

    #[test]
    fn resource_pack_hash_is_limited_to_40_chars() {
        let mut body = uuid().as_bytes().to_vec();
        body.extend(string("u"));
        body.extend(string(&"a".repeat(41)));
        body.extend([0x00, 0x00]);
        assert!(decode_exact::<ResourcePackPush>(&body).is_err());
    }

    #[test]
    fn update_tags_layout() {
        let packet = UpdateTags {
            registries: vec![RegistryTags {
                registry: id("minecraft:block"),
                tags: vec![NetworkTag {
                    name: id("minecraft:logs"),
                    entries: vec![1, 300],
                }],
            }],
        };
        let mut expected = vec![0x01];
        expected.extend(string("minecraft:block"));
        expected.push(0x01);
        expected.extend(string("minecraft:logs"));
        expected.extend([0x02, 0x01, 0xAC, 0x02]);
        assert_eq!(roundtrip(&packet), expected);
    }

    #[test]
    fn custom_report_details_layout_and_limits() {
        let packet = CustomReportDetails {
            details: vec![(String::from("k"), String::from("v"))],
        };
        assert_eq!(roundtrip(&packet), vec![0x01, 0x01, b'k', 0x01, b'v']);
        let mut too_many = vec![33];
        for _ in 0..33 {
            too_many.extend([0x01, b'k', 0x01, b'v']);
        }
        assert!(decode_exact::<CustomReportDetails>(&too_many).is_err());
        let mut long_key = vec![0x01];
        long_key.extend([0x81, 0x01]);
        long_key.extend(vec![b'k'; 129]);
        long_key.extend([0x01, b'v']);
        assert!(decode_exact::<CustomReportDetails>(&long_key).is_err());
    }

    #[test]
    fn server_links_layout() {
        let packet = ServerLinks {
            links: vec![
                ServerLink {
                    label: ServerLinkLabel::Known(KnownLinkType::Website),
                    url: String::from("https://a"),
                },
                ServerLink {
                    label: ServerLinkLabel::Custom(Text(Tag::String(String::from("x")))),
                    url: String::from("https://b"),
                },
            ],
        };
        let mut expected = vec![0x02, 0x01, 0x06];
        expected.extend(string("https://a"));
        expected.extend([0x00, 0x08, 0x00, 0x01, b'x']);
        expected.extend(string("https://b"));
        assert_eq!(roundtrip(&packet), expected);
    }

    #[test]
    fn known_link_type_out_of_range_maps_to_bug_report() {
        let mut body = vec![0x01, 0x01, 0x0A];
        body.extend(string("https://a"));
        let packet = decode_exact::<ServerLinks>(&body).unwrap();
        assert_eq!(
            packet.links[0].label,
            ServerLinkLabel::Known(KnownLinkType::BugReport)
        );
    }

    #[test]
    fn dialog_packets_layout() {
        let dialog = Tag::Compound(Compound::new().with("type", "minecraft:notice"));
        let mut nbt = Vec::new();
        rspigot_nbt::write_network(&mut nbt, &dialog);
        assert_eq!(
            roundtrip(&ShowDialog {
                dialog: dialog.clone()
            }),
            nbt
        );
        assert!(decode_exact::<ShowDialog>(&[0x00]).is_err());
        assert_eq!(
            roundtrip(&ShowDialogHolder {
                dialog: DialogHolder::Registry(3)
            }),
            vec![0x04]
        );
        let mut direct = vec![0x00];
        direct.extend(&nbt);
        assert_eq!(
            roundtrip(&ShowDialogHolder {
                dialog: DialogHolder::Direct(dialog)
            }),
            direct
        );
        assert_eq!(roundtrip(&ClearDialog), Vec::<u8>::new());
    }

    #[test]
    fn post_effects_layout() {
        let packet = PostEffects {
            effects: vec![id("minecraft:spider")],
        };
        let mut expected = vec![0x01];
        expected.extend(string("minecraft:spider"));
        assert_eq!(roundtrip(&packet), expected);
    }

    #[test]
    fn custom_click_action_payload_is_length_prefixed_nbt() {
        let some = CustomClickAction {
            id: id("rspigot:click"),
            payload: Some(Tag::Int(5)),
        };
        let mut expected = string("rspigot:click");
        expected.extend([0x05, 0x03, 0x00, 0x00, 0x00, 0x05]);
        assert_eq!(roundtrip(&some), expected);
        let none = CustomClickAction {
            id: id("rspigot:click"),
            payload: None,
        };
        let mut expected = string("rspigot:click");
        expected.extend([0x01, 0x00]);
        assert_eq!(roundtrip(&none), expected);
        let mut too_long = string("rspigot:click");
        crate::codec::VarInt(65537).encode(&mut too_long);
        too_long.extend(vec![0u8; 65537]);
        assert!(decode_exact::<CustomClickAction>(&too_long).is_err());
    }
}
