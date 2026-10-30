use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::Value;

pub fn read_operators(root: &Path) -> Vec<String> {
    let path = root.join("ops.json");
    let Ok(text) = std::fs::read_to_string(&path) else {
        let _ = std::fs::write(&path, "[]");
        return Vec::new();
    };
    let Ok(Value::Array(entries)) = serde_json::from_str::<Value>(&text) else {
        tracing::warn!("{} is not a JSON array, ignoring it", path.display());
        return Vec::new();
    };
    entries
        .iter()
        .flat_map(|e| [e.get("name"), e.get("uuid")])
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

pub fn read_favicon(root: &Path) -> Option<String> {
    let bytes = std::fs::read(root.join("server-icon.png")).ok()?;
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        tracing::warn!("server-icon.png is not a PNG image");
        return None;
    }
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if width != 64 || height != 64 {
        tracing::warn!("server-icon.png must be 64x64 pixels, got {width}x{height}");
        return None;
    }
    Some(format!("data:image/png;base64,{}", STANDARD.encode(&bytes)))
}
