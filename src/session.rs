use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::{
    Method,
    blocking::Client,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT},
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::{
    backend::{self, EndpointKind},
    http,
    models::{
        CreateUploadSessionRequestCandidate, DownloadUrlsRequestCandidate, HeadersMetadata,
        InvitationInfoResponseCandidate, PushDeviceInfoCandidate, RoomBubbleInfoResponseCandidate,
        RoomDownloadUrlsResponseCandidate, RoomFileInfoResponseCandidate,
        RoomInfoResponseCandidate, RoomMemberInfoResponseCandidate, SamsungTokenInfo,
        SessionHeadersInput, UploadStorageInfoCandidate,
    },
};

const COMMON_WRAPPER_KEYS: &[&str] = &["data", "result", "response", "payload", "body"];

#[derive(Debug, Clone, Copy)]
pub enum ExtractKind {
    Auto,
    None,
    UploadStorageInfo,
    RoomDownloadUrlsResponse,
    RoomInfoResponse,
    RoomBubbleInfoResponse,
    InvitationInfoResponse,
    RoomMemberInfoResponse,
    RoomFileInfoResponse,
    PushDeviceInfo,
}

impl ExtractKind {
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "auto" => Self::Auto,
            "none" => Self::None,
            "upload-storage-info" => Self::UploadStorageInfo,
            "room-download-urls-response" => Self::RoomDownloadUrlsResponse,
            "room-info-response" => Self::RoomInfoResponse,
            "room-bubble-info-response" => Self::RoomBubbleInfoResponse,
            "invitation-info-response" => Self::InvitationInfoResponse,
            "room-member-info-response" => Self::RoomMemberInfoResponse,
            "room-file-info-response" => Self::RoomFileInfoResponse,
            "push-device-info" => Self::PushDeviceInfo,
            _ => return None,
        })
    }

    fn resolve(self, kind: EndpointKind) -> Self {
        match self {
            Self::Auto => match kind {
                EndpointKind::RoomUploadSession => Self::UploadStorageInfo,
                EndpointKind::DownloadUrls => Self::RoomDownloadUrlsResponse,
                EndpointKind::RoomInfo | EndpointKind::RoomInfoJoined => Self::RoomInfoResponse,
                EndpointKind::RoomInvitation => Self::InvitationInfoResponse,
                EndpointKind::RoomMember => Self::RoomMemberInfoResponse,
                EndpointKind::UserDeviceInfo | EndpointKind::UserDeviceInfoCollection => {
                    Self::PushDeviceInfo
                }
                _ => Self::None,
            },
            other => other,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeEndpointResult {
    pub method: String,
    pub url: String,
    pub status: u16,
    pub extract: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate: Option<CandidateProbeValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_text: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum CandidateProbeValue {
    UploadStorageInfo(UploadStorageInfoCandidate),
    RoomDownloadUrlsResponse(RoomDownloadUrlsResponseCandidate),
    RoomInfoResponse(RoomInfoResponseCandidate),
    RoomBubbleInfoResponse(RoomBubbleInfoResponseCandidate),
    InvitationInfoResponse(InvitationInfoResponseCandidate),
    RoomMemberInfoResponse(RoomMemberInfoResponseCandidate),
    RoomMemberInfoResponseList(Vec<RoomMemberInfoResponseCandidate>),
    RoomFileInfoResponse(RoomFileInfoResponseCandidate),
    RoomFileInfoResponseList(Vec<RoomFileInfoResponseCandidate>),
    PushDeviceInfo(PushDeviceInfoCandidate),
    PushDeviceInfoList(Vec<PushDeviceInfoCandidate>),
}

pub fn probe_endpoint(
    token: &SamsungTokenInfo,
    kind: EndpointKind,
    method: &str,
    suffix: Option<&str>,
    body: Option<&Value>,
    session_headers: Option<&SessionHeadersInput>,
    extract: ExtractKind,
) -> Result<ProbeEndpointResult> {
    let method_upper = method.trim().to_ascii_uppercase();
    let req_method = Method::from_bytes(method_upper.as_bytes())
        .with_context(|| format!("invalid HTTP method: {method}"))?;
    let url = build_probe_url(token, kind, suffix)?;
    let headers = build_probe_headers(token, session_headers, body.is_some())?;

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .context("failed to build probe client")?;

    let mut request = client.request(req_method, &url).headers(headers);
    if let Some(body) = body {
        request = request.json(body);
    }

    let response = request
        .send()
        .with_context(|| format!("request failed for {url}"))?;
    let status = response.status().as_u16();
    let text = response
        .text()
        .context("failed to read probe response body")?;
    let raw_json = serde_json::from_str::<Value>(&text).ok();
    let effective_extract = extract.resolve(kind);
    let candidate = raw_json
        .as_ref()
        .and_then(|value| extract_candidate(value, effective_extract));
    let raw_text = if raw_json.is_none() && !text.is_empty() {
        Some(text)
    } else {
        None
    };

    Ok(ProbeEndpointResult {
        method: method_upper,
        url,
        status,
        extract: extract_name(effective_extract).to_string(),
        candidate,
        raw_json,
        raw_text,
    })
}

pub fn create_upload_session_candidate(
    url: &str,
    headers: &HeadersMetadata,
    request: &CreateUploadSessionRequestCandidate,
) -> Result<ProbeEndpointResult> {
    request_with_headers(
        "POST",
        url,
        headers,
        serde_json::to_value(request).context("failed to serialize upload-session request")?,
        ExtractKind::UploadStorageInfo,
    )
}

pub fn get_download_urls_candidate(
    url: &str,
    headers: &HeadersMetadata,
    request: &DownloadUrlsRequestCandidate,
) -> Result<ProbeEndpointResult> {
    request_with_headers(
        "POST",
        url,
        headers,
        serde_json::to_value(request).context("failed to serialize download-urls request")?,
        ExtractKind::RoomDownloadUrlsResponse,
    )
}

fn build_probe_url(
    token: &SamsungTokenInfo,
    kind: EndpointKind,
    suffix: Option<&str>,
) -> Result<String> {
    let base = match kind {
        EndpointKind::SamsungSignInGate
        | EndpointKind::SamsungSignOutGate
        | EndpointKind::SamsungUserVerifyServiceGate => {
            backend::known_endpoints().samsung_account.account_host_hint
        }
        EndpointKind::SamsungOauthToken => token.auth_server_url.clone(),
        _ => token.api_server_url.clone(),
    };
    backend::build_endpoint(&base, kind, suffix)
}

fn request_with_headers(
    method: &str,
    url: &str,
    headers: &HeadersMetadata,
    body: Value,
    extract: ExtractKind,
) -> Result<ProbeEndpointResult> {
    let method_upper = method.trim().to_ascii_uppercase();
    let req_method = Method::from_bytes(method_upper.as_bytes())
        .with_context(|| format!("invalid HTTP method: {method}"))?;

    let client = http::auth_client()?;
    let response = client
        .request(req_method, url)
        .headers(http::json_auth_headers(headers)?)
        .json(&body)
        .send()
        .with_context(|| format!("request failed for {url}"))?;

    let status = response.status().as_u16();
    let text = response
        .text()
        .context("failed to read candidate response body")?;
    let raw_json = serde_json::from_str::<Value>(&text).ok();
    let candidate = raw_json
        .as_ref()
        .and_then(|value| extract_candidate(value, extract));
    let raw_text = if raw_json.is_none() && !text.is_empty() {
        Some(text)
    } else {
        None
    };

    Ok(ProbeEndpointResult {
        method: method_upper,
        url: url.to_string(),
        status,
        extract: extract_name(extract).to_string(),
        candidate,
        raw_json,
        raw_text,
    })
}

fn build_probe_headers(
    token: &SamsungTokenInfo,
    session_headers: Option<&SessionHeadersInput>,
    has_json_body: bool,
) -> Result<HeaderMap> {
    let input = session_headers.cloned().unwrap_or_default();
    let authorization = input
        .authorization
        .unwrap_or_else(|| format!("Bearer {}", token.access_token));

    let mut map = HeaderMap::new();
    map.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&authorization).context("invalid Authorization header")?,
    );
    map.insert(
        USER_AGENT,
        HeaderValue::from_str(
            input
                .user_agent
                .as_deref()
                .unwrap_or(concat!("dropship-cmd/", env!("CARGO_PKG_VERSION"))),
        )
        .context("invalid User-Agent header")?,
    );
    map.insert(ACCEPT, HeaderValue::from_static("application/json"));

    if let Some(value) = input.app_type.as_deref() {
        map.insert(
            HeaderName::from_static("apptype"),
            HeaderValue::from_str(value).context("invalid appType header")?,
        );
    }
    if let Some(value) = input.device_type.as_deref() {
        map.insert(
            HeaderName::from_static("device-type"),
            HeaderValue::from_str(value).context("invalid device-type header")?,
        );
    }
    if let Some(value) = input.auth_url.as_deref() {
        map.insert(
            HeaderName::from_static("auth_url"),
            HeaderValue::from_str(value).context("invalid auth_url header")?,
        );
    }
    if let Some(value) = input.version.as_deref() {
        map.insert(
            HeaderName::from_static("version"),
            HeaderValue::from_str(value).context("invalid version header")?,
        );
    }
    for (name, value) in input.extra {
        map.insert(
            HeaderName::from_bytes(name.as_bytes())
                .with_context(|| format!("invalid header name: {name}"))?,
            HeaderValue::from_str(&value)
                .with_context(|| format!("invalid header value for {name}"))?,
        );
    }
    if has_json_body {
        map.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    }

    Ok(map)
}

fn extract_candidate(value: &Value, extract: ExtractKind) -> Option<CandidateProbeValue> {
    match extract {
        ExtractKind::Auto | ExtractKind::None => None,
        ExtractKind::UploadStorageInfo => parse_nested::<UploadStorageInfoCandidate>(
            value,
            &["uploadStorageInfo", "data", "result"],
            3,
        )
        .map(CandidateProbeValue::UploadStorageInfo),
        ExtractKind::RoomDownloadUrlsResponse => parse_nested::<RoomDownloadUrlsResponseCandidate>(
            value,
            &["downloadUrls", "roomDownloadUrlsResponse", "data", "result"],
            3,
        )
        .map(CandidateProbeValue::RoomDownloadUrlsResponse),
        ExtractKind::RoomInfoResponse => {
            parse_nested::<RoomInfoResponseCandidate>(value, &["roomInfo", "data", "result"], 3)
                .map(CandidateProbeValue::RoomInfoResponse)
        }
        ExtractKind::RoomBubbleInfoResponse => parse_nested::<RoomBubbleInfoResponseCandidate>(
            value,
            &["roomBubbleInfo", "data", "result"],
            3,
        )
        .map(CandidateProbeValue::RoomBubbleInfoResponse),
        ExtractKind::InvitationInfoResponse => parse_nested::<InvitationInfoResponseCandidate>(
            value,
            &["invitationInfo", "roomInvitation", "data", "result"],
            3,
        )
        .map(CandidateProbeValue::InvitationInfoResponse),
        ExtractKind::RoomMemberInfoResponse => parse_list_or_single(
            value,
            &["members", "member", "data", "result"],
            3,
            CandidateProbeValue::RoomMemberInfoResponse,
            CandidateProbeValue::RoomMemberInfoResponseList,
        ),
        ExtractKind::RoomFileInfoResponse => parse_list_or_single(
            value,
            &["files", "file", "downloadUrls", "data", "result"],
            3,
            CandidateProbeValue::RoomFileInfoResponse,
            CandidateProbeValue::RoomFileInfoResponseList,
        ),
        ExtractKind::PushDeviceInfo => parse_list_or_single(
            value,
            &["pushDevices", "devices", "data", "result"],
            3,
            CandidateProbeValue::PushDeviceInfo,
            CandidateProbeValue::PushDeviceInfoList,
        ),
    }
}

fn parse_list_or_single<T, FSingle, FList>(
    value: &Value,
    preferred_keys: &[&str],
    depth: usize,
    single_variant: FSingle,
    list_variant: FList,
) -> Option<CandidateProbeValue>
where
    T: DeserializeOwned,
    FSingle: FnOnce(T) -> CandidateProbeValue + Copy,
    FList: FnOnce(Vec<T>) -> CandidateProbeValue,
{
    if let Some(items) = parse_nested::<Vec<T>>(value, preferred_keys, depth) {
        return Some(list_variant(items));
    }
    parse_nested::<T>(value, preferred_keys, depth).map(single_variant)
}

fn parse_nested<T>(value: &Value, preferred_keys: &[&str], depth: usize) -> Option<T>
where
    T: DeserializeOwned,
{
    if let Ok(parsed) = serde_json::from_value::<T>(value.clone()) {
        return Some(parsed);
    }
    if depth == 0 {
        return None;
    }

    match value {
        Value::Object(object) => {
            for key in preferred_keys.iter().chain(COMMON_WRAPPER_KEYS.iter()) {
                if let Some(child) = object.get(*key) {
                    if let Some(parsed) = parse_nested(child, preferred_keys, depth - 1) {
                        return Some(parsed);
                    }
                }
            }
            for child in object.values() {
                if let Some(parsed) = parse_nested(child, preferred_keys, depth - 1) {
                    return Some(parsed);
                }
            }
            None
        }
        Value::Array(items) => items
            .iter()
            .find_map(|child| parse_nested(child, preferred_keys, depth - 1)),
        _ => None,
    }
}

fn extract_name(extract: ExtractKind) -> &'static str {
    match extract {
        ExtractKind::Auto => "auto",
        ExtractKind::None => "none",
        ExtractKind::UploadStorageInfo => "upload-storage-info",
        ExtractKind::RoomDownloadUrlsResponse => "room-download-urls-response",
        ExtractKind::RoomInfoResponse => "room-info-response",
        ExtractKind::RoomBubbleInfoResponse => "room-bubble-info-response",
        ExtractKind::InvitationInfoResponse => "invitation-info-response",
        ExtractKind::RoomMemberInfoResponse => "room-member-info-response",
        ExtractKind::RoomFileInfoResponse => "room-file-info-response",
        ExtractKind::PushDeviceInfo => "push-device-info",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_nested_upload_storage_info() {
        let value = serde_json::json!({
            "data": {
                "uploadStorageInfo": {
                    "uploadUrl": "https://upload",
                    "thumbnailUploadUrl": "https://thumb",
                    "pinString": "1234"
                }
            }
        });

        let parsed = extract_candidate(&value, ExtractKind::UploadStorageInfo);
        let Some(CandidateProbeValue::UploadStorageInfo(info)) = parsed else {
            panic!("expected upload storage info candidate");
        };

        assert_eq!(info.upload_url, "https://upload");
        assert_eq!(info.pin_string.as_deref(), Some("1234"));
    }

    #[test]
    fn extracts_member_list_from_room_info() {
        let value = serde_json::json!({
            "roomInfo": {
                "roomId": "room-1",
                "members": [
                    { "userId": "alice", "nickName": "Alice" },
                    { "userId": "bob", "nickName": "Bob" }
                ]
            }
        });

        let parsed = extract_candidate(&value, ExtractKind::RoomMemberInfoResponse);
        let Some(CandidateProbeValue::RoomMemberInfoResponseList(members)) = parsed else {
            panic!("expected member list candidate");
        };

        assert_eq!(members.len(), 2);
        assert_eq!(members[0].user_id, "alice");
    }

    #[test]
    fn auto_extracts_upload_storage_for_room_upload_session() {
        assert!(matches!(
            ExtractKind::Auto.resolve(EndpointKind::RoomUploadSession),
            ExtractKind::UploadStorageInfo
        ));
    }

    #[test]
    fn builds_default_bearer_headers() {
        let token = SamsungTokenInfo {
            access_token: "token".to_string(),
            access_token_expires_in: 3600,
            user_id: "user".to_string(),
            refresh_token: "refresh".to_string(),
            refresh_token_expires_in: 7200,
            auth_server_url: "https://auth.example.test".to_string(),
            api_server_url: "https://api.example.test".to_string(),
        };

        let headers = build_probe_headers(&token, None, true).expect("headers should build");
        assert_eq!(
            headers
                .get(AUTHORIZATION)
                .and_then(|value| value.to_str().ok()),
            Some("Bearer token")
        );
        assert_eq!(
            headers
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("application/json")
        );
    }

    #[test]
    fn uses_account_host_for_sign_in_gate() {
        let token = SamsungTokenInfo {
            access_token: "token".to_string(),
            access_token_expires_in: 3600,
            user_id: "user".to_string(),
            refresh_token: "refresh".to_string(),
            refresh_token_expires_in: 7200,
            auth_server_url: "https://auth.example.test".to_string(),
            api_server_url: "https://api.example.test".to_string(),
        };

        let url = build_probe_url(&token, EndpointKind::SamsungSignInGate, None)
            .expect("url should build");
        assert_eq!(
            url,
            "https://account.samsung.com/accounts/v1/dropship-web/signInGate"
        );
    }
}
