use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};
use reqwest::{
    blocking::multipart::Form,
    Url,
    header::{
        ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE, HeaderMap, HeaderName, HeaderValue, ORIGIN,
        REFERER, USER_AGENT,
    },
};
use serde::Serialize;
use serde_json::Value;

use crate::{
    backend::{self, EndpointKind},
    http,
    models::{
        CompleteWebLoginCandidateInput, DropshipLinkSummary, GoodlockInvitationInfoResponse,
        GoodlockInvitationRequest, GoodlockRoomDownloadUrlsResponse, GoodlockRoomInfoResponse,
        GoodlockRoomMemberInfoResponse, GoodlockSaveRoomRequest, GoodlockWebDownloadCodeRequest,
        GoodlockWebDownloadUrlsResponse, GoodlockWebHeadersInput,
        GoodlockWebIssueAccessTokenRequest, GoodlockWebIssueAccessTokenResponse,
        GoodlockWebUploadSessionCompleteRequest, GoodlockWebUploadSessionRequest,
        GoodlockWebUploadSessionResponse, GoodlockWebUsageResponse,
        GoodlockWebUserInfoResponse, GoodlockWebUserResponse, LoginSession, ReceivedUrlSummary,
        SamsungIssueAccessTokenRequestCandidate, SamsungRefreshTokenRequestCandidate,
        SamsungSignInGateRequestCandidate, SamsungTokenInfo, SamsungUserProfileInfo,
        WebLoginStateInput, WebLoginStateOutput,
    },
};

const COMMON_WRAPPER_KEYS: &[&str] = &["data", "result", "response", "payload", "body"];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthUrlBuildResult {
    pub endpoint: String,
    pub url: String,
    pub query: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthFormBuildResult {
    pub endpoint: String,
    pub form: BTreeMap<String, String>,
    pub form_encoded: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateHttpParseResult<T> {
    pub method: String,
    pub url: String,
    pub status: u16,
    pub body: T,
    pub raw_json: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteWebLoginCandidateOutput {
    pub callback: ReceivedUrlSummary,
    pub token: CandidateHttpParseResult<SamsungTokenInfo>,
    pub user_profile: CandidateHttpParseResult<SamsungUserProfileInfo>,
    pub state: WebLoginStateOutput,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmedGoodlockWebLoginOutput {
    pub token: CandidateHttpParseResult<GoodlockWebIssueAccessTokenResponse>,
    pub user_info: CandidateHttpParseResult<GoodlockWebUserInfoResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<CandidateHttpParseResult<GoodlockWebUserResponse>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawHttpResult {
    pub method: String,
    pub url: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_text: Option<String>,
}

pub fn build_sign_in_gate_url(
    base_url: &str,
    request: &SamsungSignInGateRequestCandidate,
) -> Result<AuthUrlBuildResult> {
    let endpoint = backend::build_endpoint(base_url, EndpointKind::SamsungSignInGate, None)?;
    let mut url = Url::parse(&endpoint)
        .with_context(|| format!("failed to parse sign-in endpoint URL {endpoint}"))?;
    let query = sign_in_gate_query(request);
    {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in &query {
            pairs.append_pair(key, value);
        }
    }

    Ok(AuthUrlBuildResult {
        endpoint,
        url: url.to_string(),
        query,
    })
}

pub fn build_auth_form(
    base_url: &str,
    kind: EndpointKind,
    request: &SamsungIssueAccessTokenRequestCandidate,
) -> Result<AuthFormBuildResult> {
    let endpoint = backend::build_endpoint(base_url, kind, None)?;
    let form = auth_form_pairs(request);
    let form_encoded = encode_form_pairs(&form)?;

    Ok(AuthFormBuildResult {
        endpoint,
        form,
        form_encoded,
    })
}

pub fn parse_received_url(raw_url: &str) -> Result<ReceivedUrlSummary> {
    let url =
        Url::parse(raw_url).with_context(|| format!("failed to parse received URL {raw_url}"))?;
    let query = url
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect::<BTreeMap<_, _>>();

    Ok(ReceivedUrlSummary {
        raw_url: raw_url.to_string(),
        scheme: url.scheme().to_string(),
        host: url.host_str().map(ToString::to_string),
        path: url.path().to_string(),
        fragment: url.fragment().map(ToString::to_string),
        query: query.clone(),
        code: query.get("code").cloned(),
        state: query
            .get("state")
            .cloned()
            .or_else(|| query.get("stateCode").cloned()),
        error: query.get("error").cloned(),
        error_description: query.get("error_description").cloned(),
    })
}

pub fn classify_dropship_link(raw_url: &str) -> Result<DropshipLinkSummary> {
    let parsed = parse_received_url(raw_url)?;
    let segments = parsed
        .path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let main_activity_route =
        build_main_activity_route(&parsed.path, &parsed.query, parsed.fragment.as_deref());

    let mut category = "unknown".to_string();
    let mut suggested_flutter_route = main_activity_route.clone();
    let mut candidate_pin = None;
    let mut keyword = None;
    let mut notes = Vec::new();

    if segments.len() == 1 {
        match segments[0].as_str() {
            "about" => {
                category = "goodlock-shortcut".to_string();
                suggested_flutter_route = Some("/about".to_string());
            }
            "send" => {
                category = "goodlock-shortcut".to_string();
                suggested_flutter_route = Some("/filePicker".to_string());
            }
            "receive" => {
                category = "goodlock-shortcut".to_string();
                suggested_flutter_route = Some("/receive".to_string());
            }
            segment if is_pin_like(segment) => {
                category = "pin-candidate".to_string();
                candidate_pin = Some(segment.to_string());
                notes.push("single path segment matches AOT pin route patterns".to_string());
            }
            _ => {}
        }
    }

    if category == "unknown" && parsed.path == "/loginWaiting" {
        category = "auth-callback-route".to_string();
        suggested_flutter_route = Some("/loginWaiting".to_string());
        notes.push("matches LoginWaitingRoute path recovered from AOT".to_string());
    }

    if category == "unknown" && segments.len() == 2 {
        if is_keyword_segment(&segments[0]) && is_six_digit_code(&segments[1]) {
            category = "keyword-code-candidate".to_string();
            keyword = Some(segments[0].clone());
            candidate_pin = Some(format!("{}{}", segments[0], segments[1]));
            notes.push(
                "matches :keyword([a-z]{3,10})/:code(\\d{6}) route pattern from AOT".to_string(),
            );
        }
    }

    if category == "unknown"
        && segments.len() >= 3
        && segments[0] == "room"
        && segments[1] == "code"
    {
        category = "room-code-route-candidate".to_string();
        candidate_pin = segments.last().cloned();
        notes.push("matches /room/code/ path fragment recovered from AOT".to_string());
    }

    if category == "unknown" && (parsed.code.is_some() || parsed.error.is_some()) {
        category = "auth-callback-candidate".to_string();
        suggested_flutter_route = Some("/loginWaiting".to_string());
        notes.push("contains OAuth-style code/error query parameters".to_string());
    }

    Ok(DropshipLinkSummary {
        raw_url: parsed.raw_url,
        scheme: parsed.scheme,
        host: parsed.host,
        path: parsed.path,
        fragment: parsed.fragment,
        query: parsed.query,
        main_activity_route,
        category,
        suggested_flutter_route,
        candidate_pin,
        keyword,
        notes,
    })
}

pub fn evaluate_web_login_state(input: &WebLoginStateInput) -> Result<WebLoginStateOutput> {
    let callback = input
        .received_url
        .as_deref()
        .map(parse_received_url)
        .transpose()?;
    let link = input
        .received_url
        .as_deref()
        .map(classify_dropship_link)
        .transpose()?;

    let mut next_actions = Vec::new();
    let mut evidence = Vec::new();
    let login_state = if !input.has_connectivity {
        evidence.push("connectivity flag is false".to_string());
        next_actions.push("restore network connectivity before retrying login".to_string());
        "loginFailed"
    } else if callback
        .as_ref()
        .and_then(|item| item.error.as_ref())
        .is_some()
    {
        let callback = callback
            .as_ref()
            .expect("callback should exist when error exists");
        evidence.push(format!(
            "callback URL contains error={}",
            callback.error.as_deref().unwrap_or_default()
        ));
        next_actions
            .push("inspect callback error and rebuild the sign-in request if needed".to_string());
        "loginFailed"
    } else if input.token_info.is_some() && input.user_profile.is_some() {
        evidence.push("token_info is present".to_string());
        evidence.push("user_profile is present".to_string());
        next_actions
            .push("persist token and profile for future authenticated API calls".to_string());
        "loggedIn"
    } else if input.token_info.is_some() {
        evidence.push("token_info is present".to_string());
        next_actions.push(
            "call samsung-user-info or use stored profile data to complete login state".to_string(),
        );
        "loggingIn"
    } else if callback
        .as_ref()
        .and_then(|item| item.code.as_ref())
        .is_some()
    {
        let callback = callback
            .as_ref()
            .expect("callback should exist when code exists");
        evidence.push(format!(
            "callback URL contains code={}",
            callback.code.as_deref().unwrap_or_default()
        ));
        next_actions.push(
            "exchange the callback code for token_info with samsung-issue-access-token or samsung-oauth-token"
                .to_string(),
        );
        "loggingIn"
    } else if input.due_to_token_expiration.unwrap_or(false) {
        evidence.push("due_to_token_expiration is true".to_string());
        next_actions.push("attempt refresh-token or repeat browser login flow".to_string());
        "loggingOut"
    } else {
        if let Some(link) = link.as_ref() {
            evidence.push(format!("link category classified as {}", link.category));
        }
        next_actions
            .push("open the sign-in gate in a browser and wait for callback URL".to_string());
        "notLoggedIn"
    };

    Ok(WebLoginStateOutput {
        login_state: login_state.to_string(),
        callback,
        link,
        next_actions,
        evidence,
    })
}

pub fn request_samsung_token_candidate(
    url: &str,
    request: &SamsungIssueAccessTokenRequestCandidate,
) -> Result<CandidateHttpParseResult<SamsungTokenInfo>> {
    let form_encoded = encode_form_pairs(&auth_form_pairs(request))?;
    let client = http::data_client()?;
    let response = client
        .post(url)
        .header(ACCEPT, "application/json")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(form_encoded)
        .send()
        .with_context(|| format!("POST failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse token JSON response from {url}"))?;
    let body = normalize_token_info(&raw_json)
        .context("failed to normalize SamsungTokenInfo from token response")?;

    Ok(CandidateHttpParseResult {
        method: "POST".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn fetch_samsung_user_profile_candidate(
    url: &str,
    token_info: &SamsungTokenInfo,
) -> Result<CandidateHttpParseResult<SamsungUserProfileInfo>> {
    let client = http::data_client()?;
    let response = client
        .get(url)
        .header(ACCEPT, "application/json")
        .header(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token_info.access_token))
                .context("invalid bearer token header value")?,
        )
        .send()
        .with_context(|| format!("GET failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse user profile JSON response from {url}"))?;
    let body = normalize_user_profile_info(&raw_json)
        .context("failed to normalize SamsungUserProfileInfo from user profile response")?;

    Ok(CandidateHttpParseResult {
        method: "GET".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn complete_web_login_candidate(
    input: &CompleteWebLoginCandidateInput,
) -> Result<CompleteWebLoginCandidateOutput> {
    let callback = parse_received_url(&input.received_url)?;
    let code = callback
        .code
        .clone()
        .context("received_url did not contain a code query parameter")?;

    let mut request = SamsungIssueAccessTokenRequestCandidate {
        grant_type: input.grant_type.clone(),
        client_id: Some(input.client_id.clone()),
        redirect_uri: Some(input.redirect_uri.clone()),
        code: Some(code),
        refresh_token: None,
        code_verifier: input.code_verifier.clone(),
        extra: input.extra_form.clone(),
    };

    if request.code_verifier.is_none() {
        request.extra.remove("code_verifier");
    }

    let token = request_samsung_token_candidate(&input.token_url, &request)?;
    let user_profile = fetch_samsung_user_profile_candidate(&input.user_info_url, &token.body)?;
    let state = evaluate_web_login_state(&WebLoginStateInput {
        received_url: Some(input.received_url.clone()),
        token_info: Some(token.body.clone()),
        user_profile: Some(user_profile.body.clone()),
        has_connectivity: true,
        due_to_token_expiration: None,
    })?;

    Ok(CompleteWebLoginCandidateOutput {
        callback,
        token,
        user_profile,
        state,
    })
}

pub fn request_samsung_refresh_token_candidate(
    url: &str,
    request: &SamsungRefreshTokenRequestCandidate,
) -> Result<CandidateHttpParseResult<SamsungTokenInfo>> {
    let request = SamsungIssueAccessTokenRequestCandidate {
        grant_type: request.grant_type.clone(),
        client_id: request.client_id.clone(),
        redirect_uri: request.redirect_uri.clone(),
        code: None,
        refresh_token: Some(request.refresh_token.clone()),
        code_verifier: None,
        extra: request.extra.clone(),
    };
    request_samsung_token_candidate(url, &request)
}

pub fn request_goodlock_web_issue_access_token(
    url: &str,
    request: &GoodlockWebIssueAccessTokenRequest,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<GoodlockWebIssueAccessTokenResponse>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .post(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::IssueAccessToken,
        )?)
        .body(
            serde_json::to_string(request)
                .context("failed to serialize confirmed issueAccessToken JSON body")?,
        )
        .send()
        .with_context(|| format!("POST failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse issueAccessToken JSON response from {url}"))?;
    if !status.is_success() {
        bail!(
            "goodlock issueAccessToken failed with status {}: {}",
            status,
            raw_json
        );
    }
    let body: GoodlockWebIssueAccessTokenResponse = serde_json::from_value(raw_json.clone())
        .context("failed to decode GoodlockWebIssueAccessTokenResponse")?;

    Ok(CandidateHttpParseResult {
        method: "POST".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn request_goodlock_web_refresh_token(
    url: &str,
    refresh_token: &str,
    auth_url: &str,
) -> Result<CandidateHttpParseResult<GoodlockWebIssueAccessTokenResponse>> {
    let mut headers = default_goodlock_headers();
    headers.auth_url = auth_url.to_string();

    let mut request_headers = build_goodlock_headers(&headers, GoodlockRequestKind::RefreshToken)?;
    request_headers.insert(
        HeaderName::from_static("refresh_token"),
        HeaderValue::from_str(refresh_token).context("invalid refresh_token header")?,
    );

    let client = http::data_client()?;
    let response = client
        .post(url)
        .headers(request_headers)
        .send()
        .with_context(|| format!("POST failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse refreshToken JSON response from {url}"))?;
    if !status.is_success() {
        bail!(
            "goodlock refreshToken failed with status {}: {}",
            status,
            raw_json
        );
    }
    let body: GoodlockWebIssueAccessTokenResponse = serde_json::from_value(raw_json.clone())
        .context("failed to decode GoodlockWebIssueAccessTokenResponse from refreshToken")?;

    Ok(CandidateHttpParseResult {
        method: "POST".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn fetch_goodlock_web_user_info(
    url: &str,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<GoodlockWebUserInfoResponse>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .post(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::UserInfo,
        )?)
        .body("")
        .send()
        .with_context(|| format!("POST failed for {url}"))?;

    let status = response.status();
    let raw_text = response
        .text()
        .with_context(|| format!("failed to read userInfo response body from {url}"))?;
    if !status.is_success() {
        if let Ok(raw_json) = serde_json::from_str::<Value>(&raw_text) {
            bail!(
                "goodlock userInfo failed with status {}: {}",
                status,
                raw_json
            );
        }
        bail!(
            "goodlock userInfo failed with status {}: {}",
            status,
            raw_text
        );
    }
    let raw_json = serde_json::from_str::<Value>(&raw_text)
        .with_context(|| format!("failed to parse userInfo JSON response from {url}"))?;
    let body: GoodlockWebUserInfoResponse = serde_json::from_value(raw_json.clone())
        .context("failed to decode GoodlockWebUserInfoResponse")?;

    Ok(CandidateHttpParseResult {
        method: "POST".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn fetch_goodlock_web_user(
    url: &str,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<GoodlockWebUserResponse>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .get(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::ApiUserGet,
        )?)
        .send()
        .with_context(|| format!("GET failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse user JSON response from {url}"))?;
    if !status.is_success() {
        bail!("goodlock user failed with status {}: {}", status, raw_json);
    }
    let body: GoodlockWebUserResponse = serde_json::from_value(raw_json.clone())
        .context("failed to decode GoodlockWebUserResponse")?;

    Ok(CandidateHttpParseResult {
        method: "GET".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn fetch_goodlock_web_usage(
    url: &str,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<GoodlockWebUsageResponse>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .get(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::ApiUserGet,
        )?)
        .send()
        .with_context(|| format!("GET failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse usage JSON response from {url}"))?;
    if !status.is_success() {
        bail!("goodlock usage failed with status {}: {}", status, raw_json);
    }
    let body: GoodlockWebUsageResponse = serde_json::from_value(raw_json.clone())
        .context("failed to decode GoodlockWebUsageResponse")?;

    Ok(CandidateHttpParseResult {
        method: "GET".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn fetch_goodlock_web_upload_history(
    url: &str,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<Vec<Value>>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .get(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::ApiUserGet,
        )?)
        .send()
        .with_context(|| format!("GET failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse upload history JSON response from {url}"))?;
    if !status.is_success() {
        bail!(
            "goodlock upload history failed with status {}: {}",
            status,
            raw_json
        );
    }
    let body = match raw_json.clone() {
        Value::Array(items) => items,
        other => bail!("expected upload history array but received {}", other),
    };

    Ok(CandidateHttpParseResult {
        method: "GET".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn fetch_goodlock_web_joined_rooms(
    url: &str,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<Vec<Value>>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .get(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::ApiUserGet,
        )?)
        .send()
        .with_context(|| format!("GET failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse joined rooms JSON response from {url}"))?;
    if !status.is_success() {
        bail!(
            "goodlock joined rooms failed with status {}: {}",
            status,
            raw_json
        );
    }
    let body = match raw_json.clone() {
        Value::Array(items) => items,
        other => bail!("expected joined rooms array but received {}", other),
    };

    Ok(CandidateHttpParseResult {
        method: "GET".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn create_goodlock_room(
    url: &str,
    request: &GoodlockSaveRoomRequest,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<GoodlockRoomInfoResponse>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .post(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::MultipartPost,
        )?)
        .multipart(build_room_form(request))
        .send()
        .with_context(|| format!("POST failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse room info JSON response from {url}"))?;
    if !status.is_success() {
        bail!(
            "goodlock create room failed with status {}: {}",
            status,
            raw_json
        );
    }
    let body: GoodlockRoomInfoResponse = serde_json::from_value(raw_json.clone())
        .context("failed to decode GoodlockRoomInfoResponse")?;

    Ok(CandidateHttpParseResult {
        method: "POST".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn fetch_goodlock_room_info(
    url: &str,
    room_id: &str,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<GoodlockRoomInfoResponse>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .get(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::ApiUserGet,
        )?)
        .query(&[("roomId", room_id)])
        .send()
        .with_context(|| format!("GET failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse room info JSON response from {url}"))?;
    if !status.is_success() {
        bail!(
            "goodlock room info failed with status {}: {}",
            status,
            raw_json
        );
    }
    let body: GoodlockRoomInfoResponse = serde_json::from_value(raw_json.clone())
        .context("failed to decode GoodlockRoomInfoResponse")?;

    Ok(CandidateHttpParseResult {
        method: "GET".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn create_goodlock_invitation(
    url: &str,
    request: &GoodlockInvitationRequest,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<GoodlockInvitationInfoResponse>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .post(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::MultipartPost,
        )?)
        .multipart(build_invitation_form(request))
        .send()
        .with_context(|| format!("POST failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse invitation JSON response from {url}"))?;
    if !status.is_success() {
        bail!(
            "goodlock invitation failed with status {}: {}",
            status,
            raw_json
        );
    }
    let body: GoodlockInvitationInfoResponse = serde_json::from_value(raw_json.clone())
        .context("failed to decode GoodlockInvitationInfoResponse")?;

    Ok(CandidateHttpParseResult {
        method: "POST".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn fetch_goodlock_room_members(
    url: &str,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<Vec<GoodlockRoomMemberInfoResponse>>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .get(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::ApiUserGet,
        )?)
        .send()
        .with_context(|| format!("GET failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse room members JSON response from {url}"))?;
    if !status.is_success() {
        bail!(
            "goodlock room members failed with status {}: {}",
            status,
            raw_json
        );
    }
    let body: Vec<GoodlockRoomMemberInfoResponse> = serde_json::from_value(raw_json.clone())
        .context("failed to decode room members response")?;

    Ok(CandidateHttpParseResult {
        method: "GET".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn request_goodlock_web_download_urls(
    url: &str,
    request: &GoodlockWebDownloadCodeRequest,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<GoodlockWebDownloadUrlsResponse>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .post(url)
        .headers(build_goodlock_headers(&headers, GoodlockRequestKind::JsonPost)?)
        .json(request)
        .send()
        .with_context(|| format!("POST failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse downloadUrls JSON response from {url}"))?;
    if !status.is_success() {
        bail!(
            "goodlock downloadUrls failed with status {}: {}",
            status,
            raw_json
        );
    }
    let body: GoodlockWebDownloadUrlsResponse = serde_json::from_value(raw_json.clone())
        .context("failed to decode GoodlockWebDownloadUrlsResponse")?;

    Ok(CandidateHttpParseResult {
        method: "POST".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn join_goodlock_room(
    url: &str,
    room_id: &str,
    password: &str,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<Value>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .post(url)
        .headers(build_goodlock_headers(&headers, GoodlockRequestKind::JsonPost)?)
        .json(&serde_json::json!({
            "roomId": room_id,
            "password": password,
        }))
        .send()
        .with_context(|| format!("POST failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse join room JSON response from {url}"))?;
    if !status.is_success() {
        bail!("goodlock join room failed with status {}: {}", status, raw_json);
    }

    Ok(CandidateHttpParseResult {
        method: "POST".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body: raw_json.clone(),
        raw_json,
    })
}

pub fn delete_goodlock_room_invitation(
    url: &str,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<RawHttpResult> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .delete(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::ApiUserGet,
        )?)
        .send()
        .with_context(|| format!("DELETE failed for {url}"))?;

    let status = response.status();
    let raw_text = response
        .text()
        .with_context(|| format!("failed to read room invitation delete response from {url}"))?;
    let raw_json = serde_json::from_str::<Value>(&raw_text).ok();
    if !status.is_success() {
        if let Some(raw_json) = &raw_json {
            bail!(
                "goodlock delete room invitation failed with status {}: {}",
                status,
                raw_json
            );
        }
        bail!(
            "goodlock delete room invitation failed with status {}: {}",
            status,
            raw_text
        );
    }

    Ok(RawHttpResult {
        method: "DELETE".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        raw_json,
        raw_text: if raw_text.is_empty() {
            None
        } else {
            Some(raw_text)
        },
    })
}

pub fn fetch_goodlock_room_codes(
    url: &str,
    size: u64,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<Value>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .get(url)
        .headers(build_goodlock_headers(&headers, GoodlockRequestKind::ApiUserGet)?)
        .query(&[("size", size)])
        .send()
        .with_context(|| format!("GET failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse room codes JSON response from {url}"))?;
    if !status.is_success() {
        bail!("goodlock room codes failed with status {}: {}", status, raw_json);
    }

    Ok(CandidateHttpParseResult {
        method: "GET".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body: raw_json.clone(),
        raw_json,
    })
}

pub fn request_goodlock_room_download_urls(
    url: &str,
    request: &crate::models::GoodlockDownloadUrlsRequest,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<Vec<GoodlockRoomDownloadUrlsResponse>>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .post(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::JsonPost,
        )?)
        .json(request)
        .send()
        .with_context(|| format!("POST failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse download URLs JSON response from {url}"))?;
    if !status.is_success() {
        bail!(
            "goodlock room download urls failed with status {}: {}",
            status,
            raw_json
        );
    }
    let body: Vec<GoodlockRoomDownloadUrlsResponse> = serde_json::from_value(raw_json.clone())
        .context("failed to decode room download URLs response")?;

    Ok(CandidateHttpParseResult {
        method: "POST".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn create_goodlock_upload_session(
    url: &str,
    request: &GoodlockWebUploadSessionRequest,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<CandidateHttpParseResult<GoodlockWebUploadSessionResponse>> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .post(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::JsonPost,
        )?)
        .json(request)
        .send()
        .with_context(|| format!("POST failed for {url}"))?;

    let status = response.status();
    let raw_json = response
        .json::<Value>()
        .with_context(|| format!("failed to parse uploadSession JSON response from {url}"))?;
    if !status.is_success() {
        bail!(
            "goodlock uploadSession failed with status {}: {}",
            status,
            raw_json
        );
    }
    let body: GoodlockWebUploadSessionResponse = serde_json::from_value(raw_json.clone())
        .context("failed to decode GoodlockWebUploadSessionResponse")?;

    Ok(CandidateHttpParseResult {
        method: "POST".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        body,
        raw_json,
    })
}

pub fn complete_goodlock_upload_session(
    url: &str,
    request: &GoodlockWebUploadSessionCompleteRequest,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<RawHttpResult> {
    let headers = headers.cloned().unwrap_or_else(default_goodlock_headers);
    let client = http::data_client()?;
    let response = client
        .post(url)
        .headers(build_goodlock_headers(
            &headers,
            GoodlockRequestKind::JsonPost,
        )?)
        .json(request)
        .send()
        .with_context(|| format!("POST failed for {url}"))?;

    let status = response.status();
    let text = response
        .text()
        .with_context(|| format!("failed to read uploadSession complete response from {url}"))?;
    let raw_json = serde_json::from_str::<Value>(&text).ok();
    if !status.is_success() {
        match &raw_json {
            Some(raw_json) => bail!(
                "goodlock uploadSession complete failed with status {}: {}",
                status,
                raw_json
            ),
            None => bail!(
                "goodlock uploadSession complete failed with status {}: {}",
                status,
                text
            ),
        }
    }

    Ok(RawHttpResult {
        method: "POST".to_string(),
        url: url.to_string(),
        status: status.as_u16(),
        raw_json,
        raw_text: if text.is_empty() { None } else { Some(text) },
    })
}

pub fn complete_goodlock_web_login_confirmed(
    issue_token_url: &str,
    issue_token_request: &GoodlockWebIssueAccessTokenRequest,
    user_info_url: &str,
    user_url: Option<&str>,
    headers: Option<&GoodlockWebHeadersInput>,
) -> Result<ConfirmedGoodlockWebLoginOutput> {
    let token =
        request_goodlock_web_issue_access_token(issue_token_url, issue_token_request, headers)?;
    let user_info = fetch_goodlock_web_user_info(user_info_url, headers)?;
    let user = user_url
        .map(|url| fetch_goodlock_web_user(url, headers))
        .transpose()?;

    Ok(ConfirmedGoodlockWebLoginOutput {
        token,
        user_info,
        user,
    })
}

pub fn build_samsung_token_info_from_goodlock(
    callback: &ReceivedUrlSummary,
    token: &GoodlockWebIssueAccessTokenResponse,
    user_id: &str,
) -> Result<SamsungTokenInfo> {
    let auth_server_url = callback
        .query
        .get("auth_server_url")
        .cloned()
        .context("callback URL missing auth_server_url")?;
    let api_server_url = callback
        .query
        .get("api_server_url")
        .cloned()
        .context("callback URL missing api_server_url")?;

    Ok(SamsungTokenInfo {
        access_token: token.access_token.clone(),
        access_token_expires_in: token.access_token_expiry,
        user_id: user_id.to_string(),
        refresh_token: token.refresh_token.clone(),
        refresh_token_expires_in: token.refresh_token_expiry,
        auth_server_url: ensure_scheme(&auth_server_url),
        api_server_url: ensure_scheme(&api_server_url),
    })
}

pub fn build_login_session(
    sign_in_url: String,
    callback: ReceivedUrlSummary,
    token: GoodlockWebIssueAccessTokenResponse,
    user_info: GoodlockWebUserInfoResponse,
    user: Option<GoodlockWebUserResponse>,
) -> Result<LoginSession> {
    let user_id = user
        .as_ref()
        .map(|item| item.user_id.as_str())
        .unwrap_or(user_info.user_id.as_str());
    let samsung_token_info = build_samsung_token_info_from_goodlock(&callback, &token, user_id)?;

    Ok(LoginSession {
        sign_in_url,
        callback,
        samsung_token_info,
        goodlock_token: token,
        user_info,
        user,
    })
}

pub fn rebuild_login_session(
    previous: &LoginSession,
    token: GoodlockWebIssueAccessTokenResponse,
    user_info: GoodlockWebUserInfoResponse,
    user: Option<GoodlockWebUserResponse>,
) -> Result<LoginSession> {
    let samsung_token_info =
        build_samsung_token_info_from_goodlock(&previous.callback, &token, &user_info.user_id)?;

    Ok(LoginSession {
        sign_in_url: previous.sign_in_url.clone(),
        callback: previous.callback.clone(),
        samsung_token_info,
        goodlock_token: token,
        user_info,
        user,
    })
}

fn sign_in_gate_query(request: &SamsungSignInGateRequestCandidate) -> BTreeMap<String, String> {
    let mut query = BTreeMap::new();
    query.insert("client_id".to_string(), request.client_id.clone());
    query.insert("redirect_uri".to_string(), request.redirect_uri.clone());
    query.insert("response_type".to_string(), request.response_type.clone());
    if let Some(state) = &request.state {
        query.insert("state".to_string(), state.clone());
    }
    for (key, value) in &request.extra {
        query.insert(key.clone(), value.clone());
    }
    query
}

fn auth_form_pairs(request: &SamsungIssueAccessTokenRequestCandidate) -> BTreeMap<String, String> {
    let mut form = BTreeMap::new();
    form.insert("grant_type".to_string(), request.grant_type.clone());
    if let Some(client_id) = &request.client_id {
        form.insert("client_id".to_string(), client_id.clone());
    }
    if let Some(redirect_uri) = &request.redirect_uri {
        form.insert("redirect_uri".to_string(), redirect_uri.clone());
    }
    if let Some(code) = &request.code {
        form.insert("code".to_string(), code.clone());
    }
    if let Some(refresh_token) = &request.refresh_token {
        form.insert("refresh_token".to_string(), refresh_token.clone());
    }
    if let Some(code_verifier) = &request.code_verifier {
        form.insert("code_verifier".to_string(), code_verifier.clone());
    }
    for (key, value) in &request.extra {
        form.insert(key.clone(), value.clone());
    }
    form
}

fn encode_form_pairs(form: &BTreeMap<String, String>) -> Result<String> {
    let mut url = Url::parse("https://example.invalid/")
        .context("failed to create placeholder URL for form encoding")?;
    {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in form {
            pairs.append_pair(key, value);
        }
    }
    Ok(url.query().unwrap_or_default().to_string())
}

fn normalize_token_info(value: &Value) -> Option<SamsungTokenInfo> {
    let object = find_candidate_object(value, 4)?;
    Some(SamsungTokenInfo {
        access_token: extract_string(object, &["access_token", "accessToken"])?,
        access_token_expires_in: extract_u64(
            object,
            &["access_token_expires_in", "token_expires_in", "expires_in"],
        )
        .unwrap_or(0),
        user_id: extract_string(object, &["userId", "user_id"])?,
        refresh_token: extract_string(object, &["refresh_token", "refreshToken"])
            .unwrap_or_default(),
        refresh_token_expires_in: extract_u64(
            object,
            &["refresh_token_expires_in", "refreshTokenExpiresIn"],
        )
        .unwrap_or(0),
        auth_server_url: extract_string(object, &["auth_server_url", "authServerUrl"])?,
        api_server_url: extract_string(object, &["api_server_url", "apiServerUrl"])?,
    })
}

fn normalize_user_profile_info(value: &Value) -> Option<SamsungUserProfileInfo> {
    let object = find_candidate_object(value, 4)?;
    let login_id = extract_string(object, &["login_id", "loginId"]);
    Some(SamsungUserProfileInfo {
        user_id: extract_string(object, &["userId", "user_id"])?,
        country_code: extract_string(object, &["countryCode", "cc"]),
        birth_data: extract_string(object, &["birthData", "birthday"]),
        email: extract_string(object, &["email"]).or_else(|| login_id.clone()),
        preferred_user_name: extract_string(object, &["preferredUserName"]).or(login_id),
        nickname: extract_string(object, &["nickname", "nickName"]),
        given_name: extract_string(object, &["givenName", "account_given_name"]),
        family_name: extract_string(object, &["familyName", "account_family_name"]),
    })
}

fn find_candidate_object<'a>(
    value: &'a Value,
    depth: usize,
) -> Option<&'a serde_json::Map<String, Value>> {
    match value {
        Value::Object(map) => {
            if map.contains_key("access_token")
                || map.contains_key("accessToken")
                || map.contains_key("user_id")
                || map.contains_key("userId")
                || map.contains_key("login_id")
                || map.contains_key("email")
            {
                return Some(map);
            }
            if depth == 0 {
                return Some(map);
            }
            for key in COMMON_WRAPPER_KEYS {
                if let Some(child) = map.get(*key) {
                    if let Some(found) = find_candidate_object(child, depth - 1) {
                        return Some(found);
                    }
                }
            }
            for child in map.values() {
                if let Some(found) = find_candidate_object(child, depth - 1) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(items) if depth > 0 => items
            .iter()
            .find_map(|child| find_candidate_object(child, depth - 1)),
        _ => None,
    }
}

fn extract_string(map: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| match map.get(*key) {
        Some(Value::String(value)) => Some(value.clone()),
        Some(Value::Number(value)) => Some(value.to_string()),
        _ => None,
    })
}

fn extract_u64(map: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<u64> {
    keys.iter().find_map(|key| match map.get(*key) {
        Some(Value::Number(value)) => value.as_u64(),
        Some(Value::String(value)) => value.parse::<u64>().ok(),
        _ => None,
    })
}

fn ensure_scheme(value: &str) -> String {
    if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else {
        format!("https://{value}")
    }
}

#[derive(Debug, Clone, Copy)]
enum GoodlockRequestKind {
    IssueAccessToken,
    RefreshToken,
    UserInfo,
    ApiUserGet,
    JsonPost,
    MultipartPost,
}

fn default_goodlock_headers() -> GoodlockWebHeadersInput {
    GoodlockWebHeadersInput {
        authorization: None,
        cookie: None,
        auth_url: "eu-auth2.samsungosp.com".to_string(),
        device_type: "WEB".to_string(),
        app_type: "dropship_web".to_string(),
        version: "1.2.6".to_string(),
        origin: "https://g2sh.me".to_string(),
        referer: "https://g2sh.me/".to_string(),
        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36".to_string(),
    }
}

fn build_goodlock_headers(
    input: &GoodlockWebHeadersInput,
    kind: GoodlockRequestKind,
) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        HeaderName::from_static("auth_url"),
        HeaderValue::from_str(&input.auth_url).context("invalid auth_url header")?,
    );
    headers.insert(
        HeaderName::from_static("device-type"),
        HeaderValue::from_str(&input.device_type).context("invalid device-type header")?,
    );
    headers.insert(
        ORIGIN,
        HeaderValue::from_str(&input.origin).context("invalid origin header")?,
    );
    headers.insert(
        REFERER,
        HeaderValue::from_str(&input.referer).context("invalid referer header")?,
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&input.user_agent).context("invalid user-agent header")?,
    );
    if let Some(value) = &input.authorization {
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(value).context("invalid authorization header")?,
        );
    }
    if let Some(value) = &input.cookie {
        headers.insert(
            COOKIE,
            HeaderValue::from_str(value).context("invalid cookie header")?,
        );
    }

    match kind {
        GoodlockRequestKind::IssueAccessToken => {
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            );
        }
        GoodlockRequestKind::RefreshToken => {}
        GoodlockRequestKind::UserInfo => {
            headers.insert(
                HeaderName::from_static("apptype"),
                HeaderValue::from_str(&input.app_type).context("invalid apptype header")?,
            );
            headers.insert(
                HeaderName::from_static("version"),
                HeaderValue::from_str(&input.version).context("invalid version header")?,
            );
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }
        GoodlockRequestKind::ApiUserGet => {
            headers.insert(
                HeaderName::from_static("apptype"),
                HeaderValue::from_str(&input.app_type).context("invalid apptype header")?,
            );
            headers.insert(
                HeaderName::from_static("version"),
                HeaderValue::from_str(&input.version).context("invalid version header")?,
            );
        }
        GoodlockRequestKind::JsonPost => {
            headers.insert(
                HeaderName::from_static("apptype"),
                HeaderValue::from_str(&input.app_type).context("invalid apptype header")?,
            );
            headers.insert(
                HeaderName::from_static("version"),
                HeaderValue::from_str(&input.version).context("invalid version header")?,
            );
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }
        GoodlockRequestKind::MultipartPost => {
            headers.insert(
                HeaderName::from_static("apptype"),
                HeaderValue::from_str(&input.app_type).context("invalid apptype header")?,
            );
            headers.insert(
                HeaderName::from_static("version"),
                HeaderValue::from_str(&input.version).context("invalid version header")?,
            );
        }
    }

    Ok(headers)
}

fn build_room_form(request: &GoodlockSaveRoomRequest) -> Form {
    let mut form = Form::new()
        .text("roomId", request.room_id.clone())
        .text("title", request.title.clone())
        .text("expiry", request.expiry.to_string())
        .text("isSecure", request.is_secure.to_string());
    if let Some(password) = &request.password {
        form = form.text("password", password.clone());
    }
    if let Some(background_blur_hash) = &request.background_blur_hash {
        form = form.text("backgroundBlurHash", background_blur_hash.clone());
    }
    form
}

fn build_invitation_form(request: &GoodlockInvitationRequest) -> Form {
    let mut form = Form::new()
        .text("roomId", request.room_id.clone())
        .text("expiry", request.expiry.to_string());
    if let Some(keyword) = &request.keyword {
        form = form.text("keyword", keyword.clone());
    }
    form
}

fn build_main_activity_route(
    path: &str,
    query: &BTreeMap<String, String>,
    fragment: Option<&str>,
) -> Option<String> {
    if path.is_empty() {
        return None;
    }

    let mut route = path.to_string();
    if !query.is_empty() {
        route.push('?');
        route.push_str(&encode_form_pairs(query).ok()?);
    }
    if let Some(fragment) = fragment {
        if !fragment.is_empty() {
            route.push('#');
            route.push_str(fragment);
        }
    }
    Some(route)
}

fn is_keyword_segment(value: &str) -> bool {
    let len = value.len();
    (3..=10).contains(&len) && value.chars().all(|ch| ch.is_ascii_lowercase())
}

fn is_six_digit_code(value: &str) -> bool {
    value.len() == 6 && value.chars().all(|ch| ch.is_ascii_digit())
}

fn is_pin_like(value: &str) -> bool {
    is_six_digit_code(value) || {
        let split = value.find(|ch: char| ch.is_ascii_digit());
        match split {
            Some(index) if index >= 3 && index <= 10 => {
                let (prefix, digits) = value.split_at(index);
                prefix.chars().all(|ch| ch.is_ascii_lowercase()) && is_six_digit_code(digits)
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_sign_in_gate_url_with_default_query_keys() {
        let request = SamsungSignInGateRequestCandidate {
            client_id: "client".to_string(),
            redirect_uri: "dropship://g2sh.me".to_string(),
            response_type: "code".to_string(),
            state: Some("state-123".to_string()),
            extra: BTreeMap::new(),
        };

        let built = build_sign_in_gate_url("https://account.samsung.com", &request)
            .expect("sign-in gate URL should build");

        assert_eq!(
            built.endpoint,
            "https://account.samsung.com/accounts/v1/dropship-web/signInGate"
        );
        assert!(built.url.contains("client_id=client"));
        assert!(built.url.contains("redirect_uri=dropship%3A%2F%2Fg2sh.me"));
        assert!(built.url.contains("response_type=code"));
        assert!(built.url.contains("state=state-123"));
    }

    #[test]
    fn builds_auth_form_with_code_verifier() {
        let request = SamsungIssueAccessTokenRequestCandidate {
            grant_type: "authorization_code".to_string(),
            client_id: Some("client".to_string()),
            redirect_uri: Some("dropship://g2sh.me".to_string()),
            code: Some("auth-code".to_string()),
            refresh_token: None,
            code_verifier: Some("verifier".to_string()),
            extra: BTreeMap::new(),
        };

        let built = build_auth_form(
            "https://api.example.test",
            EndpointKind::SamsungIssueAccessToken,
            &request,
        )
        .expect("auth form should build");

        assert_eq!(
            built.endpoint,
            "https://api.example.test/dropship-web/v1/user/issueAccessToken"
        );
        assert_eq!(
            built.form.get("code_verifier").map(String::as_str),
            Some("verifier")
        );
        assert!(built.form_encoded.contains("grant_type=authorization_code"));
        assert!(built.form_encoded.contains("code=auth-code"));
    }

    #[test]
    fn parses_received_url_code_and_state() {
        let parsed = parse_received_url("dropship://g2sh.me?code=abc123&state=s1")
            .expect("received URL should parse");

        assert_eq!(parsed.scheme, "dropship");
        assert_eq!(parsed.host.as_deref(), Some("g2sh.me"));
        assert_eq!(parsed.code.as_deref(), Some("abc123"));
        assert_eq!(parsed.state.as_deref(), Some("s1"));
    }

    #[test]
    fn classifies_login_waiting_route() {
        let parsed = classify_dropship_link(
            "https://g2sh.me/loginWaiting?code=abc123&state=state-code-placeholder",
        )
        .expect("dropship link should classify");

        assert_eq!(parsed.category, "auth-callback-route");
        assert_eq!(
            parsed.suggested_flutter_route.as_deref(),
            Some("/loginWaiting")
        );
        assert_eq!(
            parsed.main_activity_route.as_deref(),
            Some("/loginWaiting?code=abc123&state=state-code-placeholder")
        );
    }

    #[test]
    fn classifies_goodlock_shortcut() {
        let parsed = classify_dropship_link("dropship://g2sh.me/send")
            .expect("goodlock shortcut should classify");

        assert_eq!(parsed.category, "goodlock-shortcut");
        assert_eq!(
            parsed.suggested_flutter_route.as_deref(),
            Some("/filePicker")
        );
    }

    #[test]
    fn classifies_keyword_code_candidate() {
        let parsed = classify_dropship_link("https://g2sh.me/abc/123456")
            .expect("keyword/code link should classify");

        assert_eq!(parsed.category, "keyword-code-candidate");
        assert_eq!(parsed.keyword.as_deref(), Some("abc"));
        assert_eq!(parsed.candidate_pin.as_deref(), Some("abc123456"));
    }

    #[test]
    fn evaluates_state_as_logging_in_when_callback_has_code() {
        let output = evaluate_web_login_state(&WebLoginStateInput {
            received_url: Some(
                "https://g2sh.me/loginWaiting?code=abc123&state=state-code-placeholder".to_string(),
            ),
            token_info: None,
            user_profile: None,
            has_connectivity: true,
            due_to_token_expiration: None,
        })
        .expect("state evaluation should succeed");

        assert_eq!(output.login_state, "loggingIn");
        assert!(
            output
                .next_actions
                .iter()
                .any(|item| item.contains("exchange the callback code"))
        );
    }

    #[test]
    fn evaluates_state_as_logged_in_when_token_and_profile_exist() {
        let output = evaluate_web_login_state(&WebLoginStateInput {
            received_url: None,
            token_info: Some(crate::models::SamsungTokenInfo {
                access_token: "token".to_string(),
                access_token_expires_in: 3600,
                user_id: "user".to_string(),
                refresh_token: "refresh".to_string(),
                refresh_token_expires_in: 7200,
                auth_server_url: "https://auth.example.test".to_string(),
                api_server_url: "https://api.example.test".to_string(),
            }),
            user_profile: Some(crate::models::SamsungUserProfileInfo {
                user_id: "user".to_string(),
                country_code: Some("KR".to_string()),
                birth_data: None,
                email: Some("user@example.test".to_string()),
                preferred_user_name: Some("user@example.test".to_string()),
                nickname: None,
                given_name: None,
                family_name: None,
            }),
            has_connectivity: true,
            due_to_token_expiration: None,
        })
        .expect("state evaluation should succeed");

        assert_eq!(output.login_state, "loggedIn");
    }

    #[test]
    fn normalizes_token_info_from_raw_bundle_shape() {
        let raw = serde_json::json!({
            "access_token": "token",
            "token_expires_in": 3600,
            "user_id": "user",
            "refresh_token": "refresh",
            "auth_server_url": "https://auth.example.test",
            "api_server_url": "https://api.example.test"
        });

        let normalized = normalize_token_info(&raw).expect("token info should normalize");
        assert_eq!(normalized.user_id, "user");
        assert_eq!(normalized.access_token_expires_in, 3600);
    }

    #[test]
    fn normalizes_user_profile_from_raw_bundle_shape() {
        let raw = serde_json::json!({
            "user_id": "user",
            "cc": "KR",
            "birthday": "1990-01-01",
            "login_id": "user@example.test"
        });

        let normalized = normalize_user_profile_info(&raw).expect("user profile should normalize");
        assert_eq!(normalized.user_id, "user");
        assert_eq!(normalized.country_code.as_deref(), Some("KR"));
        assert_eq!(normalized.email.as_deref(), Some("user@example.test"));
        assert_eq!(
            normalized.preferred_user_name.as_deref(),
            Some("user@example.test")
        );
    }
}
