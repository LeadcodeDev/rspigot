use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_BODY_BYTES: u64 = 1 << 20;

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

pub trait HttpClient: Send + Sync + 'static {
    fn get(&self, url: &str) -> Result<HttpResponse, String>;
}

pub struct UreqClient {
    agent: ureq::Agent,
}

impl UreqClient {
    pub fn new() -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(REQUEST_TIMEOUT))
            .http_status_as_error(false)
            .build();
        Self {
            agent: ureq::Agent::new_with_config(config),
        }
    }
}

impl Default for UreqClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient for UreqClient {
    fn get(&self, url: &str) -> Result<HttpResponse, String> {
        let mut response = self.agent.get(url).call().map_err(|e| e.to_string())?;
        let status = response.status().as_u16();
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_BODY_BYTES)
            .read_to_string()
            .map_err(|e| e.to_string())?;
        Ok(HttpResponse { status, body })
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread::JoinHandle;

    use rspigot_core::ports::{AuthError, SessionVerifier};

    use super::*;
    use crate::MojangSessionVerifier;

    fn serve_once(reply: String) -> (String, JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut chunk = [0u8; 512];
            while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                let read = stream.read(&mut chunk).unwrap();
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..read]);
            }
            stream.write_all(reply.as_bytes()).unwrap();
            let text = String::from_utf8(request).unwrap();
            text.lines().next().unwrap().to_owned()
        });
        (base, handle)
    }

    fn reply(status_line: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    #[test]
    fn returns_status_and_body() {
        let (base, handle) = serve_once(reply("200 OK", r#"{"a":1}"#));
        let response = UreqClient::new().get(&format!("{base}/x?y=z")).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, r#"{"a":1}"#);
        assert_eq!(handle.join().unwrap(), "GET /x?y=z HTTP/1.1");
    }

    #[test]
    fn no_content_is_a_response_not_an_error() {
        let (base, _handle) = serve_once(reply("204 No Content", ""));
        let response = UreqClient::new().get(&base).unwrap();
        assert_eq!(response.status, 204);
        assert!(response.body.is_empty());
    }

    #[test]
    fn server_error_is_a_response_not_an_error() {
        let (base, _handle) = serve_once(reply("500 Internal Server Error", "boom"));
        let response = UreqClient::new().get(&base).unwrap();
        assert_eq!(response.status, 500);
        assert_eq!(response.body, "boom");
    }

    #[test]
    fn refused_connection_is_an_error() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        assert!(UreqClient::new().get(&base).is_err());
    }

    #[test]
    fn verifier_round_trip_over_http() {
        let body = r#"{"id":"069a79f444e94726a5befca90e38aaf5","name":"jeb_","properties":[]}"#;
        let (base, handle) = serve_once(reply("200 OK", body));
        let verifier = MojangSessionVerifier::with_client(&base, true, UreqClient::new());
        let profile = verifier
            .verify("jeb_", "-1a2b", Some("203.0.113.7".parse().unwrap()))
            .unwrap();
        assert_eq!(profile.name, "jeb_");
        assert_eq!(
            handle.join().unwrap(),
            "GET /session/minecraft/hasJoined?username=jeb_&serverId=-1a2b&ip=203.0.113.7 HTTP/1.1"
        );
    }

    #[test]
    fn verifier_maps_refused_connection_to_unreachable() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        drop(listener);
        let verifier = MojangSessionVerifier::with_client(&base, false, UreqClient::new());
        let error = verifier.verify("jeb_", "abc", None).unwrap_err();
        assert!(matches!(error, AuthError::Unreachable(_)));
    }
}
