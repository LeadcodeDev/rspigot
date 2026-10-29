pub mod codec;
pub mod common;
pub mod configuration;
pub mod crypto;
pub mod frame;
pub mod handshake;
pub mod ids;
pub mod login;
pub mod play;
pub mod status;
pub mod types;

pub use codec::{Decode, DecodeError, Encode, VarInt, VarLong};

pub trait Packet: Encode {
    const ID: i32;
}

pub fn write_packet<P: Packet>(packet: &P, out: &mut Vec<u8>) {
    write_packet_with_id(P::ID, packet, out);
}

pub fn write_packet_with_id<P: Encode + ?Sized>(id: i32, packet: &P, out: &mut Vec<u8>) {
    VarInt(id).encode(out);
    packet.encode(out);
}

pub fn split_packet_id(packet: &[u8]) -> Result<(i32, &[u8]), DecodeError> {
    let mut input = packet;
    let id = VarInt::decode(&mut input)?.0;
    Ok((id, input))
}
