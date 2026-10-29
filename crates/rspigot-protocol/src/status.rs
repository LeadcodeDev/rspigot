use crate::Packet;
use crate::codec::{Decode, DecodeError, Encode, decode_exact, decode_string};
use crate::ids::status::{clientbound, serverbound};

pub const MAX_STATUS_JSON_CHARS: usize = 32_767;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatusRequest;

impl Encode for StatusRequest {
    fn encode(&self, _out: &mut Vec<u8>) {}
}

impl Decode for StatusRequest {
    fn decode(_input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self)
    }
}

impl Packet for StatusRequest {
    const ID: i32 = serverbound::STATUS_REQUEST;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusResponse {
    pub json: String,
}

impl Encode for StatusResponse {
    fn encode(&self, out: &mut Vec<u8>) {
        self.json.encode(out);
    }
}

impl Decode for StatusResponse {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            json: decode_string(input, MAX_STATUS_JSON_CHARS)?,
        })
    }
}

impl Packet for StatusResponse {
    const ID: i32 = clientbound::STATUS_RESPONSE;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PingRequest {
    pub time: i64,
}

impl Encode for PingRequest {
    fn encode(&self, out: &mut Vec<u8>) {
        self.time.encode(out);
    }
}

impl Decode for PingRequest {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            time: i64::decode(input)?,
        })
    }
}

impl Packet for PingRequest {
    const ID: i32 = serverbound::PING_REQUEST;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PongResponse {
    pub time: i64,
}

impl Encode for PongResponse {
    fn encode(&self, out: &mut Vec<u8>) {
        self.time.encode(out);
    }
}

impl Decode for PongResponse {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            time: i64::decode(input)?,
        })
    }
}

impl Packet for PongResponse {
    const ID: i32 = clientbound::PONG_RESPONSE;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundStatus {
    StatusRequest(StatusRequest),
    PingRequest(PingRequest),
}

impl ServerboundStatus {
    pub fn decode(id: i32, body: &[u8]) -> Result<Self, DecodeError> {
        match id {
            StatusRequest::ID => decode_exact(body).map(Self::StatusRequest),
            PingRequest::ID => decode_exact(body).map(Self::PingRequest),
            other => Err(DecodeError::UnknownPacket(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{DecodeError, Encode, decode_exact};

    fn bytes<T: Encode>(value: &T) -> Vec<u8> {
        let mut out = Vec::new();
        value.encode(&mut out);
        out
    }

    #[test]
    fn status_packets_layout() {
        assert!(bytes(&StatusRequest).is_empty());
        let response = StatusResponse {
            json: String::from("{}"),
        };
        assert_eq!(bytes(&response), vec![0x02, b'{', b'}']);
        assert_eq!(
            decode_exact::<StatusResponse>(&bytes(&response)).unwrap(),
            response
        );
        let ping = PingRequest {
            time: 0x0102_0304_0506_0708,
        };
        assert_eq!(bytes(&ping), vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let pong = PongResponse { time: -1 };
        assert_eq!(bytes(&pong), vec![0xFF; 8]);
        assert_eq!(decode_exact::<PongResponse>(&bytes(&pong)).unwrap(), pong);
    }

    #[test]
    fn packet_ids_match_generated_ids() {
        assert_eq!(StatusResponse::ID, 0);
        assert_eq!(PongResponse::ID, 1);
    }

    #[test]
    fn serverbound_status_decodes_by_id() {
        assert_eq!(
            ServerboundStatus::decode(0, &[]).unwrap(),
            ServerboundStatus::StatusRequest(StatusRequest)
        );
        assert_eq!(
            ServerboundStatus::decode(1, &[0, 0, 0, 0, 0, 0, 0, 9]).unwrap(),
            ServerboundStatus::PingRequest(PingRequest { time: 9 })
        );
        assert_eq!(
            ServerboundStatus::decode(2, &[]),
            Err(DecodeError::UnknownPacket(2))
        );
        assert_eq!(
            ServerboundStatus::decode(0, &[0]),
            Err(DecodeError::TrailingBytes(1))
        );
    }

    #[test]
    fn status_response_is_limited_to_32767_chars() {
        let mut body = Vec::new();
        "a".repeat(32768).encode(&mut body);
        assert!(decode_exact::<StatusResponse>(&body).is_err());
    }
}
