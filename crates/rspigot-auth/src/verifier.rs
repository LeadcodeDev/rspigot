use std::net::IpAddr;

use rspigot_core::ports::{AuthError, SessionVerifier};
use rspigot_domain::player::{GameProfile, PlayerId, ProfileProperty};
use serde::Deserialize;
use uuid::Uuid;

use crate::http::{HttpClient, UreqClient};

const DEFAULT_BASE_URL: &str = "https://sessionserver.mojang.com";
const HAS_JOINED_PATH: &str = "/session/minecraft/hasJoined";
const HEX: &[u8; 16] = b"0123456789ABCDEF";

pub struct MojangSessionVerifier<C: HttpClient = UreqClient> {
    base_url: String,
    prevent_proxy_connections: bool,
    client: C,
}

impl MojangSessionVerifier<UreqClient> {
    pub fn new(prevent_proxy_connections: bool) -> Self {
        Self::with_client(
            DEFAULT_BASE_URL,
            prevent_proxy_connections,
            UreqClient::new(),
        )
    }
}

impl<C: HttpClient> MojangSessionVerifier<C> {
    pub fn with_client(base_url: &str, prevent_proxy_connections: bool, client: C) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            prevent_proxy_connections,
            client,
        }
    }

    fn has_joined_url(
        &self,
        username: &str,
        server_hash: &str,
        client_ip: Option<IpAddr>,
    ) -> String {
        let mut url = String::with_capacity(self.base_url.len() + 128);
        url.push_str(&self.base_url);
        url.push_str(HAS_JOINED_PATH);
        url.push_str("?username=");
        push_encoded(&mut url, username);
        url.push_str("&serverId=");
        push_encoded(&mut url, server_hash);
        if let Some(ip) = client_ip.filter(|_| self.prevent_proxy_connections) {
            url.push_str("&ip=");
            push_encoded(&mut url, &ip.to_string());
        }
        url
    }
}

impl<C: HttpClient> SessionVerifier for MojangSessionVerifier<C> {
    fn verify(
        &self,
        username: &str,
        server_hash: &str,
        client_ip: Option<IpAddr>,
    ) -> Result<GameProfile, AuthError> {
        let url = self.has_joined_url(username, server_hash, client_ip);
        let response = self.client.get(&url).map_err(AuthError::Unreachable)?;
        match response.status {
            200 if response.body.trim().is_empty() => {
                Err(AuthError::NotAuthenticated(username.to_owned()))
            }
            200 => parse_profile(&response.body),
            204 => Err(AuthError::NotAuthenticated(username.to_owned())),
            429 | 500..=599 => Err(AuthError::Unreachable(format!(
                "status {}",
                response.status
            ))),
            status => Err(AuthError::InvalidResponse(format!(
                "unexpected status {status}"
            ))),
        }
    }
}

#[derive(Deserialize)]
struct ProfileResponse {
    id: String,
    name: String,
    #[serde(default)]
    properties: Vec<PropertyResponse>,
}

#[derive(Deserialize)]
struct PropertyResponse {
    name: String,
    value: String,
    signature: Option<String>,
}

fn parse_profile(body: &str) -> Result<GameProfile, AuthError> {
    let response: ProfileResponse =
        serde_json::from_str(body).map_err(|e| AuthError::InvalidResponse(e.to_string()))?;
    let id = parse_id(&response.id)?;
    let properties = response
        .properties
        .into_iter()
        .map(|p| ProfileProperty {
            name: p.name,
            value: p.value,
            signature: p.signature,
        })
        .collect();
    Ok(GameProfile {
        id: PlayerId(id),
        name: response.name,
        properties,
    })
}

fn parse_id(id: &str) -> Result<Uuid, AuthError> {
    if !matches!(id.len(), 32 | 36) {
        return Err(AuthError::InvalidResponse(format!(
            "invalid profile id {id:?}"
        )));
    }
    Uuid::parse_str(id).map_err(|e| AuthError::InvalidResponse(format!("invalid profile id: {e}")))
}

fn push_encoded(out: &mut String, value: &str) {
    for &byte in value.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push('%');
            out.push(char::from(HEX[usize::from(byte >> 4)]));
            out.push(char::from(HEX[usize::from(byte & 0x0F)]));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use std::sync::{Arc, Mutex};

    use rspigot_core::ports::{AuthError, SessionVerifier};
    use rspigot_domain::player::{GameProfile, PlayerId, ProfileProperty};
    use uuid::Uuid;

    use super::*;
    use crate::http::HttpResponse;

    const BASE: &str = "https://sessionserver.mojang.com";
    const UNDASHED: &str = "069a79f444e94726a5befca90e38aaf5";
    const DASHED: &str = "069a79f4-44e9-4726-a5be-fca90e38aaf5";

    struct FakeClient {
        urls: Arc<Mutex<Vec<String>>>,
        reply: Result<(u16, String), String>,
    }

    impl HttpClient for FakeClient {
        fn get(&self, url: &str) -> Result<HttpResponse, String> {
            self.urls.lock().unwrap().push(url.to_owned());
            match &self.reply {
                Ok((status, body)) => Ok(HttpResponse {
                    status: *status,
                    body: body.clone(),
                }),
                Err(error) => Err(error.clone()),
            }
        }
    }

    fn verifier(
        prevent_proxy: bool,
        reply: Result<(u16, &str), &str>,
    ) -> (MojangSessionVerifier<FakeClient>, Arc<Mutex<Vec<String>>>) {
        let urls = Arc::new(Mutex::new(Vec::new()));
        let client = FakeClient {
            urls: Arc::clone(&urls),
            reply: reply
                .map(|(status, body)| (status, body.to_owned()))
                .map_err(str::to_owned),
        };
        (
            MojangSessionVerifier::with_client(BASE, prevent_proxy, client),
            urls,
        )
    }

    fn payload(id: &str) -> String {
        format!(
            r#"{{"id":"{id}","name":"jeb_","properties":[{{"name":"textures","value":"eyJ0aW1lc3RhbXAiOjF9","signature":"c2lnbmF0dXJl"}},{{"name":"unsigned","value":"dmFsdWU"}}]}}"#
        )
    }

    fn expected_profile() -> GameProfile {
        GameProfile {
            id: PlayerId(Uuid::parse_str(DASHED).unwrap()),
            name: "jeb_".to_owned(),
            properties: vec![
                ProfileProperty {
                    name: "textures".to_owned(),
                    value: "eyJ0aW1lc3RhbXAiOjF9".to_owned(),
                    signature: Some("c2lnbmF0dXJl".to_owned()),
                },
                ProfileProperty {
                    name: "unsigned".to_owned(),
                    value: "dmFsdWU".to_owned(),
                    signature: None,
                },
            ],
        }
    }

    fn ip(text: &str) -> Option<IpAddr> {
        Some(text.parse().unwrap())
    }

    #[test]
    fn ok_with_undashed_id_returns_profile() {
        let (verifier, _) = verifier(false, Ok((200, &payload(UNDASHED))));
        let profile = verifier.verify("jeb_", "abc", None).unwrap();
        assert_eq!(profile, expected_profile());
    }

    #[test]
    fn ok_with_dashed_id_returns_profile() {
        let (verifier, _) = verifier(false, Ok((200, &payload(DASHED))));
        let profile = verifier.verify("jeb_", "abc", None).unwrap();
        assert_eq!(profile, expected_profile());
    }

    #[test]
    fn missing_properties_default_to_empty() {
        let body = format!(r#"{{"id":"{UNDASHED}","name":"jeb_"}}"#);
        let (verifier, _) = verifier(false, Ok((200, &body)));
        let profile = verifier.verify("jeb_", "abc", None).unwrap();
        assert!(profile.properties.is_empty());
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let body = format!(r#"{{"id":"{UNDASHED}","name":"jeb_","properties":[],"extra":1}}"#);
        let (verifier, _) = verifier(false, Ok((200, &body)));
        assert!(verifier.verify("jeb_", "abc", None).is_ok());
    }

    #[test]
    fn status_204_is_not_authenticated() {
        let (verifier, _) = verifier(false, Ok((204, "")));
        let error = verifier.verify("jeb_", "abc", None).unwrap_err();
        assert!(matches!(error, AuthError::NotAuthenticated(name) if name == "jeb_"));
    }

    #[test]
    fn status_200_with_empty_body_is_not_authenticated() {
        let (verifier, _) = verifier(false, Ok((200, "  \n")));
        let error = verifier.verify("jeb_", "abc", None).unwrap_err();
        assert!(matches!(error, AuthError::NotAuthenticated(name) if name == "jeb_"));
    }

    #[test]
    fn status_500_is_unreachable() {
        let (verifier, _) = verifier(false, Ok((500, "oops")));
        let error = verifier.verify("jeb_", "abc", None).unwrap_err();
        assert!(matches!(error, AuthError::Unreachable(_)));
    }

    #[test]
    fn status_503_is_unreachable() {
        let (verifier, _) = verifier(false, Ok((503, "")));
        let error = verifier.verify("jeb_", "abc", None).unwrap_err();
        assert!(matches!(error, AuthError::Unreachable(_)));
    }

    #[test]
    fn status_429_is_unreachable() {
        let (verifier, _) = verifier(false, Ok((429, "")));
        let error = verifier.verify("jeb_", "abc", None).unwrap_err();
        assert!(matches!(error, AuthError::Unreachable(_)));
    }

    #[test]
    fn other_statuses_are_invalid_responses() {
        let (verifier, _) = verifier(false, Ok((400, "bad")));
        let error = verifier.verify("jeb_", "abc", None).unwrap_err();
        assert!(matches!(error, AuthError::InvalidResponse(_)));
    }

    #[test]
    fn transport_error_is_unreachable_and_keeps_message() {
        let (verifier, _) = verifier(false, Err("timed out"));
        let error = verifier.verify("jeb_", "abc", None).unwrap_err();
        assert!(matches!(error, AuthError::Unreachable(message) if message.contains("timed out")));
    }

    #[test]
    fn garbage_json_is_invalid_response() {
        let (verifier, _) = verifier(false, Ok((200, "{not json")));
        let error = verifier.verify("jeb_", "abc", None).unwrap_err();
        assert!(matches!(error, AuthError::InvalidResponse(_)));
    }

    #[test]
    fn missing_name_is_invalid_response() {
        let body = format!(r#"{{"id":"{UNDASHED}"}}"#);
        let (verifier, _) = verifier(false, Ok((200, &body)));
        let error = verifier.verify("jeb_", "abc", None).unwrap_err();
        assert!(matches!(error, AuthError::InvalidResponse(_)));
    }

    #[test]
    fn malformed_ids_are_invalid_responses() {
        let ids = [
            "",
            "xyz",
            "069a79f444e94726a5befca90e38aaf",
            "069a79f444e94726a5befca90e38aafg",
            "{069a79f4-44e9-4726-a5be-fca90e38aaf5}",
        ];
        for id in ids {
            let (verifier, _) = verifier(false, Ok((200, &payload(id))));
            let error = verifier.verify("jeb_", "abc", None).unwrap_err();
            assert!(matches!(error, AuthError::InvalidResponse(_)));
        }
    }

    #[test]
    fn url_without_ip() {
        let (verifier, urls) = verifier(false, Ok((204, "")));
        let _ = verifier.verify("jeb_", "abc123", None);
        assert_eq!(
            urls.lock().unwrap().as_slice(),
            [
                "https://sessionserver.mojang.com/session/minecraft/hasJoined?username=jeb_&serverId=abc123"
            ]
        );
    }

    #[test]
    fn url_with_ip_when_prevent_proxy_connections() {
        let (verifier, urls) = verifier(true, Ok((204, "")));
        let _ = verifier.verify("jeb_", "abc123", ip("203.0.113.7"));
        assert_eq!(
            urls.lock().unwrap().as_slice(),
            [
                "https://sessionserver.mojang.com/session/minecraft/hasJoined?username=jeb_&serverId=abc123&ip=203.0.113.7"
            ]
        );
    }

    #[test]
    fn url_omits_ip_when_not_configured() {
        let (verifier, urls) = verifier(false, Ok((204, "")));
        let _ = verifier.verify("jeb_", "abc123", ip("203.0.113.7"));
        assert!(!urls.lock().unwrap()[0].contains("ip="));
    }

    #[test]
    fn url_omits_ip_when_unknown() {
        let (verifier, urls) = verifier(true, Ok((204, "")));
        let _ = verifier.verify("jeb_", "abc123", None);
        assert!(!urls.lock().unwrap()[0].contains("ip="));
    }

    #[test]
    fn url_encodes_ipv6() {
        let (verifier, urls) = verifier(true, Ok((204, "")));
        let _ = verifier.verify("jeb_", "abc123", ip("2001:db8::1"));
        assert!(urls.lock().unwrap()[0].ends_with("&ip=2001%3Adb8%3A%3A1"));
    }

    #[test]
    fn url_encodes_username_and_hash() {
        let (verifier, urls) = verifier(false, Ok((204, "")));
        let _ = verifier.verify("a b&c=é/?#", "-4a5b+c d", None);
        assert_eq!(
            urls.lock().unwrap().as_slice(),
            [
                "https://sessionserver.mojang.com/session/minecraft/hasJoined?username=a%20b%26c%3D%C3%A9%2F%3F%23&serverId=-4a5b%2Bc%20d"
            ]
        );
    }

    #[test]
    fn base_url_trailing_slash_is_trimmed() {
        let urls = Arc::new(Mutex::new(Vec::new()));
        let client = FakeClient {
            urls: Arc::clone(&urls),
            reply: Ok((204, String::new())),
        };
        let verifier = MojangSessionVerifier::with_client("http://localhost:8080/", false, client);
        let _ = verifier.verify("jeb_", "abc", None);
        assert_eq!(
            urls.lock().unwrap().as_slice(),
            ["http://localhost:8080/session/minecraft/hasJoined?username=jeb_&serverId=abc"]
        );
    }
}
