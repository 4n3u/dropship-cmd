use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::{
    blocking::Client,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT},
};

use crate::models::HeadersMetadata;

pub fn auth_headers(headers: &HeadersMetadata) -> Result<HeaderMap> {
    let mut map = HeaderMap::new();
    map.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&headers.authorization).context("invalid authorization header")?,
    );
    map.insert(
        USER_AGENT,
        HeaderValue::from_str(&headers.user_agent).context("invalid user-agent header")?,
    );
    map.insert(
        HeaderName::from_static("apptype"),
        HeaderValue::from_str(&headers.app_type).context("invalid appType header")?,
    );
    map.insert(
        HeaderName::from_static("device-type"),
        HeaderValue::from_str(&headers.device_type).context("invalid device-type header")?,
    );
    map.insert(
        HeaderName::from_static("auth_url"),
        HeaderValue::from_str(&headers.auth_url).context("invalid auth_url header")?,
    );
    map.insert(
        HeaderName::from_static("version"),
        HeaderValue::from_str(&headers.version).context("invalid version header")?,
    );
    Ok(map)
}

pub fn json_auth_headers(headers: &HeadersMetadata) -> Result<HeaderMap> {
    let mut map = auth_headers(headers)?;
    map.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(map)
}

pub fn data_client() -> Result<Client> {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .context("failed to build data client")
}

pub fn auth_client() -> Result<Client> {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .context("failed to build authenticated client")
}
