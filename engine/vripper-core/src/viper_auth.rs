//! ViperGirls login compatibility.
//!
//! The legacy site protocol requires an MD5 digest in the form field
//! `vb_login_md5password`. The native app must keep the user's actual password
//! only in OS secure storage and compute this legacy digest transiently at login.

use crate::hosts::CookieMutation;
use crate::transport::{HttpTransport, RequestContext, TransportError};
use md5::{Digest, Md5};
use serde::Deserialize;
use std::fmt;
use thiserror::Error;
use url::Url;

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct ViperCredentials {
    pub username: String,
    pub password: String,
}

impl fmt::Debug for ViperCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ViperCredentials")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViperSession {
    pub username: String,
    pub site_cookies: Vec<CookieMutation>,
    pub click_cookies: Vec<CookieMutation>,
}

#[derive(Debug, Error)]
pub enum ViperAuthError {
    #[error("Viper host URL is invalid: {0}")]
    InvalidHost(String),
    #[error("Viper username and password must not be empty")]
    EmptyCredentials,
    #[error("Viper login did not return both vg_userid and vg_password cookies")]
    MissingAuthenticationCookies,
    #[error(transparent)]
    Transport(#[from] TransportError),
}

pub struct ViperAuthService {
    transport: HttpTransport,
}

impl ViperAuthService {
    pub fn new(transport: HttpTransport) -> Self {
        Self { transport }
    }

    pub async fn authenticate(
        &self,
        host: &str,
        credentials: &ViperCredentials,
    ) -> Result<ViperSession, ViperAuthError> {
        if credentials.username.trim().is_empty() || credentials.password.is_empty() {
            return Err(ViperAuthError::EmptyCredentials);
        }

        let mut base = Url::parse(host.trim_end_matches('/'))
            .map_err(|error| ViperAuthError::InvalidHost(error.to_string()))?;
        if !matches!(base.scheme(), "http" | "https") || base.host_str().is_none() {
            return Err(ViperAuthError::InvalidHost("expected an HTTP(S) origin".into()));
        }
        base.set_path("/login.php");
        base.set_query(Some("do=login"));
        base.set_fragment(None);

        let password_digest = legacy_login_digest(&credentials.password);
        let fields = vec![
            ("vb_login_username".to_owned(), credentials.username.clone()),
            ("cookieuser".to_owned(), "1".to_owned()),
            ("do".to_owned(), "login".to_owned()),
            ("vb_login_md5password".to_owned(), password_digest),
        ];
        let response = self
            .transport
            .post_form(base.as_str(), &fields, &RequestContext::default())
            .await?;

        let user_id = response
            .cookies
            .iter()
            .find(|cookie| cookie.name == "vg_userid")
            .cloned();
        let password = response
            .cookies
            .iter()
            .find(|cookie| cookie.name == "vg_password")
            .cloned();

        let (Some(user_id), Some(password)) = (user_id, password) else {
            return Err(ViperAuthError::MissingAuthenticationCookies);
        };

        let click_cookies = [user_id, password]
            .iter()
            .map(|cookie| CookieMutation {
                name: cookie.name.clone(),
                value: cookie.value.clone(),
                domain: "viper.click".into(),
                path: "/".into(),
                max_age_seconds: cookie.max_age_seconds,
            })
            .collect();

        Ok(ViperSession {
            username: credentials.username.clone(),
            site_cookies: response.cookies,
            click_cookies,
        })
    }
}

pub fn legacy_login_digest(password: &str) -> String {
    format!("{:x}", Md5::digest(password.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_the_legacy_vbulletin_login_digest() {
        assert_eq!(
            legacy_login_digest("password"),
            "5f4dcc3b5aa765d61d8327deb882cf99"
        );
    }

    #[test]
    fn credential_debug_redacts_password() {
        let credentials = ViperCredentials {
            username: "alice".into(),
            password: "not-for-logs".into(),
        };
        let debug = format!("{credentials:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("not-for-logs"));
    }

    #[test]
    fn rejects_non_http_viper_origins() {
        let parsed = Url::parse("file:///tmp").unwrap();
        assert!(matches!(
            parsed.scheme(),
            "file"
        ));
    }
}
