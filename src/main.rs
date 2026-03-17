mod auth;
mod backend;
mod candidates;
mod http;
mod models;
mod network;
mod protocol;
mod session;
mod settings;

use std::{
    collections::BTreeSet,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};
use clap::{CommandFactory, Parser, Subcommand};
use models::{
    CompleteWebLoginCandidateInput, CreateUploadSessionRequestCandidate, DownloadMetadata,
    DownloadUrlsRequestCandidate, FileDownloadInfo, FileUploadInfo, GoodlockDownloadUrlsRequest,
    GoodlockInvitationInfoResponse, GoodlockInvitationRequest, GoodlockRoomDownloadUrlsResponse,
    GoodlockRoomInfoResponse, GoodlockRoomMemberInfoResponse, GoodlockSaveRoomRequest,
    GoodlockWebDownloadCodeRequest, GoodlockWebHeadersInput,
    GoodlockWebIssueAccessTokenRequest, GoodlockWebUploadSessionCompleteFileRequest,
    GoodlockWebUploadSessionCompleteRequest, GoodlockWebUploadSessionFileRequest,
    GoodlockWebUploadSessionRequest, HeadersMetadata, LoginSession, OgUploadInfo,
    SamsungIssueAccessTokenRequestCandidate, SamsungRefreshTokenRequestCandidate,
    SamsungSignInGateRequestCandidate, SamsungTokenInfo, ServerFilesInput, SessionHeadersInput,
    UploadItem, UploadMetadata, WebLoginStateInput,
};

#[derive(Debug, Parser)]
#[command(name = "dropship-cmd")]
#[command(about = "Rust CLI scaffold for Samsung Dropship transfer contracts")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    BuildUploadWorkInput {
        #[arg(long)]
        pin: String,
        #[arg(long)]
        metadata: PathBuf,
    },
    BuildOgWorkInput {
        #[arg(long)]
        pin: String,
        #[arg(long)]
        metadata: PathBuf,
        #[arg(long)]
        og_upload_info: PathBuf,
        #[arg(long)]
        og_local_path: String,
    },
    BuildDownloadWorkInput {
        #[arg(long)]
        pin: String,
        #[arg(long)]
        metadata: PathBuf,
    },
    BuildPushRequest {
        #[arg(long)]
        pin: String,
        #[arg(long)]
        metadata: PathBuf,
        #[arg(long)]
        thumbnail_url: Option<String>,
    },
    BuildUploadReport {
        #[arg(long)]
        upload_item: PathBuf,
        #[arg(long)]
        thumbnail_file_info: Option<PathBuf>,
    },
    BuildOgCompletedItem {
        #[arg(long)]
        og_upload_info: PathBuf,
        #[arg(long)]
        file_info: PathBuf,
    },
    BuildHeaders {
        #[arg(long)]
        headers: PathBuf,
    },
    PickThumbnail {
        #[arg(long)]
        server_files: PathBuf,
    },
    UploadFile {
        #[arg(long)]
        file_info: PathBuf,
    },
    ExecuteUploadItem {
        #[arg(long)]
        pin: String,
        #[arg(long)]
        metadata: PathBuf,
        #[arg(long)]
        upload_item: PathBuf,
        #[arg(long)]
        thumbnail_file_info: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        send_push: bool,
    },
    ExecuteOgUpload {
        #[arg(long)]
        pin: String,
        #[arg(long)]
        metadata: PathBuf,
        #[arg(long)]
        og_upload_info: PathBuf,
        #[arg(long)]
        og_local_path: Option<String>,
    },
    DownloadFile {
        #[arg(long)]
        file_info: PathBuf,
        #[arg(long, default_value_t = true)]
        resume: bool,
    },
    CancelTransfer {
        #[arg(long)]
        metadata: PathBuf,
    },
    FetchThumbnailUrl {
        #[arg(long)]
        pin: String,
        #[arg(long)]
        metadata: PathBuf,
    },
    SendPush {
        #[arg(long)]
        pin: String,
        #[arg(long)]
        metadata: PathBuf,
        #[arg(long)]
        thumbnail_url: Option<String>,
    },
    ShowKnownEndpoints,
    BuildEndpoint {
        #[arg(long)]
        base_url: String,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        suffix: Option<String>,
    },
    ShowCandidateModels,
    ShowCandidateFlows,
    BuildSamsungSignInGateUrl {
        #[arg(long)]
        base_url: String,
        #[arg(long)]
        request: PathBuf,
    },
    BuildSamsungAuthForm {
        #[arg(long)]
        base_url: String,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        request: PathBuf,
    },
    ParseReceivedUrl {
        #[arg(long)]
        url: String,
    },
    ClassifyDropshipLink {
        #[arg(long)]
        url: String,
    },
    EvaluateWebLoginState {
        #[arg(long)]
        input: PathBuf,
    },
    RequestSamsungTokenCandidate {
        #[arg(long)]
        url: String,
        #[arg(long)]
        request: PathBuf,
    },
    RequestSamsungRefreshTokenCandidate {
        #[arg(long)]
        url: String,
        #[arg(long)]
        request: PathBuf,
    },
    RequestGoodlockWebIssueAccessToken {
        #[arg(long)]
        url: String,
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        headers: Option<PathBuf>,
    },
    FetchGoodlockWebUserInfo {
        #[arg(long)]
        url: String,
        #[arg(long)]
        headers: Option<PathBuf>,
    },
    FetchGoodlockWebUser {
        #[arg(long)]
        url: String,
        #[arg(long)]
        headers: Option<PathBuf>,
    },
    CompleteGoodlockWebLoginConfirmed {
        #[arg(long)]
        issue_token_url: String,
        #[arg(long)]
        issue_token_request: PathBuf,
        #[arg(long)]
        user_info_url: String,
        #[arg(long)]
        user_url: Option<String>,
        #[arg(long)]
        headers: Option<PathBuf>,
    },
    Login {
        #[arg(long)]
        callback_url: Option<String>,
        #[arg(long)]
        session_out: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        no_browser: bool,
        #[arg(long, default_value = "boyxkp1y8s")]
        client_id: String,
        #[arg(long, default_value = "https://g2sh.me/accesstoken.html")]
        redirect_uri: String,
        #[arg(long, default_value = "https://account.samsung.com")]
        sign_in_base_url: String,
        #[arg(
            long,
            default_value = "https://api.goodlocklabs.com/dropship-web/v1/user/issueAccessToken"
        )]
        issue_token_url: String,
        #[arg(
            long,
            default_value = "https://api.goodlocklabs.com/dropship-web/v1/user/userInfo"
        )]
        user_info_url: String,
        #[arg(long, default_value = "https://api.goodlocklabs.com/dropship/v1/user")]
        user_url: String,
        #[arg(long, default_value_t = false)]
        stay_signed_in: bool,
    },
    #[command(name = "whoami")]
    WhoAmI {
        #[arg(long)]
        session: Option<PathBuf>,
    },
    SessionCheck {
        #[arg(long)]
        session: Option<PathBuf>,
    },
    Usage {
        #[arg(long)]
        session: Option<PathBuf>,
    },
    UploadHistory {
        #[arg(long)]
        session: Option<PathBuf>,
    },
    JoinedRooms {
        #[arg(long)]
        session: Option<PathBuf>,
    },
    CreateUploadSession {
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        session: Option<PathBuf>,
    },
    CompleteUploadSession {
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        session: Option<PathBuf>,
    },
    ShareFile {
        #[arg(long, required = true)]
        path: Vec<PathBuf>,
        #[arg(long)]
        session: Option<PathBuf>,
        #[arg(long)]
        expiry: Option<u64>,
        #[arg(long)]
        keyword: Option<String>,
        #[arg(long)]
        share_message: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        password: Option<String>,
        #[arg(long, default_value_t = false)]
        use_keyword: bool,
        #[arg(long, default_value_t = false)]
        is_only_me: bool,
        #[arg(long, default_value_t = false)]
        no_profile: bool,
        #[arg(long, default_value_t = false)]
        use_extra_size: bool,
    },
    CreateRoom {
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        session: Option<PathBuf>,
    },
    RoomInfo {
        #[arg(long)]
        room_id: String,
        #[arg(long)]
        session: Option<PathBuf>,
    },
    RoomMembers {
        #[arg(long)]
        room_id: String,
        #[arg(long)]
        session: Option<PathBuf>,
    },
    CreateInvitation {
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        session: Option<PathBuf>,
    },
    RoomDownloadUrls {
        #[arg(long)]
        room_id: String,
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        session: Option<PathBuf>,
    },
    FetchSamsungUserProfileCandidate {
        #[arg(long)]
        url: String,
        #[arg(long)]
        token_info: PathBuf,
    },
    CompleteWebLoginCandidate {
        #[arg(long)]
        input: PathBuf,
    },
    DeriveEndpointsFromToken {
        #[arg(long)]
        token_info: PathBuf,
    },
    ProbeEndpoint {
        #[arg(long)]
        token_info: PathBuf,
        #[arg(long)]
        kind: String,
        #[arg(long)]
        method: String,
        #[arg(long)]
        body: Option<PathBuf>,
        #[arg(long)]
        session_headers: Option<PathBuf>,
        #[arg(long)]
        suffix: Option<String>,
        #[arg(long, default_value = "auto")]
        extract: String,
    },
    RequestCreateUploadSessionCandidate {
        #[arg(long)]
        url: String,
        #[arg(long)]
        headers: PathBuf,
        #[arg(long)]
        request: PathBuf,
    },
    RequestGetDownloadUrlsCandidate {
        #[arg(long)]
        url: String,
        #[arg(long)]
        headers: PathBuf,
        #[arg(long)]
        request: PathBuf,
    },
}

fn main() -> Result<()> {
    if try_handle_root_help()? {
        return Ok(());
    }

    if try_handle_receive_room_command()? {
        return Ok(());
    }

    if try_handle_settings_command()? {
        return Ok(());
    }

    let cli = Cli::parse();

    match cli.command {
        Command::BuildUploadWorkInput { pin, metadata } => {
            protocol::require_non_empty_pin(&pin)?;
            let metadata: UploadMetadata = read_json(metadata)?;
            write_json(&protocol::build_upload_work_input(&pin, &metadata))?;
        }
        Command::BuildOgWorkInput {
            pin,
            metadata,
            og_upload_info,
            og_local_path,
        } => {
            protocol::require_non_empty_pin(&pin)?;
            let metadata: UploadMetadata = read_json(metadata)?;
            let og_upload_info: OgUploadInfo = read_json(og_upload_info)?;
            write_json(&protocol::build_og_work_input(
                &pin,
                &og_local_path,
                &og_upload_info,
                &metadata,
            ))?;
        }
        Command::BuildDownloadWorkInput { pin, metadata } => {
            protocol::require_non_empty_pin(&pin)?;
            let metadata: DownloadMetadata = read_json(metadata)?;
            write_json(&protocol::build_download_work_input(
                &pin,
                &metadata.notification,
            ))?;
        }
        Command::BuildPushRequest {
            pin,
            metadata,
            thumbnail_url,
        } => {
            protocol::require_non_empty_pin(&pin)?;
            let metadata: UploadMetadata = read_json(metadata)?;
            let push = protocol::require_push_message(&metadata)?;
            write_json(&protocol::build_push_request(
                push,
                &pin,
                thumbnail_url.as_deref(),
            ))?;
        }
        Command::BuildUploadReport {
            upload_item,
            thumbnail_file_info,
        } => {
            let upload_item: UploadItem = read_json(upload_item)?;
            let thumbnail_file_info = thumbnail_file_info
                .map(read_json::<FileUploadInfo>)
                .transpose()?;
            write_json(&protocol::build_upload_report(
                &upload_item,
                thumbnail_file_info.as_ref(),
            ))?;
        }
        Command::BuildOgCompletedItem {
            og_upload_info,
            file_info,
        } => {
            let og_upload_info: OgUploadInfo = read_json(og_upload_info)?;
            let file_info: FileUploadInfo = read_json(file_info)?;
            write_json(&protocol::build_og_completed_item(
                &og_upload_info,
                &file_info,
            ))?;
        }
        Command::BuildHeaders { headers } => {
            let headers: HeadersMetadata = read_json(headers)?;
            write_json(&protocol::build_upload_headers(&headers))?;
        }
        Command::PickThumbnail { server_files } => {
            let server_files: ServerFilesInput = read_json(server_files)?;
            let selected = protocol::pick_thumbnail_url(&server_files.into_vec());
            write_json(&serde_json::json!({ "thumbnailUrl": selected }))?;
        }
        Command::UploadFile { file_info } => {
            let file_info: FileUploadInfo = read_json(file_info)?;
            write_json(&network::upload_file(&file_info)?)?;
        }
        Command::ExecuteUploadItem {
            pin,
            metadata,
            upload_item,
            thumbnail_file_info,
            send_push,
        } => {
            protocol::require_non_empty_pin(&pin)?;
            let metadata: UploadMetadata = read_json(metadata)?;
            let upload_item: UploadItem = read_json(upload_item)?;
            let thumbnail_file_info = thumbnail_file_info
                .map(read_json::<FileUploadInfo>)
                .transpose()?;
            write_json(&network::execute_upload_item(
                &pin,
                &metadata,
                &upload_item,
                thumbnail_file_info.as_ref(),
                send_push,
            )?)?;
        }
        Command::ExecuteOgUpload {
            pin,
            metadata,
            og_upload_info,
            og_local_path,
        } => {
            protocol::require_non_empty_pin(&pin)?;
            let metadata: UploadMetadata = read_json(metadata)?;
            let og_upload_info: OgUploadInfo = read_json(og_upload_info)?;
            write_json(&network::execute_og_upload(
                &pin,
                &metadata,
                &og_upload_info,
                og_local_path.as_deref(),
            )?)?;
        }
        Command::DownloadFile { file_info, resume } => {
            let file_info: FileDownloadInfo = read_json(file_info)?;
            write_json(&network::download_file(&file_info, resume)?)?;
        }
        Command::CancelTransfer { metadata } => {
            let metadata: UploadMetadata = read_json(metadata)?;
            write_json(&serde_json::json!({
                "status": network::cancel_transfer(&metadata)?,
            }))?;
        }
        Command::FetchThumbnailUrl { pin, metadata } => {
            protocol::require_non_empty_pin(&pin)?;
            let metadata: UploadMetadata = read_json(metadata)?;
            write_json(&serde_json::json!({
                "thumbnailUrl": network::fetch_thumbnail_url(&pin, &metadata)?,
            }))?;
        }
        Command::SendPush {
            pin,
            metadata,
            thumbnail_url,
        } => {
            protocol::require_non_empty_pin(&pin)?;
            let metadata: UploadMetadata = read_json(metadata)?;
            write_json(&serde_json::json!({
                "status": network::send_push(&pin, &metadata, thumbnail_url.as_deref())?,
            }))?;
        }
        Command::ShowKnownEndpoints => {
            write_json(&backend::known_endpoints())?;
        }
        Command::BuildEndpoint {
            base_url,
            kind,
            suffix,
        } => {
            let kind = backend::EndpointKind::parse(&kind)
                .with_context(|| format!("unknown endpoint kind: {kind}"))?;
            write_json(&serde_json::json!({
                "url": backend::build_endpoint(&base_url, kind, suffix.as_deref())?,
            }))?;
        }
        Command::ShowCandidateModels => {
            write_json(&candidates::candidate_models())?;
        }
        Command::ShowCandidateFlows => {
            write_json(&candidates::candidate_flows())?;
        }
        Command::BuildSamsungSignInGateUrl { base_url, request } => {
            let request: SamsungSignInGateRequestCandidate = read_json(request)?;
            write_json(&auth::build_sign_in_gate_url(&base_url, &request)?)?;
        }
        Command::BuildSamsungAuthForm {
            base_url,
            kind,
            request,
        } => {
            let request: SamsungIssueAccessTokenRequestCandidate = read_json(request)?;
            let kind = backend::EndpointKind::parse(&kind)
                .with_context(|| format!("unknown endpoint kind: {kind}"))?;
            match kind {
                backend::EndpointKind::SamsungIssueAccessToken
                | backend::EndpointKind::SamsungOauthToken => {}
                _ => {
                    anyhow::bail!(
                        "build-samsung-auth-form only supports samsung-issue-access-token or samsung-oauth-token"
                    );
                }
            }
            write_json(&auth::build_auth_form(&base_url, kind, &request)?)?;
        }
        Command::ParseReceivedUrl { url } => {
            write_json(&auth::parse_received_url(&url)?)?;
        }
        Command::ClassifyDropshipLink { url } => {
            write_json(&auth::classify_dropship_link(&url)?)?;
        }
        Command::EvaluateWebLoginState { input } => {
            let input: WebLoginStateInput = read_json(input)?;
            write_json(&auth::evaluate_web_login_state(&input)?)?;
        }
        Command::RequestSamsungTokenCandidate { url, request } => {
            let request: SamsungIssueAccessTokenRequestCandidate = read_json(request)?;
            write_json(&auth::request_samsung_token_candidate(&url, &request)?)?;
        }
        Command::RequestSamsungRefreshTokenCandidate { url, request } => {
            let request: SamsungRefreshTokenRequestCandidate = read_json(request)?;
            write_json(&auth::request_samsung_refresh_token_candidate(
                &url, &request,
            )?)?;
        }
        Command::RequestGoodlockWebIssueAccessToken {
            url,
            request,
            headers,
        } => {
            let request: GoodlockWebIssueAccessTokenRequest = read_json(request)?;
            let headers = headers
                .map(read_json::<GoodlockWebHeadersInput>)
                .transpose()?;
            write_json(&auth::request_goodlock_web_issue_access_token(
                &url,
                &request,
                headers.as_ref(),
            )?)?;
        }
        Command::FetchGoodlockWebUserInfo { url, headers } => {
            let headers = headers
                .map(read_json::<GoodlockWebHeadersInput>)
                .transpose()?;
            write_json(&auth::fetch_goodlock_web_user_info(&url, headers.as_ref())?)?;
        }
        Command::FetchGoodlockWebUser { url, headers } => {
            let headers = headers
                .map(read_json::<GoodlockWebHeadersInput>)
                .transpose()?;
            write_json(&auth::fetch_goodlock_web_user(&url, headers.as_ref())?)?;
        }
        Command::CompleteGoodlockWebLoginConfirmed {
            issue_token_url,
            issue_token_request,
            user_info_url,
            user_url,
            headers,
        } => {
            let issue_token_request: GoodlockWebIssueAccessTokenRequest =
                read_json(issue_token_request)?;
            let headers = headers
                .map(read_json::<GoodlockWebHeadersInput>)
                .transpose()?;
            write_json(&auth::complete_goodlock_web_login_confirmed(
                &issue_token_url,
                &issue_token_request,
                &user_info_url,
                user_url.as_deref(),
                headers.as_ref(),
            )?)?;
        }
        Command::Login {
            callback_url,
            session_out,
            no_browser,
            client_id,
            redirect_uri,
            sign_in_base_url,
            issue_token_url,
            user_info_url,
            user_url,
            stay_signed_in,
        } => {
            let state = generate_login_state();
            let sign_in_request = SamsungSignInGateRequestCandidate {
                client_id,
                redirect_uri: redirect_uri.clone(),
                response_type: "code".to_string(),
                state: Some(state),
                extra: {
                    let mut extra = std::collections::BTreeMap::new();
                    extra.insert("scope".to_string(), "offline.access".to_string());
                    extra
                },
            };
            let sign_in = auth::build_sign_in_gate_url(&sign_in_base_url, &sign_in_request)?;

            eprintln!("Sign-in URL:");
            eprintln!("{}", sign_in.url);
            if !no_browser {
                if let Err(error) = open_browser(&sign_in.url) {
                    eprintln!("Failed to open browser automatically: {error}");
                }
            }

            let callback_url = match callback_url {
                Some(url) => url,
                None => prompt_line(
                    "Paste the full callback URL from https://g2sh.me/accesstoken.html: ",
                )?,
            };

            let callback = auth::parse_received_url(&callback_url)?;
            let code = callback
                .code
                .clone()
                .context("callback URL did not contain a code query parameter")?;
            let headers = GoodlockWebHeadersInput {
                authorization: None,
                cookie: None,
                ..default_goodlock_web_headers()
            };
            let issue_request = GoodlockWebIssueAccessTokenRequest {
                code,
                redirect_url: redirect_uri,
                stay_signed_in,
            };
            let token = auth::request_goodlock_web_issue_access_token(
                &issue_token_url,
                &issue_request,
                Some(&headers),
            )?;

            let bearer_headers = GoodlockWebHeadersInput {
                authorization: Some(format!("Bearer {}", token.body.access_token)),
                ..headers
            };
            let user_info =
                auth::fetch_goodlock_web_user_info(&user_info_url, Some(&bearer_headers))?;
            let user = auth::fetch_goodlock_web_user(&user_url, Some(&bearer_headers)).ok();

            let session = auth::build_login_session(
                sign_in.url,
                callback,
                token.body,
                user_info.body,
                user.map(|item| item.body),
            )?;
            let session_path =
                session_out.unwrap_or_else(|| PathBuf::from("dropship-session.json"));
            write_json_file(&session_path, &session)?;
            write_json(&serde_json::json!({
                "sessionPath": session_path,
                "loginState": "loggedIn",
                "userId": session.samsung_token_info.user_id,
                "authServerUrl": session.samsung_token_info.auth_server_url,
                "apiServerUrl": session.samsung_token_info.api_server_url,
            }))?;
        }
        Command::WhoAmI { session } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let output = with_session_retry(&session_path, |session, headers| {
                let user = auth::fetch_goodlock_web_user(
                    "https://api.goodlocklabs.com/dropship/v1/user",
                    Some(headers),
                )?;
                let (user_info, user_info_source) =
                    match auth::fetch_goodlock_web_user_info(
                        "https://api.goodlocklabs.com/dropship-web/v1/user/userInfo",
                        Some(headers),
                    ) {
                        Ok(value) => (value.body, "live"),
                        Err(_) => (session.user_info.clone(), "session"),
                    };
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "sessionUserId": session.samsung_token_info.user_id,
                    "userInfo": user_info,
                    "userInfoSource": user_info_source,
                    "user": user.body,
                }))
            })?;
            write_json(&output)?;
        }
        Command::SessionCheck { session } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let output = with_session_retry(&session_path, |session, headers| {
                let user = auth::fetch_goodlock_web_user(
                    "https://api.goodlocklabs.com/dropship/v1/user",
                    Some(headers),
                )?;
                let (user_info_status, user_info_error) =
                    match auth::fetch_goodlock_web_user_info(
                        "https://api.goodlocklabs.com/dropship-web/v1/user/userInfo",
                        Some(headers),
                    ) {
                        Ok(value) => (Some(value.status), None),
                        Err(error) => (None, Some(error.to_string())),
                    };
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "valid": true,
                    "userId": session.samsung_token_info.user_id,
                    "userInfoStatus": user_info_status,
                    "userInfoError": user_info_error,
                    "userStatus": user.status,
                    "accessTokenExpiresIn": session.samsung_token_info.access_token_expires_in,
                    "refreshTokenExpiresIn": session.samsung_token_info.refresh_token_expires_in,
                }))
            })?;
            write_json(&output)?;
        }
        Command::Usage { session } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let output = with_session_retry(&session_path, |_, headers| {
                let usage = auth::fetch_goodlock_web_usage(
                    "https://api.goodlocklabs.com/dropship/v1/user/usage",
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "usage": usage.body,
                    "status": usage.status,
                }))
            })?;
            write_json(&output)?;
        }
        Command::UploadHistory { session } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let output = with_session_retry(&session_path, |_, headers| {
                let history = auth::fetch_goodlock_web_upload_history(
                    "https://api.goodlocklabs.com/dropship/v1/history/upload",
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "count": history.body.len(),
                    "items": history.body,
                    "status": history.status,
                }))
            })?;
            write_json(&output)?;
        }
        Command::JoinedRooms { session } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let output = with_session_retry(&session_path, |_, headers| {
                let rooms = auth::fetch_goodlock_web_joined_rooms(
                    "https://api.goodlocklabs.com/dropship/v1/room/info/joined",
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "count": rooms.body.len(),
                    "items": rooms.body,
                    "status": rooms.status,
                }))
            })?;
            write_json(&output)?;
        }
        Command::CreateUploadSession { request, session } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let request: GoodlockWebUploadSessionRequest = read_json(request)?;
            let output = with_session_retry(&session_path, |_, headers| {
                let response = auth::create_goodlock_upload_session(
                    "https://api.goodlocklabs.com/dropship/v1/uploadSession",
                    &request,
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "status": response.status,
                    "shareUrlCandidates": share_url_candidates(&response.body.code),
                    "uploadSession": response.body,
                }))
            })?;
            write_json(&output)?;
        }
        Command::CompleteUploadSession { request, session } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let request: GoodlockWebUploadSessionCompleteRequest = read_json(request)?;
            let output = with_session_retry(&session_path, |_, headers| {
                let response = auth::complete_goodlock_upload_session(
                    "https://api.goodlocklabs.com/dropship/v1/uploadSession/complete",
                    &request,
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "status": response.status,
                    "rawJson": response.raw_json,
                    "rawText": response.raw_text,
                }))
            })?;
            write_json(&output)?;
        }
        Command::ShareFile {
            path,
            session,
            expiry,
            keyword,
            share_message,
            description,
            password,
            use_keyword,
            is_only_me,
            no_profile,
            use_extra_size,
        } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let stored_settings =
                settings::load_settings_optional(&settings::default_settings_path())?;
            let output = with_session_retry(&session_path, |session, headers| {
                let description_map = description
                    .as_ref()
                    .map(|value| {
                        let mut extra = std::collections::BTreeMap::new();
                        extra.insert("description".to_string(), value.clone());
                        extra
                    });
                let file_info_list = path
                    .iter()
                    .map(|path| build_upload_session_file_request(path, description_map.clone()))
                    .collect::<Result<Vec<_>>>()?;
                let session_user = session.user.as_ref();
                let settings_local = stored_settings.as_ref().map(|value| &value.local);
                let request = GoodlockWebUploadSessionRequest {
                    expiry: match expiry {
                        Some(hours) => hours,
                        None => match stored_settings.as_ref() {
                            Some(value) => settings::resolve_expiry_hours(value)?,
                            None => 24,
                        },
                    },
                    password: password.clone(),
                    keyword: keyword
                        .clone()
                        .or_else(|| session_user.and_then(|user| user.keyword.clone())),
                    nick_name: session_user
                        .and_then(|user| user.nick_name.clone())
                        .or_else(|| session.user_info.nick_name.clone()),
                    share_message: share_message
                        .clone()
                        .or_else(|| description.clone())
                        .or_else(|| Some("shared from dropship-cmd".to_string())),
                    use_keyword: if use_keyword {
                        true
                    } else {
                        settings_local
                            .map(|value| value.use_keyword)
                            .or_else(|| session_user.and_then(|user| user.use_keyword))
                            .unwrap_or(false)
                    },
                    is_internal_user: session_user
                        .and_then(|user| user.is_internal_user)
                        .unwrap_or(false),
                    is_only_me,
                    use_profile: if no_profile {
                        false
                    } else {
                        session_user
                            .and_then(|user| user.use_profile)
                            .unwrap_or(true)
                    },
                    use_extra_size: if use_extra_size {
                        true
                    } else {
                        session_user
                            .and_then(|user| user.use_extra_size)
                            .unwrap_or(false)
                    },
                    profile_url: if no_profile {
                        None
                    } else {
                        session_user.and_then(|user| user.thumbnail_url.clone())
                    },
                    is_invitation: false,
                    room_id: None,
                    description: description.clone(),
                    file_info_list,
                };
                let upload_session = auth::create_goodlock_upload_session(
                    "https://api.goodlocklabs.com/dropship/v1/uploadSession",
                    &request,
                    Some(headers),
                )?;
                if upload_session.body.file_info_list.len() != request.file_info_list.len() {
                    bail!(
                        "uploadSession returned {} file entries for {} requested files",
                        upload_session.body.file_info_list.len(),
                        request.file_info_list.len()
                    );
                }

                let mut uploads = Vec::with_capacity(request.file_info_list.len());
                let mut complete_files = Vec::with_capacity(request.file_info_list.len());
                for (request_file, response_file) in request
                    .file_info_list
                    .iter()
                    .zip(upload_session.body.file_info_list.iter())
                {
                    uploads.push(network::upload_file(&FileUploadInfo {
                        local_path: request_file.file_path.clone(),
                        upload_url: response_file.upload_url.clone(),
                        name: request_file.file_name.clone(),
                        file_type: request_file.file_type.clone(),
                        size: request_file.file_size,
                    })?);
                    complete_files.push(GoodlockWebUploadSessionCompleteFileRequest {
                        file_name: request_file.file_name.clone(),
                        file_type: request_file.file_type.clone(),
                        file_size: request_file.file_size,
                        object_id: response_file.object_id.clone(),
                        t_object_id: None,
                        is_og_image: false,
                        extra: request_file.extra.clone(),
                    });
                }

                let complete = auth::complete_goodlock_upload_session(
                    "https://api.goodlocklabs.com/dropship/v1/uploadSession/complete",
                    &GoodlockWebUploadSessionCompleteRequest {
                        code: upload_session.body.code.clone(),
                        file_info_list: complete_files,
                    },
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "code": upload_session.body.code,
                    "shareUrlCandidates": share_url_candidates(&upload_session.body.code),
                    "expiry": upload_session.body.expiry,
                    "uploadSessionStatus": upload_session.status,
                    "completeStatus": complete.status,
                    "fileCount": request.file_info_list.len(),
                    "objectIds": upload_session.body.file_info_list.iter().map(|item| item.object_id.clone()).collect::<Vec<_>>(),
                    "uploads": uploads.iter().map(|item| serde_json::json!({
                        "localPath": item.local_path,
                        "size": item.size,
                        "status": item.status,
                    })).collect::<Vec<_>>(),
                    "completeRawJson": complete.raw_json,
                    "completeRawText": complete.raw_text,
                }))
            })?;
            write_json(&output)?;
        }
        Command::CreateRoom { request, session } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let request: GoodlockSaveRoomRequest = read_json(request)?;
            let output = with_session_retry(&session_path, |_, headers| {
                let room = auth::create_goodlock_room(
                    "https://api.goodlocklabs.com/dropship/v1/room/info",
                    &request,
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "status": room.status,
                    "summary": summarize_room(&room.body),
                    "room": room.body,
                }))
            })?;
            write_json(&output)?;
        }
        Command::RoomInfo { room_id, session } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let output = with_session_retry(&session_path, |_, headers| {
                match auth::fetch_goodlock_room_info(
                    "https://api.goodlocklabs.com/dropship/v1/room/info",
                    &room_id,
                    Some(headers),
                ) {
                    Ok(room) => Ok(serde_json::json!({
                        "sessionPath": session_path,
                        "status": room.status,
                        "summary": summarize_room(&room.body),
                        "room": room.body,
                    })),
                    Err(error) if room_invitation_missing(&error) => Ok(serde_json::json!({
                        "sessionPath": session_path,
                        "status": 404,
                        "roomId": room_id,
                        "invitationActive": false,
                        "message": "room exists but no active invitation is attached; run room-invite first",
                    })),
                    Err(error) => Err(error),
                }
            })?;
            write_json(&output)?;
        }
        Command::RoomMembers { room_id, session } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let output = with_session_retry(&session_path, |_, headers| {
                let members = auth::fetch_goodlock_room_members(
                    &format!("https://api.goodlocklabs.com/dropship/v1/room/{room_id}/members"),
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "status": members.status,
                    "count": members.body.len(),
                    "summary": summarize_room_members(&room_id, &members.body),
                    "members": members.body,
                }))
            })?;
            write_json(&output)?;
        }
        Command::CreateInvitation { request, session } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let request: GoodlockInvitationRequest = read_json(request)?;
            let output = with_session_retry(&session_path, |_, headers| {
                let invitation = auth::create_goodlock_invitation(
                    "https://api.goodlocklabs.com/dropship/v1/room/invitation",
                    &request,
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "status": invitation.status,
                    "summary": summarize_invitation(&invitation.body),
                    "invitation": invitation.body,
                }))
            })?;
            write_json(&output)?;
        }
        Command::RoomDownloadUrls {
            room_id,
            request,
            session,
        } => {
            let session_path = session.clone().unwrap_or_else(default_session_path);
            let request: GoodlockDownloadUrlsRequest = read_json(request)?;
            let output = with_session_retry(&session_path, |_, headers| {
                let response = auth::request_goodlock_room_download_urls(
                    &format!("https://api.goodlocklabs.com/dropship/v1/room/{room_id}/downloadUrls"),
                    &request,
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "status": response.status,
                    "count": response.body.len(),
                    "summary": summarize_room_download_urls(&room_id, &response.body),
                    "items": response.body,
                }))
            })?;
            write_json(&output)?;
        }
        Command::FetchSamsungUserProfileCandidate { url, token_info } => {
            let token_info: SamsungTokenInfo = read_json(token_info)?;
            write_json(&auth::fetch_samsung_user_profile_candidate(
                &url,
                &token_info,
            )?)?;
        }
        Command::CompleteWebLoginCandidate { input } => {
            let input: CompleteWebLoginCandidateInput = read_json(input)?;
            write_json(&auth::complete_web_login_candidate(&input)?)?;
        }
        Command::DeriveEndpointsFromToken { token_info } => {
            let token_info: SamsungTokenInfo = read_json(token_info)?;
            write_json(&backend::derive_endpoints_from_token(&token_info)?)?;
        }
        Command::ProbeEndpoint {
            token_info,
            kind,
            method,
            body,
            session_headers,
            suffix,
            extract,
        } => {
            let token_info: SamsungTokenInfo = read_json(token_info)?;
            let kind = backend::EndpointKind::parse(&kind)
                .with_context(|| format!("unknown endpoint kind: {kind}"))?;
            let extract = session::ExtractKind::parse(&extract)
                .with_context(|| format!("unknown extract kind: {extract}"))?;
            let body = body.map(read_json::<serde_json::Value>).transpose()?;
            let session_headers = session_headers
                .map(read_json::<SessionHeadersInput>)
                .transpose()?;
            write_json(&session::probe_endpoint(
                &token_info,
                kind,
                &method,
                suffix.as_deref(),
                body.as_ref(),
                session_headers.as_ref(),
                extract,
            )?)?;
        }
        Command::RequestCreateUploadSessionCandidate {
            url,
            headers,
            request,
        } => {
            let headers: HeadersMetadata = read_json(headers)?;
            let request: CreateUploadSessionRequestCandidate = read_json(request)?;
            write_json(&session::create_upload_session_candidate(
                &url, &headers, &request,
            )?)?;
        }
        Command::RequestGetDownloadUrlsCandidate {
            url,
            headers,
            request,
        } => {
            let headers: HeadersMetadata = read_json(headers)?;
            let request: DownloadUrlsRequestCandidate = read_json(request)?;
            write_json(&session::get_download_urls_candidate(
                &url, &headers, &request,
            )?)?;
        }
    }

    Ok(())
}

fn try_handle_root_help() -> Result<bool> {
    let args = std::env::args().collect::<Vec<_>>();
    match args.as_slice() {
        [_, flag] if flag == "--help" || flag == "-h" || flag == "help" => {
            print_root_help()?;
            Ok(true)
        }
        [_, flag] if flag == "--advanced-help" => {
            print_advanced_help()?;
            Ok(true)
        }
        [_, flag, topic] if flag == "help" && topic == "advanced" => {
            print_advanced_help()?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn print_root_help() -> Result<()> {
    println!("dropship-cmd");
    println!("User-facing CLI for Samsung Dropship web flows.");
    println!();
    println!("Daily Commands:");
    println!("  login                     Sign in and create dropship-session.json");
    println!("  whoami                    Show current account and device information");
    println!("  session-check             Verify the current session");
    println!("  refresh-session           Refresh dropship-session.json");
    println!("  share-file                Upload local files and create a public share");
    println!("  receive-info              Inspect a share code or URL");
    println!("  receive                   Download files from a share code or URL");
    println!("  joined-rooms              List rooms joined by the current account");
    println!();
    println!("Share Settings:");
    println!("  settings-init             Create dropship-settings.json");
    println!("  settings-show             Show local share settings");
    println!("  settings-set              Update one local share setting");
    println!();
    println!("Room Commands:");
    println!("  create-room               Create a new room");
    println!("  room-info                 Show room info or invitation state");
    println!("  room-members              Show room member list");
    println!("  room-invite               Create an invitation code for a room");
    println!("  delete-room-invitation    Delete the active invitation");
    println!("  room-download-urls        Resolve room invitation codes");
    println!("  room-codes                Show recent room code list");
    println!("  join-room                 Join a room by roomId");
    println!();
    println!("Default Paths:");
    println!("  session file              dropship-session.json");
    println!("  settings file             dropship-settings.json");
    println!("  receive output            received/<code>");
    println!();
    println!("More:");
    println!("  dropship-cmd --advanced-help");
    println!("  COMMANDS.md");
    Ok(())
}

fn print_advanced_help() -> Result<()> {
    let mut command = Cli::command();
    command
        .print_help()
        .context("failed to print clap help")?;
    println!();
    println!();
    println!("Manual Top-Level Commands:");
    println!("  receive-info");
    println!("  receive");
    println!("  refresh-session");
    println!("  create-room");
    println!("  save-room");
    println!("  room-invite");
    println!("  create-invitation");
    println!("  delete-room-invitation");
    println!("  join-room");
    println!("  room-codes");
    println!("  settings-show");
    println!("  settings-init");
    println!("  settings-set");
    Ok(())
}

fn try_handle_receive_room_command() -> Result<bool> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        return Ok(false);
    }

    match args[1].as_str() {
        "receive-info" => {
            if settings_help_requested(&args[2..]) {
                print_receive_info_help();
                return Ok(true);
            }
            let options = parse_long_options(&args[2..])?;
            let source = option_value(&options, "--code")
                .context("receive-info requires --code <CODE_OR_URL>")?;
            let resolved_code = normalize_receive_code(&source)?;
            let headers = option_path(&options, "--session")
                .map(load_headers_from_session_path)
                .transpose()?;
            let outcome = fetch_receive_download_info(
                &resolved_code,
                option_value(&options, "--password"),
                option_value(&options, "--access-type")
                    .unwrap_or_else(|| "VALIDATE".to_string()),
                headers.as_ref(),
                false,
            )?;
            let file_count = outcome
                .response
                .body
                .file_list
                .as_ref()
                .map(|items| items.len())
                .unwrap_or(0);
            write_json(&serde_json::json!({
                "status": outcome.response.status,
                "source": source,
                "resolvedCode": resolved_code,
                "suggestedOutputDir": default_receive_output_dir(&resolved_code),
                "fileCount": file_count,
                "hasFiles": file_count > 0,
                "requiresPassword": outcome.response.body.is_secure,
                "passwordProvided": !outcome.request.password.is_empty(),
                "isOnlyMe": outcome.response.body.is_only_me,
                "nextAction": if outcome.response.body.is_secure && outcome.request.password.is_empty() {
                    Some("rerun receive with --password or use receive to get an interactive password prompt")
                } else {
                    None
                },
                "request": outcome.request,
                "download": outcome.response.body,
                "rawJson": outcome.response.raw_json,
            }))?;
            Ok(true)
        }
        "receive" => {
            if settings_help_requested(&args[2..]) {
                print_receive_help();
                return Ok(true);
            }
            let options = parse_long_options(&args[2..])?;
            let source = option_value(&options, "--code")
                .context("receive requires --code <CODE_OR_URL>")?;
            let resolved_code = normalize_receive_code(&source)?;
            let headers = option_path(&options, "--session")
                .map(load_headers_from_session_path)
                .transpose()?;
            let outcome = fetch_receive_download_info(
                &resolved_code,
                option_value(&options, "--password"),
                option_value(&options, "--access-type")
                    .unwrap_or_else(|| "VALIDATE".to_string()),
                headers.as_ref(),
                true,
            )?;
            let output_dir = option_path(&options, "--output-dir")
                .unwrap_or_else(|| default_receive_output_dir(&resolved_code));
            let resume = option_value(&options, "--resume")
                .map(|value| parse_cli_bool(&value))
                .transpose()?
                .unwrap_or(true);
            let overwrite = option_value(&options, "--overwrite")
                .map(|value| parse_cli_bool(&value))
                .transpose()?
                .unwrap_or(false);
            let file_list = outcome.response.body.file_list.clone().unwrap_or_default();
            if file_list.is_empty() {
                if outcome.response.body.is_secure && outcome.request.password.is_empty() {
                    bail!("share is password protected; rerun with --password <TEXT>");
                }
                bail!("downloadUrls did not contain any downloadable files");
            }

            let mut used_paths = BTreeSet::new();
            let mut downloads = Vec::with_capacity(file_list.len());
            let mut skipped = Vec::new();
            let mut plan = Vec::with_capacity(file_list.len());
            let mut conflicts_resolved = 0usize;
            for file in &file_list {
                let planned = plan_receive_target(
                    &output_dir,
                    &safe_download_file_name(&file.file_name, &file.object_id),
                    &file.object_id,
                    file.file_size,
                    resume,
                    overwrite,
                    &mut used_paths,
                )?;
                if planned.conflict_resolved {
                    conflicts_resolved += 1;
                }
                plan.push(serde_json::json!({
                    "fileName": file.file_name,
                    "targetPath": planned.path,
                    "action": planned.action,
                    "conflictResolved": planned.conflict_resolved,
                    "reason": planned.reason,
                }));
                if planned.skip_existing {
                    skipped.push(serde_json::json!({
                        "fileName": file.file_name,
                        "targetPath": planned.path,
                        "size": file.file_size,
                        "reason": planned.reason,
                    }));
                    continue;
                }
                let file_info = FileDownloadInfo {
                    file_id: file.object_id.clone(),
                    local_path: planned.path.to_string_lossy().into_owned(),
                    download_url: file.file_url.clone(),
                    name: file.file_name.clone(),
                    size: file.file_size,
                };
                downloads.push(network::download_file(&file_info, resume)?);
            }

            write_json(&serde_json::json!({
                "status": outcome.response.status,
                "source": source,
                "resolvedCode": resolved_code,
                "outputDir": output_dir,
                "requiresPassword": outcome.response.body.is_secure,
                "passwordProvided": !outcome.request.password.is_empty(),
                "promptedForPassword": outcome.prompted_for_password,
                "overwrite": overwrite,
                "resume": resume,
                "fileCount": file_list.len(),
                "downloadedCount": downloads.len(),
                "skippedCount": skipped.len(),
                "conflictsResolved": conflicts_resolved,
                "plan": plan,
                "downloadInfo": outcome.response.body,
                "downloads": downloads,
                "skipped": skipped,
            }))?;
            Ok(true)
        }
        "save-room" | "create-room" => {
            if settings_help_requested(&args[2..]) {
                print_save_room_help();
                return Ok(true);
            }
            let options = parse_long_options(&args[2..])?;
            let session_path = option_path(&options, "--session").unwrap_or_else(default_session_path);
            let request = match option_path(&options, "--request") {
                Some(request_path) => read_json(request_path)?,
                None => GoodlockSaveRoomRequest {
                    room_id: option_value(&options, "--room-id").unwrap_or_default(),
                    title: option_value(&options, "--title")
                        .context("create-room requires --title <TEXT> when --request is omitted")?,
                    expiry: option_value(&options, "--expiry")
                        .map(|value| value.parse::<u64>().with_context(|| format!("invalid u64 value: {value}")))
                        .transpose()?
                        .unwrap_or(24),
                    is_secure: option_value(&options, "--is-secure")
                        .map(|value| parse_cli_bool(&value))
                        .transpose()?
                        .unwrap_or_else(|| option_value(&options, "--password").is_some()),
                    password: option_value(&options, "--password"),
                    background_blur_hash: option_value(&options, "--background-blur-hash"),
                },
            };
            let output = with_session_retry(&session_path, |_, headers| {
                let room = auth::create_goodlock_room(
                    "https://api.goodlocklabs.com/dropship/v1/room/info",
                    &request,
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "status": room.status,
                    "summary": summarize_room(&room.body),
                    "room": room.body,
                }))
            })?;
            write_json(&output)?;
            Ok(true)
        }
        "room-invite" | "create-invitation" => {
            if settings_help_requested(&args[2..]) {
                print_room_invite_help();
                return Ok(true);
            }
            let options = parse_long_options(&args[2..])?;
            let session_path = option_path(&options, "--session").unwrap_or_else(default_session_path);
            let request = match option_path(&options, "--request") {
                Some(request_path) => read_json(request_path)?,
                None => GoodlockInvitationRequest {
                    room_id: option_value(&options, "--room-id")
                        .context("room-invite requires --room-id <ROOM_ID> when --request is omitted")?,
                    expiry: option_value(&options, "--expiry")
                        .map(|value| value.parse::<u64>().with_context(|| format!("invalid u64 value: {value}")))
                        .transpose()?
                        .unwrap_or(24),
                    keyword: option_value(&options, "--keyword"),
                },
            };
            let output = with_session_retry(&session_path, |_, headers| {
                let invitation = auth::create_goodlock_invitation(
                    "https://api.goodlocklabs.com/dropship/v1/room/invitation",
                    &request,
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "status": invitation.status,
                    "summary": summarize_invitation(&invitation.body),
                    "invitation": invitation.body,
                }))
            })?;
            write_json(&output)?;
            Ok(true)
        }
        "room-invitation-info" => {
            bail!("room-invitation-info is not supported; the web endpoint is DELETE-only. Use delete-room-invitation instead.")
        }
        "delete-room-invitation" => {
            if settings_help_requested(&args[2..]) {
                print_delete_room_invitation_help();
                return Ok(true);
            }
            let options = parse_long_options(&args[2..])?;
            let room_id = option_value(&options, "--room-id")
                .context("delete-room-invitation requires --room-id <ROOM_ID>")?;
            let session_path = option_path(&options, "--session").unwrap_or_else(default_session_path);
            let output = with_session_retry(&session_path, |_, headers| {
                match auth::delete_goodlock_room_invitation(
                    &format!("https://api.goodlocklabs.com/dropship/v1/room/{room_id}/invitation"),
                    Some(headers),
                ) {
                    Ok(response) => Ok(serde_json::json!({
                        "sessionPath": session_path,
                        "status": response.status,
                        "roomId": room_id,
                        "deleted": true,
                        "rawJson": response.raw_json,
                        "rawText": response.raw_text,
                    })),
                    Err(error) if delete_room_invitation_missing(&error) => Ok(serde_json::json!({
                        "sessionPath": session_path,
                        "status": 404,
                        "roomId": room_id,
                        "deleted": false,
                        "invitationActive": false,
                        "message": "room has no active invitation to delete",
                    })),
                    Err(error) => Err(error),
                }
            })?;
            write_json(&output)?;
            Ok(true)
        }
        "join-room" => {
            if settings_help_requested(&args[2..]) {
                print_join_room_help();
                return Ok(true);
            }
            let options = parse_long_options(&args[2..])?;
            let room_id = option_value(&options, "--room-id")
                .context("join-room requires --room-id <ROOM_ID>")?;
            let password = option_value(&options, "--password").unwrap_or_default();
            let session_path = option_path(&options, "--session").unwrap_or_else(default_session_path);
            let output = with_session_retry(&session_path, |_, headers| {
                let joined = auth::join_goodlock_room(
                    "https://api.goodlocklabs.com/dropship/v1/room/member",
                    &room_id,
                    &password,
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "status": joined.status,
                    "roomId": room_id,
                    "joined": true,
                    "rawJson": joined.raw_json,
                }))
            })?;
            write_json(&output)?;
            Ok(true)
        }
        "room-codes" => {
            if settings_help_requested(&args[2..]) {
                print_room_codes_help();
                return Ok(true);
            }
            let options = parse_long_options(&args[2..])?;
            let room_id = option_value(&options, "--room-id")
                .context("room-codes requires --room-id <ROOM_ID>")?;
            let size = option_value(&options, "--size")
                .map(|value| value.parse::<u64>().with_context(|| format!("invalid u64 value: {value}")))
                .transpose()?
                .unwrap_or(10);
            let session_path = option_path(&options, "--session").unwrap_or_else(default_session_path);
            let output = with_session_retry(&session_path, |_, headers| {
                let codes = auth::fetch_goodlock_room_codes(
                    &format!("https://api.goodlocklabs.com/dropship/v1/room/{room_id}/codes"),
                    size,
                    Some(headers),
                )?;
                Ok(serde_json::json!({
                    "sessionPath": session_path,
                    "status": codes.status,
                    "roomId": room_id,
                    "size": size,
                    "summary": summarize_room_codes(&room_id, &codes.body),
                    "codes": codes.body,
                }))
            })?;
            write_json(&output)?;
            Ok(true)
        }
        "refresh-session" => {
            if settings_help_requested(&args[2..]) {
                print_refresh_session_help();
                return Ok(true);
            }
            let options = parse_long_options(&args[2..])?;
            let session_path = option_path(&options, "--session").unwrap_or_else(default_session_path);
            let session = refresh_login_session_file(&session_path)?;
            write_json(&serde_json::json!({
                "sessionPath": session_path,
                "loginState": "loggedIn",
                "userId": session.samsung_token_info.user_id,
                "accessTokenExpiresIn": session.samsung_token_info.access_token_expires_in,
                "refreshTokenExpiresIn": session.samsung_token_info.refresh_token_expires_in,
                "authServerUrl": session.samsung_token_info.auth_server_url,
                "apiServerUrl": session.samsung_token_info.api_server_url,
            }))?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn read_json<T>(path: PathBuf) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let bytes =
        fs::read(&path).with_context(|| format!("failed to read JSON file {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse JSON file {}", path.display()))
}

fn write_json<T>(value: &T) -> Result<()>
where
    T: serde::Serialize,
{
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer_pretty(&mut handle, value).context("failed to serialize JSON")?;
    handle.write_all(b"\n").context("failed to write newline")?;
    Ok(())
}

fn write_json_file<T>(path: &PathBuf, value: &T) -> Result<()>
where
    T: serde::Serialize,
{
    let bytes = serde_json::to_vec_pretty(value).context("failed to serialize JSON file body")?;
    fs::write(path, bytes)
        .with_context(|| format!("failed to write JSON file {}", path.display()))?;
    Ok(())
}

fn prompt_line(prompt: &str) -> Result<String> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle
        .write_all(prompt.as_bytes())
        .context("failed to write prompt")?;
    handle.flush().context("failed to flush prompt")?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .context("failed to read user input")?;
    Ok(line.trim().to_string())
}

fn generate_login_state() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

fn open_browser(url: &str) -> Result<()> {
    if cfg!(target_os = "windows") {
        ProcessCommand::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("Start-Process '{}'", url.replace('\'', "''")),
            ])
            .status()
            .with_context(|| format!("failed to start browser for {url}"))?;
    } else if cfg!(target_os = "macos") {
        ProcessCommand::new("open")
            .arg(url)
            .status()
            .with_context(|| format!("failed to start browser for {url}"))?;
    } else {
        ProcessCommand::new("xdg-open")
            .arg(url)
            .status()
            .with_context(|| format!("failed to start browser for {url}"))?;
    }
    Ok(())
}

fn default_goodlock_web_headers() -> GoodlockWebHeadersInput {
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

fn default_session_path() -> PathBuf {
    PathBuf::from("dropship-session.json")
}

fn goodlock_headers_from_session(session: &LoginSession) -> Result<GoodlockWebHeadersInput> {
    Ok(GoodlockWebHeadersInput {
        authorization: Some(format!(
            "Bearer {}",
            session.samsung_token_info.access_token
        )),
        auth_url: auth_host_from_url(&session.samsung_token_info.auth_server_url)?,
        ..default_goodlock_web_headers()
    })
}

fn load_headers_from_session_path(path: PathBuf) -> Result<GoodlockWebHeadersInput> {
    let session = load_login_session(&path)?;
    goodlock_headers_from_session(&session)
}

fn load_login_session(path: &PathBuf) -> Result<LoginSession> {
    read_json::<LoginSession>(path.clone())
}

fn with_session_retry<T, F>(session_path: &PathBuf, mut operation: F) -> Result<T>
where
    F: FnMut(&LoginSession, &GoodlockWebHeadersInput) -> Result<T>,
{
    let session = load_login_session(session_path)?;
    let headers = goodlock_headers_from_session(&session)?;
    match operation(&session, &headers) {
        Ok(value) => Ok(value),
        Err(error) if session_needs_refresh(&error) => {
            let refreshed = refresh_login_session_file(session_path)?;
            let refreshed_headers = goodlock_headers_from_session(&refreshed)?;
            operation(&refreshed, &refreshed_headers)
        }
        Err(error) => Err(error),
    }
}

fn refresh_login_session_file(path: &PathBuf) -> Result<LoginSession> {
    let session = read_json::<LoginSession>(path.clone())?;
    let refreshed = refresh_login_session(&session)?;
    write_json_file(path, &refreshed)?;
    Ok(refreshed)
}

fn refresh_login_session(session: &LoginSession) -> Result<LoginSession> {
    if session.goodlock_token.refresh_token.is_empty() {
        bail!("session did not contain a refresh token");
    }

    let auth_url = auth_host_from_url(&session.samsung_token_info.auth_server_url)?;
    let token = auth::request_goodlock_web_refresh_token(
        "https://api.goodlocklabs.com/dropship-web/v1/user/refreshToken",
        &session.goodlock_token.refresh_token,
        &auth_url,
    )
    .context("refresh-session failed; run `cargo run -- login` if the stored refresh token is no longer accepted")?;
    let bearer_headers = GoodlockWebHeadersInput {
        authorization: Some(format!("Bearer {}", token.body.access_token)),
        auth_url,
        ..default_goodlock_web_headers()
    };
    let user_info = auth::fetch_goodlock_web_user_info(
        "https://api.goodlocklabs.com/dropship-web/v1/user/userInfo",
        Some(&bearer_headers),
    )?;
    let user = auth::fetch_goodlock_web_user(
        "https://api.goodlocklabs.com/dropship/v1/user",
        Some(&bearer_headers),
    )
    .ok()
    .map(|item| item.body);

    auth::rebuild_login_session(session, token.body, user_info.body, user)
}

fn session_needs_refresh(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.contains("401 Unauthorized")
        || message.contains("\"UNAUTHENTICATED\"")
        || message.contains("unauthenticated")
}

fn room_invitation_missing(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.contains("NOT_FOUND_INVITATION")
}

fn delete_room_invitation_missing(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    (message.contains("getInvitation()") || message.contains("Invitation.getCode()"))
        && message.contains("is null")
}

fn summarize_room(room: &GoodlockRoomInfoResponse) -> serde_json::Value {
    serde_json::json!({
        "roomId": room.room_id,
        "title": room.title,
        "expiryHours": room.expiry,
        "isSecure": room.is_secure,
        "invitationActive": room.invitation.is_some(),
        "invitationCode": room.invitation.as_ref().map(|item| item.code.clone()),
        "updatedAt": room.updated_at,
    })
}

fn summarize_invitation(invitation: &GoodlockInvitationInfoResponse) -> serde_json::Value {
    serde_json::json!({
        "roomId": invitation.room_id,
        "code": invitation.code,
        "shareUrlCandidates": share_url_candidates(&invitation.code),
        "expiry": invitation.expiry,
    })
}

fn summarize_room_members(
    room_id: &str,
    members: &[GoodlockRoomMemberInfoResponse],
) -> serde_json::Value {
    serde_json::json!({
        "roomId": room_id,
        "memberCount": members.len(),
        "nickNames": members.iter().map(|item| item.nick_name.clone()).collect::<Vec<_>>(),
    })
}

fn summarize_room_codes(room_id: &str, codes: &serde_json::Value) -> serde_json::Value {
    let count = codes
        .get("roomCodeInfoList")
        .and_then(|value| value.as_array())
        .map(|items| items.len())
        .unwrap_or(0);
    serde_json::json!({
        "roomId": room_id,
        "codeCount": count,
        "lastTime": codes.get("lastTime").cloned(),
    })
}

fn summarize_room_download_urls(
    room_id: &str,
    items: &[GoodlockRoomDownloadUrlsResponse],
) -> serde_json::Value {
    serde_json::json!({
        "roomId": room_id,
        "codeCount": items.len(),
        "codes": items.iter().map(|item| serde_json::json!({
            "code": item.code,
            "shareUrlCandidates": share_url_candidates(&item.code),
            "fileCount": item.total_cnt,
            "expiry": item.expiry,
        })).collect::<Vec<_>>(),
    })
}

fn auth_host_from_url(url: &str) -> Result<String> {
    let parsed = reqwest::Url::parse(url)
        .with_context(|| format!("failed to parse auth_server_url {url}"))?;
    parsed
        .host_str()
        .map(ToString::to_string)
        .context("auth_server_url did not contain a host")
}

fn try_handle_settings_command() -> Result<bool> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        return Ok(false);
    }

    match args[1].as_str() {
        "settings-show" | "share-settings-show" => {
            if settings_help_requested(&args[2..]) {
                print_settings_show_help();
                return Ok(true);
            }
            let options = parse_long_options(&args[2..])?;
            let settings_path = option_path(&options, "--settings-file")
                .unwrap_or_else(settings::default_settings_path);
            let stored = settings::load_settings_optional(&settings_path)?;
            let share_settings = stored
                .as_ref()
                .map(settings::share_settings_view)
                .transpose()?;
            write_json(&serde_json::json!({
                "settingsPath": settings_path,
                "stored": stored,
                "shareSettings": share_settings,
                "knownSharingPeriods": settings::sharing_periods().iter().map(|period| serde_json::json!({
                    "name": period.name,
                    "hours": period.hours,
                    "code": period.code,
                })).collect::<Vec<_>>(),
            }))?;
            Ok(true)
        }
        "settings-init" | "share-settings-init" => {
            if settings_help_requested(&args[2..]) {
                print_settings_init_help();
                return Ok(true);
            }
            let options = parse_long_options(&args[2..])?;
            let settings_path = option_path(&options, "--settings-file")
                .unwrap_or_else(settings::default_settings_path);
            let settings_value = settings::default_settings();
            settings::validate_settings(&settings_value)?;
            settings::write_settings(&settings_path, &settings_value)?;
            write_json(&serde_json::json!({
                "settingsPath": settings_path,
                "shareSettings": settings::share_settings_view(&settings_value)?,
                "settings": settings_value,
            }))?;
            Ok(true)
        }
        "settings-set" | "share-settings-set" => {
            if settings_help_requested(&args[2..]) {
                print_settings_set_help();
                return Ok(true);
            }
            let options = parse_long_options(&args[2..])?;
            let settings_path = option_path(&options, "--settings-file")
                .unwrap_or_else(settings::default_settings_path);
            let key = option_value(&options, "--key")
                .context("settings-set requires --key <name>")?;
            let value = option_value(&options, "--value")
                .context("settings-set requires --value <value>")?;
            let mut settings_value = settings::load_settings_optional(&settings_path)?
                .unwrap_or_else(settings::default_settings);
            settings::apply_key_value(&mut settings_value, &key, &value)?;
            settings::validate_settings(&settings_value)?;
            settings::write_settings(&settings_path, &settings_value)?;
            write_json(&serde_json::json!({
                "settingsPath": settings_path,
                "shareSettings": settings::share_settings_view(&settings_value)?,
                "settings": settings_value,
            }))?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn option_path(
    options: &std::collections::BTreeMap<String, String>,
    key: &str,
) -> Option<PathBuf> {
    option_value(options, key).map(PathBuf::from)
}

fn option_value(
    options: &std::collections::BTreeMap<String, String>,
    key: &str,
) -> Option<String> {
    options.get(key).cloned()
}

fn settings_help_requested(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--help" || arg == "-h")
}

fn parse_cli_bool(value: &str) -> Result<bool> {
    value
        .parse::<bool>()
        .with_context(|| format!("invalid bool value: {value}"))
}

fn parse_long_options(args: &[String]) -> Result<std::collections::BTreeMap<String, String>> {
    let mut options = std::collections::BTreeMap::new();
    let mut index = 0usize;
    while index < args.len() {
        let key = &args[index];
        if !key.starts_with("--") {
            bail!("unexpected positional argument: {key}");
        }
        let value = args
            .get(index + 1)
            .cloned()
            .with_context(|| format!("missing value for option {key}"))?;
        if value.starts_with("--") {
            bail!("missing value for option {key}");
        }
        options.insert(key.clone(), value);
        index += 2;
    }
    Ok(options)
}

fn print_settings_show_help() {
    println!("Usage: dropship-cmd settings-show [--settings-file <PATH>]");
    println!("Alias: dropship-cmd share-settings-show [--settings-file <PATH>]");
}

fn print_settings_init_help() {
    println!("Usage: dropship-cmd settings-init [--settings-file <PATH>]");
    println!("Alias: dropship-cmd share-settings-init [--settings-file <PATH>]");
}

fn print_settings_set_help() {
    println!("Usage: dropship-cmd settings-set [--settings-file <PATH>] --key <NAME> --value <VALUE>");
    println!("Alias: dropship-cmd share-settings-set [--settings-file <PATH>] --key <NAME> --value <VALUE>");
}

fn print_receive_info_help() {
    println!("Usage: dropship-cmd receive-info --code <CODE_OR_URL> [--password <TEXT>] [--session <PATH>] [--access-type <TYPE>]");
}

fn print_receive_help() {
    println!("Usage: dropship-cmd receive --code <CODE_OR_URL> [--password <TEXT>] [--output-dir <PATH>] [--session <PATH>] [--access-type <TYPE>] [--resume <true|false>] [--overwrite <true|false>]");
}

fn print_save_room_help() {
    println!("Usage: dropship-cmd save-room --request <PATH> [--session <PATH>]");
    println!("Usage: dropship-cmd create-room --title <TEXT> [--expiry <HOURS>] [--is-secure <true|false>] [--password <TEXT>] [--background-blur-hash <TEXT>] [--room-id <ROOM_ID>] [--session <PATH>]");
    println!("Alias: dropship-cmd create-room --request <PATH> [--session <PATH>]");
}

fn print_room_invite_help() {
    println!("Usage: dropship-cmd room-invite --request <PATH> [--session <PATH>]");
    println!("Usage: dropship-cmd room-invite --room-id <ROOM_ID> [--expiry <HOURS>] [--keyword <TEXT>] [--session <PATH>]");
    println!("Alias: dropship-cmd create-invitation --request <PATH> [--session <PATH>]");
}

fn print_join_room_help() {
    println!("Usage: dropship-cmd join-room --room-id <ROOM_ID> [--password <TEXT>] [--session <PATH>]");
}

fn print_room_codes_help() {
    println!("Usage: dropship-cmd room-codes --room-id <ROOM_ID> [--size <COUNT>] [--session <PATH>]");
}

fn print_refresh_session_help() {
    println!("Usage: dropship-cmd refresh-session [--session <PATH>]");
}

fn print_delete_room_invitation_help() {
    println!("Usage: dropship-cmd delete-room-invitation --room-id <ROOM_ID> [--session <PATH>]");
}

fn build_upload_session_file_request(
    path: &Path,
    extra: Option<std::collections::BTreeMap<String, String>>,
) -> Result<GoodlockWebUploadSessionFileRequest> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("failed to read file metadata {}", path.display()))?;
    if !metadata.is_file() {
        bail!("path is not a regular file: {}", path.display());
    }
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToString::to_string)
        .with_context(|| format!("failed to derive file name for {}", path.display()))?;
    let file_type = mime_guess::from_path(path)
        .first_raw()
        .unwrap_or("application/octet-stream")
        .to_string();

    Ok(GoodlockWebUploadSessionFileRequest {
        file_path: path.to_string_lossy().replace('\\', "/"),
        file_name,
        file_type,
        file_size: metadata.len(),
        extra,
    })
}

fn normalize_receive_code(input: &str) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        bail!("receive code cannot be empty");
    }
    if is_receive_code(trimmed) {
        return Ok(trimmed.to_string());
    }
    if let Some((keyword, digits)) = split_keyword_path(trimmed) {
        return Ok(format!("{keyword}{digits}"));
    }
    if trimmed.contains("://") {
        let summary = auth::classify_dropship_link(trimmed)?;
        if let Some(code) = summary.candidate_pin {
            return Ok(code);
        }
    }

    bail!("failed to resolve receive code from input: {trimmed}")
}

fn default_receive_output_dir(code: &str) -> PathBuf {
    let safe = code
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    PathBuf::from("received").join(safe)
}

struct PlannedReceiveTarget {
    path: PathBuf,
    action: &'static str,
    conflict_resolved: bool,
    skip_existing: bool,
    reason: Option<String>,
}

struct ReceiveFetchOutcome {
    request: GoodlockWebDownloadCodeRequest,
    response: auth::CandidateHttpParseResult<crate::models::GoodlockWebDownloadUrlsResponse>,
    prompted_for_password: bool,
}

fn fetch_receive_download_info(
    resolved_code: &str,
    password: Option<String>,
    access_type: String,
    headers: Option<&GoodlockWebHeadersInput>,
    allow_password_prompt: bool,
) -> Result<ReceiveFetchOutcome> {
    let mut request = GoodlockWebDownloadCodeRequest {
        code: resolved_code.to_string(),
        password: password.unwrap_or_default(),
        access_type,
    };
    let mut response = auth::request_goodlock_web_download_urls(
        "https://api.goodlocklabs.com/dropship/v1/downloadUrls",
        &request,
        headers,
    )?;
    let mut prompted_for_password = false;

    if allow_password_prompt
        && request.password.is_empty()
        && response.body.is_secure
        && response
            .body
            .file_list
            .as_ref()
            .map(|items| items.is_empty())
            .unwrap_or(true)
    {
        let password = prompt_line("Share password: ")?;
        if password.is_empty() {
            bail!("share is password protected; password was not provided");
        }
        request.password = password;
        response = auth::request_goodlock_web_download_urls(
            "https://api.goodlocklabs.com/dropship/v1/downloadUrls",
            &request,
            headers,
        )?;
        prompted_for_password = true;
    }

    if response.body.is_secure
        && !request.password.is_empty()
        && response
            .body
            .file_list
            .as_ref()
            .map(|items| items.is_empty())
            .unwrap_or(true)
    {
        bail!("share password was rejected or the share does not contain downloadable files");
    }

    Ok(ReceiveFetchOutcome {
        request,
        response,
        prompted_for_password,
    })
}

fn plan_receive_target(
    output_dir: &Path,
    file_name: &str,
    object_id: &str,
    expected_size: u64,
    resume: bool,
    overwrite: bool,
    used_paths: &mut BTreeSet<PathBuf>,
) -> Result<PlannedReceiveTarget> {
    let base_path = output_dir.join(file_name);
    let mut candidate = base_path.clone();
    let mut conflict_resolved = false;
    let mut reason = None;

    if used_paths.contains(&candidate) {
        candidate = unique_receive_path(output_dir, file_name, object_id, used_paths);
        conflict_resolved = true;
        reason = Some("duplicate file name in current share".to_string());
    } else if let Ok(metadata) = fs::metadata(&candidate) {
        if !metadata.is_file() {
            candidate = unique_receive_path(output_dir, file_name, object_id, used_paths);
            conflict_resolved = true;
            reason = Some("target path already exists and is not a regular file".to_string());
        } else if !overwrite && metadata.len() == expected_size {
            used_paths.insert(candidate.clone());
            return Ok(PlannedReceiveTarget {
                path: candidate,
                action: "skip",
                conflict_resolved,
                skip_existing: true,
                reason: Some("file already exists with matching size".to_string()),
            });
        } else if !overwrite && (!resume || metadata.len() > expected_size) {
            candidate = unique_receive_path(output_dir, file_name, object_id, used_paths);
            conflict_resolved = true;
            reason = Some("existing file would be overwritten".to_string());
        } else if overwrite {
            reason = Some("overwriting existing file".to_string());
        } else if resume && metadata.len() < expected_size {
            reason = Some("resuming partial file".to_string());
        }
    }

    used_paths.insert(candidate.clone());
    Ok(PlannedReceiveTarget {
        path: candidate,
        action: if overwrite { "overwrite" } else { "download" },
        conflict_resolved,
        skip_existing: false,
        reason,
    })
}

fn unique_receive_path(
    output_dir: &Path,
    file_name: &str,
    object_id: &str,
    used_paths: &BTreeSet<PathBuf>,
) -> PathBuf {
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("file");
    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty());
    let object_suffix = sanitize_object_suffix(object_id);

    for index in 1..1000 {
        let candidate_name = match extension {
            Some(extension) => format!("{stem}-{object_suffix}-{index}.{extension}"),
            None => format!("{stem}-{object_suffix}-{index}"),
        };
        let candidate = output_dir.join(candidate_name);
        if !used_paths.contains(&candidate) && !candidate.exists() {
            return candidate;
        }
    }

    output_dir.join(format!("{stem}-{object_suffix}-fallback.bin"))
}

fn sanitize_object_suffix(object_id: &str) -> String {
    let tail = object_id
        .rsplit('/')
        .next()
        .unwrap_or(object_id)
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .take(12)
        .collect::<String>();
    if tail.is_empty() {
        "obj".to_string()
    } else {
        tail
    }
}

fn safe_download_file_name(file_name: &str, object_id: &str) -> String {
    Path::new(file_name)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{object_id}.bin"))
}

fn share_url_candidates(code: &str) -> Vec<String> {
    let mut urls = Vec::new();
    if !code.is_empty() {
        urls.push(format!("https://g2sh.me/{code}"));
        if let Some((keyword, digits)) = split_keyword_code(code) {
            urls.push(format!("https://g2sh.me/{keyword}/{digits}"));
        }
    }
    urls.sort();
    urls.dedup();
    urls
}

fn split_keyword_path(input: &str) -> Option<(&str, &str)> {
    let (keyword, digits) = input.split_once('/')?;
    if keyword.len() >= 3
        && keyword.len() <= 10
        && keyword.chars().all(|ch| ch.is_ascii_lowercase())
        && digits.len() == 6
        && digits.chars().all(|ch| ch.is_ascii_digit())
    {
        Some((keyword, digits))
    } else {
        None
    }
}

fn split_keyword_code(code: &str) -> Option<(&str, &str)> {
    let split_at = code.find(|ch: char| ch.is_ascii_digit())?;
    let (keyword, digits) = code.split_at(split_at);
    if keyword.len() >= 3
        && keyword.len() <= 10
        && keyword.chars().all(|ch| ch.is_ascii_lowercase())
        && digits.len() == 6
        && digits.chars().all(|ch| ch.is_ascii_digit())
    {
        Some((keyword, digits))
    } else {
        None
    }
}

fn is_receive_code(input: &str) -> bool {
    input.len() == 6 && input.chars().all(|ch| ch.is_ascii_digit()) || split_keyword_code(input).is_some()
}
