use crate::Packet;
use crate::codec::{
    Decode, DecodeError, Encode, MAX_STRING_CHARS, VarInt, decode_exact, decode_string,
};
use crate::ids;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientIntent {
    Status,
    Login,
    Transfer,
}

impl ClientIntent {
    pub const fn id(self) -> i32 {
        match self {
            Self::Status => 1,
            Self::Login => 2,
            Self::Transfer => 3,
        }
    }

    pub fn from_id(id: i32) -> Result<Self, DecodeError> {
        match id {
            1 => Ok(Self::Status),
            2 => Ok(Self::Login),
            3 => Ok(Self::Transfer),
            other => Err(DecodeError::InvalidValue {
                field: "client intent",
                value: i64::from(other),
            }),
        }
    }
}

impl Encode for ClientIntent {
    fn encode(&self, out: &mut Vec<u8>) {
        VarInt(self.id()).encode(out);
    }
}

impl Decode for ClientIntent {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Self::from_id(VarInt::decode(input)?.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intention {
    pub protocol_version: i32,
    pub host: String,
    pub port: u16,
    pub intent: ClientIntent,
}

impl Encode for Intention {
    fn encode(&self, out: &mut Vec<u8>) {
        VarInt(self.protocol_version).encode(out);
        self.host.encode(out);
        self.port.encode(out);
        self.intent.encode(out);
    }
}

impl Decode for Intention {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            protocol_version: VarInt::decode(input)?.0,
            host: decode_string(input, MAX_STRING_CHARS)?,
            port: u16::decode(input)?,
            intent: ClientIntent::decode(input)?,
        })
    }
}

impl Packet for Intention {
    const ID: i32 = ids::handshake::serverbound::INTENTION;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerboundHandshake {
    Intention(Intention),
}

impl ServerboundHandshake {
    pub fn decode(id: i32, body: &[u8]) -> Result<Self, DecodeError> {
        match id {
            Intention::ID => decode_exact(body).map(Self::Intention),
            other => Err(DecodeError::UnknownPacket(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{DecodeError, Encode};

    fn intention() -> Intention {
        Intention {
            protocol_version: 777,
            host: String::from("localhost"),
            port: 25565,
            intent: ClientIntent::Login,
        }
    }

    #[test]
    fn intention_layout() {
        let mut out = Vec::new();
        intention().encode(&mut out);
        let mut expected = vec![0x89, 0x06, 0x09];
        expected.extend_from_slice(b"localhost");
        expected.extend([0x63, 0xDD, 0x02]);
        assert_eq!(out, expected);
        assert_eq!(
            ServerboundHandshake::decode(Intention::ID, &out).unwrap(),
            ServerboundHandshake::Intention(intention())
        );
    }

    #[test]
    fn intent_ids_match_vanilla() {
        for (intent, id) in [
            (ClientIntent::Status, 1),
            (ClientIntent::Login, 2),
            (ClientIntent::Transfer, 3),
        ] {
            assert_eq!(intent.id(), id);
            assert_eq!(ClientIntent::from_id(id).unwrap(), intent);
        }
        assert!(ClientIntent::from_id(0).is_err());
        assert!(ClientIntent::from_id(4).is_err());
    }

    #[test]
    fn decode_rejects_unknown_ids_and_trailing_bytes() {
        let mut out = Vec::new();
        intention().encode(&mut out);
        assert_eq!(
            ServerboundHandshake::decode(1, &out),
            Err(DecodeError::UnknownPacket(1))
        );
        out.push(0);
        assert_eq!(
            ServerboundHandshake::decode(Intention::ID, &out),
            Err(DecodeError::TrailingBytes(1))
        );
    }
}
