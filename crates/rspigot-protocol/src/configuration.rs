use rspigot_nbt::Tag;

use crate::Packet;
use crate::codec::{Decode, DecodeError, Encode, decode_exact};
use crate::common::{
    ClientInformation, CookieResponse, CustomClickAction, Identifier, KeepAlive, Pong,
    ResourcePackResponse, ServerboundCustomPayload, UNBOUNDED, decode_list, decode_tag, encode_tag,
};
use crate::ids::configuration::{clientbound, serverbound};
use crate::types::KnownPack;

pub const MAX_SERVERBOUND_KNOWN_PACKS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientboundFinishConfiguration;

impl Encode for ClientboundFinishConfiguration {
    fn encode(&self, _out: &mut Vec<u8>) {}
}

impl Decode for ClientboundFinishConfiguration {
    fn decode(_input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self)
    }
}

impl Packet for ClientboundFinishConfiguration {
    const ID: i32 = clientbound::FINISH_CONFIGURATION;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResetChat;

impl Encode for ResetChat {
    fn encode(&self, _out: &mut Vec<u8>) {}
}

impl Decode for ResetChat {
    fn decode(_input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self)
    }
}

impl Packet for ResetChat {
    const ID: i32 = clientbound::RESET_CHAT;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryEntry {
    pub id: Identifier,
    pub data: Option<Tag>,
}

impl Encode for RegistryEntry {
    fn encode(&self, out: &mut Vec<u8>) {
        self.id.encode(out);
        match &self.data {
            Some(tag) => {
                true.encode(out);
                encode_tag(tag, out);
            }
            None => false.encode(out),
        }
    }
}

impl Decode for RegistryEntry {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let id = Identifier::decode(input)?;
        let data = if bool::decode(input)? {
            Some(decode_tag(input)?)
        } else {
            None
        };
        Ok(Self { id, data })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryData {
    pub registry: Identifier,
    pub entries: Vec<RegistryEntry>,
}

impl Encode for RegistryData {
    fn encode(&self, out: &mut Vec<u8>) {
        self.registry.encode(out);
        self.entries.encode(out);
    }
}

impl Decode for RegistryData {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            registry: Identifier::decode(input)?,
            entries: decode_list(input, UNBOUNDED)?,
        })
    }
}

impl Packet for RegistryData {
    const ID: i32 = clientbound::REGISTRY_DATA;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEnabledFeatures {
    pub features: Vec<Identifier>,
}

impl Encode for UpdateEnabledFeatures {
    fn encode(&self, out: &mut Vec<u8>) {
        self.features.encode(out);
    }
}

impl Decode for UpdateEnabledFeatures {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            features: decode_list(input, UNBOUNDED)?,
        })
    }
}

impl Packet for UpdateEnabledFeatures {
    const ID: i32 = clientbound::UPDATE_ENABLED_FEATURES;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSelectKnownPacks {
    pub packs: Vec<KnownPack>,
}

impl Encode for ClientboundSelectKnownPacks {
    fn encode(&self, out: &mut Vec<u8>) {
        self.packs.encode(out);
    }
}

impl Decode for ClientboundSelectKnownPacks {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            packs: decode_list(input, UNBOUNDED)?,
        })
    }
}

impl Packet for ClientboundSelectKnownPacks {
    const ID: i32 = clientbound::SELECT_KNOWN_PACKS;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeOfConduct {
    pub text: String,
}

impl Encode for CodeOfConduct {
    fn encode(&self, out: &mut Vec<u8>) {
        self.text.encode(out);
    }
}

impl Decode for CodeOfConduct {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            text: String::decode(input)?,
        })
    }
}

impl Packet for CodeOfConduct {
    const ID: i32 = clientbound::CODE_OF_CONDUCT;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ServerboundFinishConfiguration;

impl Encode for ServerboundFinishConfiguration {
    fn encode(&self, _out: &mut Vec<u8>) {}
}

impl Decode for ServerboundFinishConfiguration {
    fn decode(_input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self)
    }
}

impl Packet for ServerboundFinishConfiguration {
    const ID: i32 = serverbound::FINISH_CONFIGURATION;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSelectKnownPacks {
    pub packs: Vec<KnownPack>,
}

impl Encode for ServerboundSelectKnownPacks {
    fn encode(&self, out: &mut Vec<u8>) {
        self.packs.encode(out);
    }
}

impl Decode for ServerboundSelectKnownPacks {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            packs: decode_list(input, MAX_SERVERBOUND_KNOWN_PACKS)?,
        })
    }
}

impl Packet for ServerboundSelectKnownPacks {
    const ID: i32 = serverbound::SELECT_KNOWN_PACKS;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AcceptCodeOfConduct;

impl Encode for AcceptCodeOfConduct {
    fn encode(&self, _out: &mut Vec<u8>) {}
}

impl Decode for AcceptCodeOfConduct {
    fn decode(_input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self)
    }
}

impl Packet for AcceptCodeOfConduct {
    const ID: i32 = serverbound::ACCEPT_CODE_OF_CONDUCT;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerboundConfiguration {
    ClientInformation(ClientInformation),
    CookieResponse(CookieResponse),
    CustomPayload(ServerboundCustomPayload),
    FinishConfiguration(ServerboundFinishConfiguration),
    KeepAlive(KeepAlive),
    Pong(Pong),
    ResourcePack(ResourcePackResponse),
    SelectKnownPacks(ServerboundSelectKnownPacks),
    CustomClickAction(CustomClickAction),
    AcceptCodeOfConduct(AcceptCodeOfConduct),
}

impl ServerboundConfiguration {
    pub fn decode(id: i32, body: &[u8]) -> Result<Self, DecodeError> {
        match id {
            serverbound::CLIENT_INFORMATION => decode_exact(body).map(Self::ClientInformation),
            serverbound::COOKIE_RESPONSE => decode_exact(body).map(Self::CookieResponse),
            serverbound::CUSTOM_PAYLOAD => decode_exact(body).map(Self::CustomPayload),
            serverbound::FINISH_CONFIGURATION => decode_exact(body).map(Self::FinishConfiguration),
            serverbound::KEEP_ALIVE => decode_exact(body).map(Self::KeepAlive),
            serverbound::PONG => decode_exact(body).map(Self::Pong),
            serverbound::RESOURCE_PACK => decode_exact(body).map(Self::ResourcePack),
            serverbound::SELECT_KNOWN_PACKS => decode_exact(body).map(Self::SelectKnownPacks),
            serverbound::CUSTOM_CLICK_ACTION => decode_exact(body).map(Self::CustomClickAction),
            serverbound::ACCEPT_CODE_OF_CONDUCT => {
                decode_exact(body).map(Self::AcceptCodeOfConduct)
            }
            other => Err(DecodeError::UnknownPacket(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use rspigot_nbt::{Compound, Tag};

    use super::*;
    use crate::codec::{Decode, DecodeError, Encode, decode_exact};
    use crate::common::{
        ChatVisibility, ClientInformation, CustomClickAction, Identifier, KeepAlive, MainHand,
        ParticleStatus, Pong, ResourcePackAction, ResourcePackResponse,
    };
    use crate::types::KnownPack;
    use crate::{ids, write_packet};

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

    fn framed(write: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
        let mut out = Vec::new();
        write(&mut out);
        out
    }

    fn id(text: &str) -> Identifier {
        Identifier::parse(text).unwrap()
    }

    fn core_pack() -> KnownPack {
        KnownPack {
            namespace: String::from("minecraft"),
            id: String::from("core"),
            version: String::from("26.3"),
        }
    }

    #[test]
    fn packet_ids_match_generated_ids() {
        assert_eq!(ClientboundFinishConfiguration::ID, 3);
        assert_eq!(ResetChat::ID, 6);
        assert_eq!(RegistryData::ID, 7);
        assert_eq!(UpdateEnabledFeatures::ID, 13);
        assert_eq!(ClientboundSelectKnownPacks::ID, 15);
        assert_eq!(CodeOfConduct::ID, 20);
        assert_eq!(ServerboundFinishConfiguration::ID, 3);
        assert_eq!(ServerboundSelectKnownPacks::ID, 7);
        assert_eq!(AcceptCodeOfConduct::ID, 9);
    }

    #[test]
    fn unit_packets_are_empty() {
        assert!(roundtrip(&ClientboundFinishConfiguration).is_empty());
        assert!(roundtrip(&ResetChat).is_empty());
        assert!(roundtrip(&ServerboundFinishConfiguration).is_empty());
        assert!(roundtrip(&AcceptCodeOfConduct).is_empty());
        let mut framed = Vec::new();
        write_packet(&ClientboundFinishConfiguration, &mut framed);
        assert_eq!(framed, vec![0x03]);
    }

    #[test]
    fn registry_data_layout() {
        let biome = Tag::Compound(Compound::new().with("has_precipitation", 1i8));
        let packet = RegistryData {
            registry: id("minecraft:worldgen/biome"),
            entries: vec![
                RegistryEntry {
                    id: id("minecraft:plains"),
                    data: None,
                },
                RegistryEntry {
                    id: id("rspigot:custom"),
                    data: Some(biome.clone()),
                },
            ],
        };
        let mut expected = string("minecraft:worldgen/biome");
        expected.push(0x02);
        expected.extend(string("minecraft:plains"));
        expected.push(0x00);
        expected.extend(string("rspigot:custom"));
        expected.push(0x01);
        rspigot_nbt::write_network(&mut expected, &biome);
        assert_eq!(roundtrip(&packet), expected);
    }

    #[test]
    fn registry_entry_present_data_must_not_be_end_tag() {
        let mut body = string("minecraft:worldgen/biome");
        body.push(0x01);
        body.extend(string("minecraft:plains"));
        body.extend([0x01, 0x00]);
        assert!(decode_exact::<RegistryData>(&body).is_err());
    }

    #[test]
    fn update_enabled_features_layout() {
        let packet = UpdateEnabledFeatures {
            features: vec![id("minecraft:vanilla")],
        };
        let mut expected = vec![0x01];
        expected.extend(string("minecraft:vanilla"));
        assert_eq!(roundtrip(&packet), expected);
    }

    #[test]
    fn select_known_packs_layout_both_directions() {
        let mut expected = vec![0x01];
        expected.extend(string("minecraft"));
        expected.extend(string("core"));
        expected.extend(string("26.3"));
        assert_eq!(
            roundtrip(&ClientboundSelectKnownPacks {
                packs: vec![core_pack()]
            }),
            expected
        );
        assert_eq!(
            roundtrip(&ServerboundSelectKnownPacks {
                packs: vec![core_pack()]
            }),
            expected
        );
    }

    #[test]
    fn serverbound_known_packs_are_limited_to_64() {
        let mut body = vec![65];
        for _ in 0..65 {
            body.extend(bytes(&core_pack()));
        }
        assert!(decode_exact::<ServerboundSelectKnownPacks>(&body).is_err());
        assert_eq!(
            decode_exact::<ClientboundSelectKnownPacks>(&body)
                .unwrap()
                .packs
                .len(),
            65
        );
    }

    #[test]
    fn code_of_conduct_is_a_string() {
        let packet = CodeOfConduct {
            text: String::from("Be nice"),
        };
        assert_eq!(roundtrip(&packet), string("Be nice"));
    }

    #[test]
    fn every_clientbound_packet_encodes_with_its_id() {
        use crate::common::{
            ClearDialog, ClientboundCustomPayload, CookieRequest, CustomReportDetails, Disconnect,
            Ping, PostEffects, ResourcePackPop, ResourcePackPush, ServerLinks, ShowDialog,
            StoreCookie, Transfer, UpdateTags,
        };
        use crate::types::Text;
        use crate::write_packet_with_id;
        use ids::configuration::clientbound as cb;
        let text = Text(Tag::String(String::from("x")));
        let frames = vec![
            framed(|out| {
                write_packet_with_id(cb::COOKIE_REQUEST, &CookieRequest { key: id("a:b") }, out)
            }),
            framed(|out| {
                write_packet_with_id(
                    cb::CUSTOM_PAYLOAD,
                    &ClientboundCustomPayload {
                        channel: id("minecraft:brand"),
                        data: Vec::new(),
                    },
                    out,
                )
            }),
            framed(|out| {
                write_packet_with_id(
                    cb::DISCONNECT,
                    &Disconnect {
                        reason: text.clone(),
                    },
                    out,
                )
            }),
            framed(|out| write_packet(&ClientboundFinishConfiguration, out)),
            framed(|out| write_packet_with_id(cb::KEEP_ALIVE, &KeepAlive { id: 1 }, out)),
            framed(|out| write_packet_with_id(cb::PING, &Ping { id: 1 }, out)),
            framed(|out| write_packet(&ResetChat, out)),
            framed(|out| {
                write_packet(
                    &RegistryData {
                        registry: id("minecraft:dimension_type"),
                        entries: Vec::new(),
                    },
                    out,
                )
            }),
            framed(|out| {
                write_packet_with_id(cb::RESOURCE_PACK_POP, &ResourcePackPop { id: None }, out)
            }),
            framed(|out| {
                write_packet_with_id(
                    cb::RESOURCE_PACK_PUSH,
                    &ResourcePackPush {
                        id: uuid::Uuid::nil(),
                        url: String::new(),
                        hash: String::new(),
                        required: false,
                        prompt: None,
                    },
                    out,
                )
            }),
            framed(|out| {
                write_packet_with_id(
                    cb::POST_EFFECTS,
                    &PostEffects {
                        effects: Vec::new(),
                    },
                    out,
                )
            }),
            framed(|out| {
                write_packet_with_id(
                    cb::STORE_COOKIE,
                    &StoreCookie {
                        key: id("a:b"),
                        payload: Vec::new(),
                    },
                    out,
                )
            }),
            framed(|out| {
                write_packet_with_id(
                    cb::TRANSFER,
                    &Transfer {
                        host: String::new(),
                        port: 25565,
                    },
                    out,
                )
            }),
            framed(|out| {
                write_packet(
                    &UpdateEnabledFeatures {
                        features: Vec::new(),
                    },
                    out,
                )
            }),
            framed(|out| {
                write_packet_with_id(
                    cb::UPDATE_TAGS,
                    &UpdateTags {
                        registries: Vec::new(),
                    },
                    out,
                )
            }),
            framed(|out| {
                write_packet(
                    &ClientboundSelectKnownPacks {
                        packs: vec![core_pack()],
                    },
                    out,
                )
            }),
            framed(|out| {
                write_packet_with_id(
                    cb::CUSTOM_REPORT_DETAILS,
                    &CustomReportDetails {
                        details: Vec::new(),
                    },
                    out,
                )
            }),
            framed(|out| {
                write_packet_with_id(cb::SERVER_LINKS, &ServerLinks { links: Vec::new() }, out)
            }),
            framed(|out| write_packet_with_id(cb::CLEAR_DIALOG, &ClearDialog, out)),
            framed(|out| {
                write_packet_with_id(
                    cb::SHOW_DIALOG,
                    &ShowDialog {
                        dialog: Tag::Compound(Compound::new()),
                    },
                    out,
                )
            }),
            framed(|out| {
                write_packet(
                    &CodeOfConduct {
                        text: String::new(),
                    },
                    out,
                )
            }),
        ];
        assert_eq!(frames.len(), 21);
        for (index, frame) in frames.iter().enumerate() {
            assert_eq!(crate::split_packet_id(frame).unwrap().0, index as i32);
        }
    }

    #[test]
    fn serverbound_configuration_decodes_every_packet() {
        use ids::configuration::serverbound as sb;
        let mut info = string("en_us");
        info.extend([0x0A, 0x00, 0x01, 0x7F, 0x01, 0x00, 0x01, 0x00]);
        assert_eq!(
            ServerboundConfiguration::decode(sb::CLIENT_INFORMATION, &info).unwrap(),
            ServerboundConfiguration::ClientInformation(ClientInformation {
                language: String::from("en_us"),
                view_distance: 10,
                chat_visibility: ChatVisibility::Full,
                chat_colors: true,
                model_customisation: 0x7F,
                main_hand: MainHand::Right,
                text_filtering_enabled: false,
                allows_listing: true,
                particle_status: ParticleStatus::All,
            })
        );
        let mut cookie = string("rspigot:c");
        cookie.push(0x00);
        assert!(matches!(
            ServerboundConfiguration::decode(sb::COOKIE_RESPONSE, &cookie).unwrap(),
            ServerboundConfiguration::CookieResponse(_)
        ));
        let mut payload = string("minecraft:brand");
        payload.extend(string("vanilla"));
        assert!(matches!(
            ServerboundConfiguration::decode(sb::CUSTOM_PAYLOAD, &payload).unwrap(),
            ServerboundConfiguration::CustomPayload(_)
        ));
        assert_eq!(
            ServerboundConfiguration::decode(sb::FINISH_CONFIGURATION, &[]).unwrap(),
            ServerboundConfiguration::FinishConfiguration(ServerboundFinishConfiguration)
        );
        assert_eq!(
            ServerboundConfiguration::decode(sb::KEEP_ALIVE, &[0, 0, 0, 0, 0, 0, 0, 3]).unwrap(),
            ServerboundConfiguration::KeepAlive(KeepAlive { id: 3 })
        );
        assert_eq!(
            ServerboundConfiguration::decode(sb::PONG, &[0, 0, 0, 4]).unwrap(),
            ServerboundConfiguration::Pong(Pong { id: 4 })
        );
        let pack_id = uuid::Uuid::from_u128(9);
        let mut response = pack_id.as_bytes().to_vec();
        response.push(0x03);
        assert_eq!(
            ServerboundConfiguration::decode(sb::RESOURCE_PACK, &response).unwrap(),
            ServerboundConfiguration::ResourcePack(ResourcePackResponse {
                id: pack_id,
                action: ResourcePackAction::Accepted,
            })
        );
        assert_eq!(
            ServerboundConfiguration::decode(sb::SELECT_KNOWN_PACKS, &[0x00]).unwrap(),
            ServerboundConfiguration::SelectKnownPacks(ServerboundSelectKnownPacks {
                packs: Vec::new()
            })
        );
        let mut click = string("rspigot:click");
        click.extend([0x01, 0x00]);
        assert_eq!(
            ServerboundConfiguration::decode(sb::CUSTOM_CLICK_ACTION, &click).unwrap(),
            ServerboundConfiguration::CustomClickAction(CustomClickAction {
                id: id("rspigot:click"),
                payload: None,
            })
        );
        assert_eq!(
            ServerboundConfiguration::decode(sb::ACCEPT_CODE_OF_CONDUCT, &[]).unwrap(),
            ServerboundConfiguration::AcceptCodeOfConduct(AcceptCodeOfConduct)
        );
        assert_eq!(
            ServerboundConfiguration::decode(10, &[]),
            Err(DecodeError::UnknownPacket(10))
        );
        assert_eq!(
            ServerboundConfiguration::decode(sb::ACCEPT_CODE_OF_CONDUCT, &[0]),
            Err(DecodeError::TrailingBytes(1))
        );
    }
}
