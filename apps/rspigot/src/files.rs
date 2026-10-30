use std::path::{Path, PathBuf};

use crossbeam_channel::Receiver;
use rspigot_core::admin::{AdminLists, ListEntry};
use uuid::Uuid;

use crate::config::Properties;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::Value;

fn read_list(path: &Path) -> Vec<ListEntry> {
    let Ok(text) = std::fs::read_to_string(path) else {
        let _ = std::fs::write(path, "[]");
        return Vec::new();
    };
    let Ok(Value::Array(entries)) = serde_json::from_str::<Value>(&text) else {
        tracing::warn!("{} is not a JSON array, ignoring it", path.display());
        return Vec::new();
    };
    entries
        .iter()
        .filter_map(|e| {
            let name = e.get("name").and_then(Value::as_str)?;
            let uuid = e
                .get("uuid")
                .and_then(Value::as_str)
                .and_then(|u| Uuid::parse_str(u).ok());
            Some(ListEntry {
                name: name.to_owned(),
                uuid,
            })
        })
        .collect()
}

pub fn read_admin_lists(root: &Path, whitelist_enabled: bool) -> AdminLists {
    AdminLists {
        operators: read_list(&root.join("ops.json")),
        whitelist: read_list(&root.join("whitelist.json")),
        whitelist_enabled,
    }
}

fn write_list(path: &Path, entries: &[ListEntry], operator: bool) -> std::io::Result<()> {
    let values: Vec<Value> = entries
        .iter()
        .map(|e| {
            let mut object = serde_json::Map::new();
            if let Some(uuid) = e.uuid {
                object.insert(
                    "uuid".to_owned(),
                    Value::String(uuid.hyphenated().to_string()),
                );
            }
            object.insert("name".to_owned(), Value::String(e.name.clone()));
            if operator {
                object.insert("level".to_owned(), Value::from(4));
                object.insert("bypassesPlayerLimit".to_owned(), Value::Bool(false));
            }
            Value::Object(object)
        })
        .collect();
    let text =
        serde_json::to_string_pretty(&Value::Array(values)).map_err(std::io::Error::other)?;
    std::fs::write(path, text)
}

pub fn spawn_admin_writer(root: PathBuf, updates: Receiver<AdminLists>) {
    let spawned = std::thread::Builder::new()
        .name("admin-lists".to_owned())
        .spawn(move || {
            let mut whitelist_enabled = None;
            for lists in updates {
                let result =
                    write_list(&root.join("ops.json"), &lists.operators, true).and_then(|()| {
                        write_list(&root.join("whitelist.json"), &lists.whitelist, false)
                    });
                if let Err(error) = result {
                    tracing::error!(%error, "could not save ops.json or whitelist.json");
                }
                if whitelist_enabled != Some(lists.whitelist_enabled) {
                    whitelist_enabled = Some(lists.whitelist_enabled);
                    let path = root.join("server.properties");
                    if let Ok(mut properties) = Properties::load_or_create(&path) {
                        properties.set("white-list", &lists.whitelist_enabled.to_string());
                        if let Err(error) = properties.save(&path) {
                            tracing::error!(%error, "could not save server.properties");
                        }
                    }
                }
            }
        });
    if let Err(error) = spawned {
        tracing::error!(%error, "admin list writer failed to start");
    }
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
