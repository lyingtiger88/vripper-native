//! Centralized outbound HTTP transport configuration.
//!
//! All engine requests must obtain their reqwest client from this module. Proxy
//! secrets are ephemeral inputs from the platform secure-storage adapter and are
//! intentionally excluded from NetworkConfig serialization/persistence.

use reqwest::{Client, NoProxy, Proxy};
use std::fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyMode {
    Disabled,
    System,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyProtocol {
    Http,
    Https,
    Socks5,
}

impl ProxyProtocol {
    fn scheme(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::Socks5 => "socks5h",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyEndpoint {
    pub protocol: ProxyProtocol,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    pub mode: ProxyMode,
    pub manual_proxy: Option<ProxyEndpoint>,
    pub bypass_hosts: Vec<String>,
}

impl Default for ProxyMode {
    fn default() -> Self { Self::Disabled }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            mode: ProxyMode::Disabled,
            manual_proxy: None,
            bypass_hosts: Vec::new(),
        }
    }
}

/// Credentials are loaded just-in-time from the OS credential store.
/// This type deliberately does not implement Serialize.
#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct ProxyCredentials {
    pub username: String,
    pub password: String,
}

impl fmt::Debug for ProxyCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProxyCredentials")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProxyError {
    #[error("system proxy configuration is unavailable on this platform")]
    SystemProxyUnavailable,
    #[error("manual proxy endpoint is not configured")]
    ManualProxyMissing,
    #[error("proxy hostname is empty or invalid")]
    InvalidHost,
    #[error("proxy port must be between 1 and 65535")]
    InvalidPort,
    #[error("proxy username and password must be supplied together")]
    IncompleteCredentials,
    #[error("proxy URL is invalid: {0}")]
    InvalidProxyUrl(String),
    #[error("failed to build HTTP client: {0}")]
    ClientBuild(String),
}

impl ProxyEndpoint {
    pub fn validate(&self) -> Result<(), ProxyError> {
        let host = self.host.trim();
        if host.is_empty() || host.chars().any(char::is_whitespace) {
            return Err(ProxyError::InvalidHost);
        }
        if self.port == 0 {
            return Err(ProxyError::InvalidPort);
        }
        Ok(())
    }
}

/// Build the shared HTTP client. In System mode, the caller must pass an endpoint
/// resolved by the OS adapter. Missing system configuration is an error, never a
/// silent direct-connection fallback.
pub fn build_http_client(
    config: &NetworkConfig,
    credentials: Option<&ProxyCredentials>,
    system_proxy: Option<&ProxyEndpoint>,
) -> Result<Client, ProxyError> {
    match config.mode {
        ProxyMode::Disabled => Client::builder()
            .no_proxy()
            .build()
            .map_err(|error| ProxyError::ClientBuild(error.to_string())),
        ProxyMode::System => {
            let endpoint = system_proxy.ok_or(ProxyError::SystemProxyUnavailable)?;
            build_proxied_client(endpoint, &config.bypass_hosts, credentials)
        }
        ProxyMode::Manual => {
            let endpoint = config
                .manual_proxy
                .as_ref()
                .ok_or(ProxyError::ManualProxyMissing)?;
            build_proxied_client(endpoint, &config.bypass_hosts, credentials)
        }
    }
}

fn build_proxied_client(
    endpoint: &ProxyEndpoint,
    bypass_hosts: &[String],
    credentials: Option<&ProxyCredentials>,
) -> Result<Client, ProxyError> {
    endpoint.validate()?;

    let host = endpoint.host.trim();
    let authority = if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{}", endpoint.port)
    } else {
        format!("{host}:{}", endpoint.port)
    };
    let raw_url = format!("{}://{authority}", endpoint.protocol.scheme());
    let mut proxy_url = Url::parse(&raw_url)
        .map_err(|error| ProxyError::InvalidProxyUrl(error.to_string()))?;

    if let Some(credentials) = credentials {
        if credentials.username.is_empty() || credentials.password.is_empty() {
            return Err(ProxyError::IncompleteCredentials);
        }
        if endpoint.protocol == ProxyProtocol::Socks5 {
            proxy_url
                .set_username(&credentials.username)
                .map_err(|_| ProxyError::InvalidProxyUrl("invalid proxy username".into()))?;
            proxy_url
                .set_password(Some(&credentials.password))
                .map_err(|_| ProxyError::InvalidProxyUrl("invalid proxy password".into()))?;
        }
    }

    let mut proxy = Proxy::all(proxy_url.as_str())
        .map_err(|error| ProxyError::InvalidProxyUrl(error.to_string()))?;

    let bypass = bypass_hosts
        .iter()
        .map(|host| host.trim())
        .filter(|host| !host.is_empty())
        .collect::<Vec<_>>()
        .join(",");
    if !bypass.is_empty() {
        proxy = proxy.no_proxy(NoProxy::from_string(&bypass));
    }
    if let Some(credentials) = credentials {
        if endpoint.protocol != ProxyProtocol::Socks5 {
            proxy = proxy.basic_auth(&credentials.username, &credentials.password);
        }
    }

    Client::builder()
        .no_proxy()
        .proxy(proxy)
        .build()
        .map_err(|error| ProxyError::ClientBuild(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoint(protocol: ProxyProtocol) -> ProxyEndpoint {
        ProxyEndpoint {
            protocol,
            host: "127.0.0.1".into(),
            port: 8080,
        }
    }

    #[test]
    fn direct_mode_disables_ambient_proxy_configuration() {
        let client = build_http_client(&NetworkConfig::default(), None, None);
        assert!(client.is_ok());
    }

    #[test]
    fn system_mode_never_falls_back_to_direct() {
        let config = NetworkConfig {
            mode: ProxyMode::System,
            ..NetworkConfig::default()
        };
        assert_eq!(
            build_http_client(&config, None, None).err().unwrap(),
            ProxyError::SystemProxyUnavailable
        );
    }

    #[test]
    fn manual_mode_requires_valid_endpoint() {
        let missing = NetworkConfig {
            mode: ProxyMode::Manual,
            ..NetworkConfig::default()
        };
        assert_eq!(
            build_http_client(&missing, None, None).err().unwrap(),
            ProxyError::ManualProxyMissing
        );

        let invalid_port = NetworkConfig {
            mode: ProxyMode::Manual,
            manual_proxy: Some(ProxyEndpoint {
                protocol: ProxyProtocol::Http,
                host: "localhost".into(),
                port: 0,
            }),
            bypass_hosts: Vec::new(),
        };
        assert_eq!(
            build_http_client(&invalid_port, None, None).err().unwrap(),
            ProxyError::InvalidPort
        );
    }

    #[test]
    fn builds_http_https_and_socks5_proxy_clients() {
        for protocol in [ProxyProtocol::Http, ProxyProtocol::Https, ProxyProtocol::Socks5] {
            let config = NetworkConfig {
                mode: ProxyMode::Manual,
                manual_proxy: Some(endpoint(protocol)),
                bypass_hosts: vec!["localhost".into(), ".example.test".into()],
            };
            assert!(build_http_client(&config, None, None).is_ok(), "{protocol:?}");
        }
    }

    #[test]
    fn proxy_credentials_debug_output_is_redacted() {
        let credentials = ProxyCredentials {
            username: "proxy-user".into(),
            password: "do-not-log-this".into(),
        };
        let debug = format!("{credentials:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("do-not-log-this"));
    }

    #[test]
    fn rejects_partial_proxy_credentials() {
        let config = NetworkConfig {
            mode: ProxyMode::Manual,
            manual_proxy: Some(endpoint(ProxyProtocol::Http)),
            bypass_hosts: Vec::new(),
        };
        let credentials = ProxyCredentials {
            username: "proxy-user".into(),
            password: String::new(),
        };
        assert_eq!(
            build_http_client(&config, Some(&credentials), None).err().unwrap(),
            ProxyError::IncompleteCredentials
        );
    }
}
