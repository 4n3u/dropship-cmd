use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadersMetadata {
    pub authorization: String,
    pub app_type: String,
    pub auth_url: String,
    pub user_agent: String,
    pub device_type: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationMetadata {
    pub content_uri: String,
    pub title: String,
    pub progress_channel_id: String,
    pub progress_content_text: String,
    pub result_channel_id: String,
    pub completed_content_text: String,
    pub failed_content_text: String,
    pub cancel_button_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushMessageMeta {
    pub push_api_url: String,
    pub thumbnail_api_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub room_id: String,
    pub room_title: String,
    pub sender: String,
    pub sender_device_id: String,
    pub sender_profile_url: String,
    pub message: String,
    pub room_img_url: String,
    pub api_key: String,
    pub project_id: String,
    pub file_counts: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadMetadata {
    pub complete_api_url: String,
    pub cancel_api_url: String,
    pub headers: HeadersMetadata,
    pub notification: NotificationMetadata,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_message: Option<PushMessageMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadMetadata {
    pub notification: NotificationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadInfo {
    pub local_path: String,
    pub upload_url: String,
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadItem {
    pub server_id: String,
    pub thumbnail_server_id: String,
    pub file_info: FileUploadInfo,
    pub thumbnail_uplink: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OgUploadInfo {
    pub upload_url: String,
    pub server_id: String,
    pub file_name: String,
    pub buffer: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDownloadInfo {
    pub file_id: String,
    pub local_path: String,
    pub download_url: String,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerFileInfo {
    pub file_type: String,
    pub thumbnail_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletedItem {
    pub file_name: String,
    pub file_type: String,
    pub file_size: u64,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_object_id: Option<String>,
    pub is_og_image: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportItem {
    pub code: String,
    pub file_info_list: Vec<CompletedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletedItemPair {
    pub file: CompletedItem,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<CompletedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FcmRequest {
    pub fcm: FcmBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FcmBody {
    pub topic: String,
    pub data: FcmData,
    pub apns: ApnData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FcmData {
    pub room_id: String,
    pub room_title: String,
    pub sender: String,
    pub sender_device_id: String,
    pub pin: String,
    pub files_count: String,
    pub room_img_url: String,
    pub sender_profile_url: String,
    pub thumbnail_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApnData {
    pub headers: ApnHeaders,
    pub payload: ApnPayload,
    pub fcm_options: ApnFcmOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApnHeaders {
    #[serde(rename = "apns-priority")]
    pub priority: String,
    #[serde(rename = "apns-push-type")]
    pub push_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApnPayload {
    pub aps: ApnApsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApnApsData {
    pub alert: ApnAlert,
    pub sound: String,
    #[serde(rename = "mutable-content")]
    pub mutable_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApnAlert {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApnFcmOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailRequest {
    pub code: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamsungTokenInfo {
    pub access_token: String,
    pub access_token_expires_in: u64,
    #[serde(alias = "user_id", alias = "userId")]
    pub user_id: String,
    pub refresh_token: String,
    pub refresh_token_expires_in: u64,
    pub auth_server_url: String,
    pub api_server_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamsungUserProfileInfo {
    #[serde(alias = "userId")]
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_user_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebHeadersInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cookie: Option<String>,
    #[serde(default = "default_goodlock_auth_url")]
    pub auth_url: String,
    #[serde(default = "default_goodlock_device_type")]
    pub device_type: String,
    #[serde(default = "default_goodlock_app_type")]
    pub app_type: String,
    #[serde(default = "default_goodlock_version")]
    pub version: String,
    #[serde(default = "default_goodlock_origin")]
    pub origin: String,
    #[serde(default = "default_goodlock_referer")]
    pub referer: String,
    #[serde(default = "default_goodlock_user_agent")]
    pub user_agent: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionHeadersInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamsungSignInGateRequestCandidate {
    pub client_id: String,
    pub redirect_uri: String,
    pub response_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamsungIssueAccessTokenRequestCandidate {
    pub grant_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_verifier: Option<String>,
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivedUrlSummary {
    pub raw_url: String,
    pub scheme: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fragment: Option<String>,
    pub query: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropshipLinkSummary {
    pub raw_url: String,
    pub scheme: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fragment: Option<String>,
    pub query: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main_activity_route: Option<String>,
    pub category: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_flutter_route: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_pin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebLoginStateInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_info: Option<SamsungTokenInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_profile: Option<SamsungUserProfileInfo>,
    #[serde(default = "default_true")]
    pub has_connectivity: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_to_token_expiration: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebLoginStateOutput {
    pub login_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback: Option<ReceivedUrlSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<DropshipLinkSummary>,
    pub next_actions: Vec<String>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteWebLoginCandidateInput {
    pub received_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub client_id: String,
    pub redirect_uri: String,
    #[serde(default = "default_authorization_code")]
    pub grant_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_verifier: Option<String>,
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra_form: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamsungRefreshTokenRequestCandidate {
    #[serde(default = "default_refresh_token_grant")]
    pub grant_type: String,
    pub refresh_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebIssueAccessTokenRequest {
    pub code: String,
    #[serde(alias = "redirect_uri", alias = "redirectURI")]
    pub redirect_url: String,
    pub stay_signed_in: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebIssueAccessTokenResponse {
    #[serde(alias = "access_token")]
    pub access_token: String,
    #[serde(alias = "access_token_expires_in")]
    pub access_token_expiry: u64,
    #[serde(alias = "refresh_token")]
    pub refresh_token: String,
    #[serde(alias = "refresh_token_expires_in")]
    pub refresh_token_expiry: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebUserInfoResponse {
    #[serde(alias = "user_id")]
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nick_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebDeviceInfo {
    pub device_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebUserResponse {
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nick_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_keyword: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_profile: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need_agreement: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_extra_size: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_internal_user: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_info_list: Option<Vec<GoodlockWebDeviceInfo>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_quick_sending: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_auto_copy_link: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_auto_push_message: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebUsageResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cnt: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockSaveRoomRequest {
    pub room_id: String,
    pub title: String,
    pub expiry: u64,
    pub is_secure: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_blur_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockInvitationRequest {
    pub room_id: String,
    pub expiry: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockDownloadUrlsRequest {
    pub code_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebUploadSessionFileRequest {
    pub file_path: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebUploadSessionRequest {
    pub expiry: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nick_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub share_message: Option<String>,
    pub use_keyword: bool,
    pub is_internal_user: bool,
    pub is_only_me: bool,
    pub use_profile: bool,
    pub use_extra_size: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_url: Option<String>,
    pub is_invitation: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub file_info_list: Vec<GoodlockWebUploadSessionFileRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebUploadSessionFileResponse {
    pub object_id: String,
    pub upload_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_object_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebUploadSessionResponse {
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub og_upload_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub og_object_id: Option<String>,
    pub file_info_list: Vec<GoodlockWebUploadSessionFileResponse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebUploadSessionCompleteFileRequest {
    pub file_name: String,
    pub file_type: String,
    pub file_size: u64,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_object_id: Option<String>,
    pub is_og_image: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebUploadSessionCompleteRequest {
    pub code: String,
    pub file_info_list: Vec<GoodlockWebUploadSessionCompleteFileRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebDownloadCodeRequest {
    pub code: String,
    #[serde(default = "default_empty_string")]
    pub password: String,
    #[serde(default = "default_validate_access_type")]
    pub access_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockWebDownloadUrlsResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_list: Option<Vec<GoodlockRoomFileInfoResponse>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_cnt: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invitation: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
    pub is_secure: bool,
    pub is_gts: bool,
    pub is_creator: bool,
    pub is_only_me: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockInvitationInfoResponse {
    pub room_id: String,
    pub expiry: String,
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub img_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockRoomInfoResponse {
    pub room_id: String,
    pub user_id: String,
    pub title: String,
    pub expiry: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub img_url: Option<String>,
    pub is_secure: bool,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invitation: Option<GoodlockInvitationInfoResponse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_blur_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockRoomMemberInfoResponse {
    pub user_id: String,
    pub nick_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_img_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockRoomFileInfoResponse {
    pub file_name: String,
    pub file_type: String,
    pub file_size: u64,
    pub file_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    pub object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoodlockRoomDownloadUrlsResponse {
    pub code: String,
    pub expiry: String,
    pub total_cnt: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_list: Option<Vec<GoodlockRoomFileInfoResponse>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginSession {
    pub sign_in_url: String,
    pub callback: ReceivedUrlSummary,
    pub samsung_token_info: SamsungTokenInfo,
    pub goodlock_token: GoodlockWebIssueAccessTokenResponse,
    pub user_info: GoodlockWebUserInfoResponse,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<GoodlockWebUserResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUploadSessionRequestCandidate {
    pub room_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub share_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_count: Option<serde_json::Value>,
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadUrlsRequestCandidate {
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadStorageInfoCandidate {
    pub upload_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_upload_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin_string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomDownloadUrlsResponseCandidate {
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_urls: Option<Vec<FileDownloadInfo>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub share_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expire_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator_profile_image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator_nick_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_code_info_list: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_status: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfoResponseCandidate {
    pub room_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub join_status: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub share_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_img_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_img_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<RoomMemberInfoResponseCandidate>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<RoomFileInfoResponseCandidate>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_devices: Option<Vec<PushDeviceInfoCandidate>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomBubbleInfoResponseCandidate {
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub img_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub join_status: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationInfoResponseCandidate {
    pub room_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "inviteUrl", alias = "shareURL")]
    pub invite_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "inviteCode", alias = "pinString")]
    pub invite_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRoomRequestCandidate {
    pub room_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub favorite: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "joinedAt")]
    pub joined_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomMemberInfoResponseCandidate {
    #[serde(alias = "userId")]
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "nickName")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "creatorProfileImage")]
    pub profile_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomFileInfoResponseCandidate {
    #[serde(alias = "fileName")]
    pub file_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "fileSize")]
    pub file_size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "thumbnailUrl")]
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PushDeviceInfoCandidate {
    #[serde(alias = "deviceId")]
    pub device_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub push_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ServerFilesInput {
    Wrapped {
        #[serde(rename = "fileList", alias = "file_list")]
        file_list: Vec<ServerFileInfo>,
    },
    Bare(Vec<ServerFileInfo>),
}

impl ServerFilesInput {
    pub fn into_vec(self) -> Vec<ServerFileInfo> {
        match self {
            Self::Wrapped { file_list } => file_list,
            Self::Bare(items) => items,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_authorization_code() -> String {
    "authorization_code".to_string()
}

fn default_refresh_token_grant() -> String {
    "refresh_token".to_string()
}

fn default_goodlock_auth_url() -> String {
    "eu-auth2.samsungosp.com".to_string()
}

fn default_goodlock_device_type() -> String {
    "WEB".to_string()
}

fn default_goodlock_app_type() -> String {
    "dropship_web".to_string()
}

fn default_goodlock_version() -> String {
    "1.2.6".to_string()
}

fn default_goodlock_origin() -> String {
    "https://g2sh.me".to_string()
}

fn default_goodlock_referer() -> String {
    "https://g2sh.me/".to_string()
}

fn default_goodlock_user_agent() -> String {
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36".to_string()
}

fn default_empty_string() -> String {
    String::new()
}

fn default_validate_access_type() -> String {
    "VALIDATE".to_string()
}
