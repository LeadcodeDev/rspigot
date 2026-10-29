use rspigot_core::status::StatusSnapshot;
use rspigot_domain::text::Component;
use serde_json::{Value, json};

use crate::NetworkConfig;
use crate::text::to_json_value;

pub fn status_json(config: &NetworkConfig, snapshot: &StatusSnapshot) -> String {
    let sample: Vec<Value> = if config.hide_online_players {
        Vec::new()
    } else {
        snapshot
            .sample
            .iter()
            .map(|p| json!({ "name": p.name, "id": p.id.0.hyphenated().to_string() }))
            .collect()
    };
    let mut status = json!({
        "version": { "name": config.version_name, "protocol": rspigot_data::PROTOCOL_VERSION },
        "players": { "max": snapshot.max_players, "online": snapshot.online, "sample": sample },
        "description": to_json_value(&Component::text(config.motd.clone())),
        "enforcesSecureChat": config.enforce_secure_profile,
    });
    if let (Some(favicon), Some(object)) = (&config.favicon, status.as_object_mut()) {
        object.insert("favicon".to_owned(), Value::String(favicon.clone()));
    }
    status.to_string()
}

pub fn legacy_status(config: &NetworkConfig, snapshot: &StatusSnapshot) -> Vec<u8> {
    let motd: String = config.motd.chars().filter(|c| *c != '\0').collect();
    let text = format!(
        "§1\0{}\0{}\0{}\0{}\0{}",
        rspigot_data::PROTOCOL_VERSION,
        config.version_name,
        motd,
        snapshot.online,
        snapshot.max_players
    );
    let units: Vec<u16> = text.encode_utf16().collect();
    let mut out = Vec::with_capacity(3 + units.len() * 2);
    out.push(0xFF);
    out.extend_from_slice(&(units.len() as u16).to_be_bytes());
    for unit in units {
        out.extend_from_slice(&unit.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_contains_protocol_and_motd() {
        let config = NetworkConfig::default();
        let snapshot = StatusSnapshot {
            online: 1,
            max_players: 20,
            sample: Vec::new(),
        };
        let parsed: Value = serde_json::from_str(&status_json(&config, &snapshot)).unwrap();
        assert_eq!(parsed["version"]["protocol"], 777);
        assert_eq!(parsed["players"]["online"], 1);
        assert_eq!(parsed["description"]["text"], "A Minecraft Server");
    }

    #[test]
    fn legacy_status_is_utf16_with_kick_prefix() {
        let bytes = legacy_status(&NetworkConfig::default(), &StatusSnapshot::default());
        assert_eq!(bytes[0], 0xFF);
        assert_eq!(&bytes[3..5], &[0x00, 0xA7]);
    }
}
