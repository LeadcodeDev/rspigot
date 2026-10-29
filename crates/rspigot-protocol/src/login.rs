use uuid::Uuid;

use crate::Packet;
use crate::codec::{Decode, DecodeError, Encode, VarInt, decode_exact, decode_string};
use crate::common::{
    CookieResponse, Identifier, MAX_CLIENTBOUND_PAYLOAD_LEN, decode_bytes, decode_remaining,
    encode_bytes,
};
use crate::ids::login::{clientbound, serverbound};
use crate::types::{GameProfile, JsonText};

pub const MAX_SERVER_ID_CHARS: usize = 20;
pub const MAX_PLAYER_NAME_CHARS: usize = 16;
pub const MAX_QUERY_PAYLOAD_LEN: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginDisconnect {
    pub reason: JsonText,
}

impl Encode for LoginDisconnect {
    fn encode(&self, out: &mut Vec<u8>) {
        self.reason.encode(out);
    }
}

impl Decode for LoginDisconnect {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            reason: JsonText::decode(input)?,
        })
    }
}

impl Packet for LoginDisconnect {
    const ID: i32 = clientbound::LOGIN_DISCONNECT;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundHello {
    pub server_id: String,
    pub public_key: Vec<u8>,
    pub challenge: Vec<u8>,
    pub should_authenticate: bool,
}

impl Encode for ClientboundHello {
    fn encode(&self, out: &mut Vec<u8>) {
        self.server_id.encode(out);
        encode_bytes(&self.public_key, out);
        encode_bytes(&self.challenge, out);
        self.should_authenticate.encode(out);
    }
}

impl Decode for ClientboundHello {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            server_id: decode_string(input, MAX_SERVER_ID_CHARS)?,
            public_key: decode_bytes(input, input.len())?,
            challenge: decode_bytes(input, input.len())?,
            should_authenticate: bool::decode(input)?,
        })
    }
}

impl Packet for ClientboundHello {
    const ID: i32 = clientbound::HELLO;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginFinished {
    pub profile: GameProfile,
    pub session_id: Uuid,
}

impl Encode for LoginFinished {
    fn encode(&self, out: &mut Vec<u8>) {
        self.profile.encode(out);
        self.session_id.encode(out);
    }
}

impl Decode for LoginFinished {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            profile: GameProfile::decode(input)?,
            session_id: Uuid::decode(input)?,
        })
    }
}

impl Packet for LoginFinished {
    const ID: i32 = clientbound::LOGIN_FINISHED;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoginCompression {
    pub threshold: i32,
}

impl Encode for LoginCompression {
    fn encode(&self, out: &mut Vec<u8>) {
        VarInt(self.threshold).encode(out);
    }
}

impl Decode for LoginCompression {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            threshold: VarInt::decode(input)?.0,
        })
    }
}

impl Packet for LoginCompression {
    const ID: i32 = clientbound::LOGIN_COMPRESSION;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomQuery {
    pub transaction_id: i32,
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl Encode for CustomQuery {
    fn encode(&self, out: &mut Vec<u8>) {
        VarInt(self.transaction_id).encode(out);
        self.channel.encode(out);
        out.extend_from_slice(&self.data);
    }
}

impl Decode for CustomQuery {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            transaction_id: VarInt::decode(input)?.0,
            channel: Identifier::decode(input)?,
            data: decode_remaining(input, MAX_CLIENTBOUND_PAYLOAD_LEN)?,
        })
    }
}

impl Packet for CustomQuery {
    const ID: i32 = clientbound::CUSTOM_QUERY;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundHello {
    pub name: String,
    pub profile_id: Uuid,
}

impl Encode for ServerboundHello {
    fn encode(&self, out: &mut Vec<u8>) {
        self.name.encode(out);
        self.profile_id.encode(out);
    }
}

impl Decode for ServerboundHello {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            name: decode_string(input, MAX_PLAYER_NAME_CHARS)?,
            profile_id: Uuid::decode(input)?,
        })
    }
}

impl Packet for ServerboundHello {
    const ID: i32 = serverbound::HELLO;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    pub encrypted_secret: Vec<u8>,
    pub encrypted_challenge: Vec<u8>,
}

impl Encode for Key {
    fn encode(&self, out: &mut Vec<u8>) {
        encode_bytes(&self.encrypted_secret, out);
        encode_bytes(&self.encrypted_challenge, out);
    }
}

impl Decode for Key {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            encrypted_secret: decode_bytes(input, input.len())?,
            encrypted_challenge: decode_bytes(input, input.len())?,
        })
    }
}

impl Packet for Key {
    const ID: i32 = serverbound::KEY;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomQueryAnswer {
    pub transaction_id: i32,
    pub payload: Option<Vec<u8>>,
}

impl Encode for CustomQueryAnswer {
    fn encode(&self, out: &mut Vec<u8>) {
        VarInt(self.transaction_id).encode(out);
        match &self.payload {
            Some(payload) => {
                true.encode(out);
                out.extend_from_slice(payload);
            }
            None => false.encode(out),
        }
    }
}

impl Decode for CustomQueryAnswer {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let transaction_id = VarInt::decode(input)?.0;
        if input.len() > MAX_QUERY_PAYLOAD_LEN {
            return Err(DecodeError::TooLong {
                len: input.len(),
                max: MAX_QUERY_PAYLOAD_LEN,
            });
        }
        let payload = if bool::decode(input)? {
            Some(std::mem::take(input).to_vec())
        } else {
            None
        };
        Ok(Self {
            transaction_id,
            payload,
        })
    }
}

impl Packet for CustomQueryAnswer {
    const ID: i32 = serverbound::CUSTOM_QUERY_ANSWER;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LoginAcknowledged;

impl Encode for LoginAcknowledged {
    fn encode(&self, _out: &mut Vec<u8>) {}
}

impl Decode for LoginAcknowledged {
    fn decode(_input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self)
    }
}

impl Packet for LoginAcknowledged {
    const ID: i32 = serverbound::LOGIN_ACKNOWLEDGED;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerboundLogin {
    Hello(ServerboundHello),
    Key(Key),
    CustomQueryAnswer(CustomQueryAnswer),
    LoginAcknowledged(LoginAcknowledged),
    CookieResponse(CookieResponse),
}

impl ServerboundLogin {
    pub fn decode(id: i32, body: &[u8]) -> Result<Self, DecodeError> {
        match id {
            serverbound::HELLO => decode_exact(body).map(Self::Hello),
            serverbound::KEY => decode_exact(body).map(Self::Key),
            serverbound::CUSTOM_QUERY_ANSWER => decode_exact(body).map(Self::CustomQueryAnswer),
            serverbound::LOGIN_ACKNOWLEDGED => decode_exact(body).map(Self::LoginAcknowledged),
            serverbound::COOKIE_RESPONSE => decode_exact(body).map(Self::CookieResponse),
            other => Err(DecodeError::UnknownPacket(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::codec::{Decode, DecodeError, Encode, decode_exact};
    use crate::common::{CookieResponse, Identifier};
    use crate::types::{GameProfile, JsonText, ProfileProperty};

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

    fn uuid(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    #[test]
    fn packet_ids_match_generated_ids() {
        assert_eq!(LoginDisconnect::ID, 0);
        assert_eq!(ClientboundHello::ID, 1);
        assert_eq!(LoginFinished::ID, 2);
        assert_eq!(LoginCompression::ID, 3);
        assert_eq!(CustomQuery::ID, 4);
        assert_eq!(ServerboundHello::ID, 0);
        assert_eq!(Key::ID, 1);
        assert_eq!(CustomQueryAnswer::ID, 2);
        assert_eq!(LoginAcknowledged::ID, 3);
    }

    #[test]
    fn login_disconnect_is_json() {
        let packet = LoginDisconnect {
            reason: JsonText(String::from("{\"text\":\"x\"}")),
        };
        assert_eq!(roundtrip(&packet), string("{\"text\":\"x\"}"));
    }

    #[test]
    fn clientbound_hello_layout() {
        let packet = ClientboundHello {
            server_id: String::new(),
            public_key: vec![0x30, 0x81],
            challenge: vec![9, 9, 9, 9],
            should_authenticate: true,
        };
        assert_eq!(
            roundtrip(&packet),
            vec![0x00, 0x02, 0x30, 0x81, 0x04, 9, 9, 9, 9, 0x01]
        );
        let mut long_id = string(&"a".repeat(21));
        long_id.extend([0x00, 0x00, 0x00]);
        assert!(decode_exact::<ClientboundHello>(&long_id).is_err());
    }

    #[test]
    fn login_finished_carries_profile_then_session_id() {
        let packet = LoginFinished {
            profile: GameProfile {
                id: uuid(1),
                name: String::from("Notch"),
                properties: vec![ProfileProperty {
                    name: String::from("textures"),
                    value: String::from("v"),
                    signature: Some(String::from("s")),
                }],
            },
            session_id: uuid(2),
        };
        let mut expected = uuid(1).as_bytes().to_vec();
        expected.extend(string("Notch"));
        expected.push(0x01);
        expected.extend(string("textures"));
        expected.extend(string("v"));
        expected.push(0x01);
        expected.extend(string("s"));
        expected.extend_from_slice(uuid(2).as_bytes());
        assert_eq!(roundtrip(&packet), expected);
    }

    #[test]
    fn login_compression_is_varint() {
        assert_eq!(
            roundtrip(&LoginCompression { threshold: 256 }),
            vec![0x80, 0x02]
        );
        assert_eq!(
            roundtrip(&LoginCompression { threshold: -1 }),
            vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]
        );
    }

    #[test]
    fn custom_query_layout() {
        let packet = CustomQuery {
            transaction_id: 300,
            channel: Identifier::parse("velocity:player_info").unwrap(),
            data: vec![0x01, 0x02],
        };
        let mut expected = vec![0xAC, 0x02];
        expected.extend(string("velocity:player_info"));
        expected.extend([0x01, 0x02]);
        assert_eq!(roundtrip(&packet), expected);
    }

    #[test]
    fn serverbound_hello_layout() {
        let packet = ServerboundHello {
            name: String::from("Notch"),
            profile_id: uuid(7),
        };
        let mut expected = string("Notch");
        expected.extend_from_slice(uuid(7).as_bytes());
        assert_eq!(roundtrip(&packet), expected);
        let mut long_name = string(&"a".repeat(17));
        long_name.extend_from_slice(uuid(7).as_bytes());
        assert!(decode_exact::<ServerboundHello>(&long_name).is_err());
    }

    #[test]
    fn key_layout() {
        let packet = Key {
            encrypted_secret: vec![1, 2, 3],
            encrypted_challenge: vec![4, 5],
        };
        assert_eq!(roundtrip(&packet), vec![0x03, 1, 2, 3, 0x02, 4, 5]);
    }

    #[test]
    fn custom_query_answer_layout() {
        assert_eq!(
            roundtrip(&CustomQueryAnswer {
                transaction_id: 5,
                payload: None
            }),
            vec![0x05, 0x00]
        );
        assert_eq!(
            roundtrip(&CustomQueryAnswer {
                transaction_id: 5,
                payload: Some(vec![0xAA, 0xBB])
            }),
            vec![0x05, 0x01, 0xAA, 0xBB]
        );
        let mut huge = vec![0x05, 0x01];
        huge.extend(vec![0u8; 1_048_576]);
        assert!(decode_exact::<CustomQueryAnswer>(&huge).is_err());
    }

    #[test]
    fn login_acknowledged_is_empty() {
        assert!(roundtrip(&LoginAcknowledged).is_empty());
    }

    #[test]
    fn serverbound_login_decodes_every_packet() {
        let mut hello = string("Notch");
        hello.extend_from_slice(uuid(7).as_bytes());
        assert_eq!(
            ServerboundLogin::decode(0, &hello).unwrap(),
            ServerboundLogin::Hello(ServerboundHello {
                name: String::from("Notch"),
                profile_id: uuid(7),
            })
        );
        assert!(matches!(
            ServerboundLogin::decode(1, &[0x01, 0xAA, 0x01, 0xBB]).unwrap(),
            ServerboundLogin::Key(_)
        ));
        assert!(matches!(
            ServerboundLogin::decode(2, &[0x01, 0x00]).unwrap(),
            ServerboundLogin::CustomQueryAnswer(_)
        ));
        assert_eq!(
            ServerboundLogin::decode(3, &[]).unwrap(),
            ServerboundLogin::LoginAcknowledged(LoginAcknowledged)
        );
        let mut cookie = string("rspigot:c");
        cookie.push(0x00);
        assert_eq!(
            ServerboundLogin::decode(4, &cookie).unwrap(),
            ServerboundLogin::CookieResponse(CookieResponse {
                key: Identifier::parse("rspigot:c").unwrap(),
                payload: None,
            })
        );
        assert_eq!(
            ServerboundLogin::decode(5, &[]),
            Err(DecodeError::UnknownPacket(5))
        );
        assert_eq!(
            ServerboundLogin::decode(3, &[0]),
            Err(DecodeError::TrailingBytes(1))
        );
    }
}
