use serde::Serialize;

use crate::models::SamsungTokenInfo;

use anyhow::{Result, bail};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnownEndpoints {
    pub samsung_account: SamsungAccountEndpoints,
    pub dropship_api: DropshipApiEndpoints,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedEndpoints {
    pub auth_server_url: String,
    pub api_server_url: String,
    pub oauth_token: String,
    pub issue_access_token: String,
    pub logout: String,
    pub user_info: String,
    pub user_profile: String,
    pub user_verify: String,
    pub user_device_info: String,
    pub user_nickname: String,
    pub user_share_option: String,
    pub user_usage: String,
    pub user_usage_limit: String,
    pub user_policy: String,
    pub room_info: String,
    pub room_info_joined: String,
    pub room_invitation: String,
    pub room_member: String,
    pub room_member_kick: String,
    pub room_upload_session_base: String,
    pub upload_session_complete: String,
    pub download_urls: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SamsungAccountEndpoints {
    pub account_host_hint: String,
    pub oauth_host_hint: String,
    pub sign_in_gate: String,
    pub sign_out_gate: String,
    pub user_verify_service_gate: String,
    pub oauth_token: String,
    pub issue_access_token: String,
    pub logout: String,
    pub user_info: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DropshipApiEndpoints {
    pub user_profile: String,
    pub user_verify: String,
    pub user_device_info: String,
    pub user_device_info_collection: String,
    pub user_nickname: String,
    pub user_share_option: String,
    pub user_usage: String,
    pub user_usage_limit: String,
    pub user_policy: String,
    pub user_internal: String,
    pub room_info: String,
    pub room_info_joined: String,
    pub room_invitation: String,
    pub room_member: String,
    pub room_member_kick: String,
    pub room_upload_session: String,
    pub upload_session_complete: String,
    pub download_urls: String,
}

#[derive(Debug, Clone, Copy)]
pub enum EndpointKind {
    SamsungSignInGate,
    SamsungSignOutGate,
    SamsungUserVerifyServiceGate,
    SamsungOauthToken,
    SamsungIssueAccessToken,
    SamsungLogout,
    SamsungUserInfo,
    UserProfile,
    UserVerify,
    UserDeviceInfo,
    UserDeviceInfoCollection,
    UserNickname,
    UserShareOption,
    UserUsage,
    UserUsageLimit,
    UserPolicy,
    UserInternal,
    RoomInfo,
    RoomInfoJoined,
    RoomInvitation,
    RoomMember,
    RoomMemberKick,
    RoomUploadSession,
    UploadSessionComplete,
    DownloadUrls,
}

impl EndpointKind {
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "samsung-sign-in-gate" => Self::SamsungSignInGate,
            "samsung-sign-out-gate" => Self::SamsungSignOutGate,
            "samsung-user-verify-service-gate" => Self::SamsungUserVerifyServiceGate,
            "samsung-oauth-token" => Self::SamsungOauthToken,
            "samsung-issue-access-token" => Self::SamsungIssueAccessToken,
            "samsung-logout" => Self::SamsungLogout,
            "samsung-user-info" => Self::SamsungUserInfo,
            "user-profile" => Self::UserProfile,
            "user-verify" => Self::UserVerify,
            "user-device-info" => Self::UserDeviceInfo,
            "user-device-info-collection" => Self::UserDeviceInfoCollection,
            "user-nickname" => Self::UserNickname,
            "user-share-option" => Self::UserShareOption,
            "user-usage" => Self::UserUsage,
            "user-usage-limit" => Self::UserUsageLimit,
            "user-policy" => Self::UserPolicy,
            "user-internal" => Self::UserInternal,
            "room-info" => Self::RoomInfo,
            "room-info-joined" => Self::RoomInfoJoined,
            "room-invitation" => Self::RoomInvitation,
            "room-member" => Self::RoomMember,
            "room-member-kick" => Self::RoomMemberKick,
            "room-upload-session" => Self::RoomUploadSession,
            "upload-session-complete" => Self::UploadSessionComplete,
            "download-urls" => Self::DownloadUrls,
            _ => return None,
        })
    }
}

pub fn known_endpoints() -> KnownEndpoints {
    KnownEndpoints {
        samsung_account: SamsungAccountEndpoints {
            account_host_hint: "https://account.samsung.com".to_string(),
            oauth_host_hint: "https://eu-auth2.samsungosp.com".to_string(),
            sign_in_gate: "/accounts/v1/dropship-web/signInGate".to_string(),
            sign_out_gate: "/accounts/v1/dropship-web/signOutGate".to_string(),
            user_verify_service_gate: "/accounts/dfltMobileHybrid/userVerifyServiceGate"
                .to_string(),
            oauth_token: "/auth/oauth2/token".to_string(),
            issue_access_token: "/dropship-web/v1/user/issueAccessToken".to_string(),
            logout: "/dropship-web/v1/user/logout".to_string(),
            user_info: "/dropship-web/v1/user/userInfo".to_string(),
        },
        dropship_api: DropshipApiEndpoints {
            user_profile: "/user/profile".to_string(),
            user_verify: "/user/verify".to_string(),
            user_device_info: "/user/device-info".to_string(),
            user_device_info_collection: "/user/device-info/".to_string(),
            user_nickname: "/user/nickname".to_string(),
            user_share_option: "/user/shareOption".to_string(),
            user_usage: "/user/usage".to_string(),
            user_usage_limit: "/user/usage/limit".to_string(),
            user_policy: "/user/policy".to_string(),
            user_internal: "/user/internal".to_string(),
            room_info: "/room/info".to_string(),
            room_info_joined: "/room/info/joined".to_string(),
            room_invitation: "/room/invitation".to_string(),
            room_member: "/room/member".to_string(),
            room_member_kick: "/room/member/kick".to_string(),
            room_upload_session: "/room/uploadSession/".to_string(),
            upload_session_complete: "/uploadSession/complete".to_string(),
            download_urls: "/downloadUrls".to_string(),
        },
    }
}

pub fn build_endpoint(base_url: &str, kind: EndpointKind, suffix: Option<&str>) -> Result<String> {
    let endpoints = known_endpoints();
    let mut path = match kind {
        EndpointKind::SamsungSignInGate => endpoints.samsung_account.sign_in_gate,
        EndpointKind::SamsungSignOutGate => endpoints.samsung_account.sign_out_gate,
        EndpointKind::SamsungUserVerifyServiceGate => {
            endpoints.samsung_account.user_verify_service_gate
        }
        EndpointKind::SamsungOauthToken => endpoints.samsung_account.oauth_token,
        EndpointKind::SamsungIssueAccessToken => endpoints.samsung_account.issue_access_token,
        EndpointKind::SamsungLogout => endpoints.samsung_account.logout,
        EndpointKind::SamsungUserInfo => endpoints.samsung_account.user_info,
        EndpointKind::UserProfile => endpoints.dropship_api.user_profile,
        EndpointKind::UserVerify => endpoints.dropship_api.user_verify,
        EndpointKind::UserDeviceInfo => endpoints.dropship_api.user_device_info,
        EndpointKind::UserDeviceInfoCollection => {
            endpoints.dropship_api.user_device_info_collection
        }
        EndpointKind::UserNickname => endpoints.dropship_api.user_nickname,
        EndpointKind::UserShareOption => endpoints.dropship_api.user_share_option,
        EndpointKind::UserUsage => endpoints.dropship_api.user_usage,
        EndpointKind::UserUsageLimit => endpoints.dropship_api.user_usage_limit,
        EndpointKind::UserPolicy => endpoints.dropship_api.user_policy,
        EndpointKind::UserInternal => endpoints.dropship_api.user_internal,
        EndpointKind::RoomInfo => endpoints.dropship_api.room_info,
        EndpointKind::RoomInfoJoined => endpoints.dropship_api.room_info_joined,
        EndpointKind::RoomInvitation => endpoints.dropship_api.room_invitation,
        EndpointKind::RoomMember => endpoints.dropship_api.room_member,
        EndpointKind::RoomMemberKick => endpoints.dropship_api.room_member_kick,
        EndpointKind::RoomUploadSession => endpoints.dropship_api.room_upload_session,
        EndpointKind::UploadSessionComplete => endpoints.dropship_api.upload_session_complete,
        EndpointKind::DownloadUrls => endpoints.dropship_api.download_urls,
    };

    let base = base_url.trim_end_matches('/');
    if base.is_empty() {
        bail!("base_url must not be empty");
    }

    if let Some(suffix) = suffix {
        let suffix = suffix.trim_matches('/');
        if !suffix.is_empty() {
            if !path.ends_with('/') {
                path.push('/');
            }
            path.push_str(suffix);
        }
    }

    Ok(format!("{base}{path}"))
}

pub fn derive_endpoints_from_token(token: &SamsungTokenInfo) -> Result<DerivedEndpoints> {
    Ok(DerivedEndpoints {
        auth_server_url: token.auth_server_url.clone(),
        api_server_url: token.api_server_url.clone(),
        oauth_token: build_endpoint(
            &token.auth_server_url,
            EndpointKind::SamsungOauthToken,
            None,
        )?,
        issue_access_token: build_endpoint(
            &token.api_server_url,
            EndpointKind::SamsungIssueAccessToken,
            None,
        )?,
        logout: build_endpoint(&token.api_server_url, EndpointKind::SamsungLogout, None)?,
        user_info: build_endpoint(&token.api_server_url, EndpointKind::SamsungUserInfo, None)?,
        user_profile: build_endpoint(&token.api_server_url, EndpointKind::UserProfile, None)?,
        user_verify: build_endpoint(&token.api_server_url, EndpointKind::UserVerify, None)?,
        user_device_info: build_endpoint(
            &token.api_server_url,
            EndpointKind::UserDeviceInfo,
            None,
        )?,
        user_nickname: build_endpoint(&token.api_server_url, EndpointKind::UserNickname, None)?,
        user_share_option: build_endpoint(
            &token.api_server_url,
            EndpointKind::UserShareOption,
            None,
        )?,
        user_usage: build_endpoint(&token.api_server_url, EndpointKind::UserUsage, None)?,
        user_usage_limit: build_endpoint(
            &token.api_server_url,
            EndpointKind::UserUsageLimit,
            None,
        )?,
        user_policy: build_endpoint(&token.api_server_url, EndpointKind::UserPolicy, None)?,
        room_info: build_endpoint(&token.api_server_url, EndpointKind::RoomInfo, None)?,
        room_info_joined: build_endpoint(
            &token.api_server_url,
            EndpointKind::RoomInfoJoined,
            None,
        )?,
        room_invitation: build_endpoint(&token.api_server_url, EndpointKind::RoomInvitation, None)?,
        room_member: build_endpoint(&token.api_server_url, EndpointKind::RoomMember, None)?,
        room_member_kick: build_endpoint(
            &token.api_server_url,
            EndpointKind::RoomMemberKick,
            None,
        )?,
        room_upload_session_base: build_endpoint(
            &token.api_server_url,
            EndpointKind::RoomUploadSession,
            None,
        )?,
        upload_session_complete: build_endpoint(
            &token.api_server_url,
            EndpointKind::UploadSessionComplete,
            None,
        )?,
        download_urls: build_endpoint(&token.api_server_url, EndpointKind::DownloadUrls, None)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_room_upload_session_endpoint_with_suffix() {
        let url = build_endpoint(
            "https://example.test/api",
            EndpointKind::RoomUploadSession,
            Some("room-123"),
        )
        .expect("endpoint should build");

        assert_eq!(url, "https://example.test/api/room/uploadSession/room-123");
    }

    #[test]
    fn builds_plain_endpoint_without_suffix() {
        let url = build_endpoint("https://example.test", EndpointKind::UserProfile, None)
            .expect("endpoint should build");

        assert_eq!(url, "https://example.test/user/profile");
    }

    #[test]
    fn derives_known_urls_from_token_hosts() {
        let token = SamsungTokenInfo {
            access_token: "token".to_string(),
            access_token_expires_in: 3600,
            user_id: "user".to_string(),
            refresh_token: "refresh".to_string(),
            refresh_token_expires_in: 7200,
            auth_server_url: "https://auth.example.test".to_string(),
            api_server_url: "https://api.example.test".to_string(),
        };

        let derived = derive_endpoints_from_token(&token).expect("derived endpoints should build");

        assert_eq!(
            derived.oauth_token,
            "https://auth.example.test/auth/oauth2/token"
        );
        assert_eq!(
            derived.issue_access_token,
            "https://api.example.test/dropship-web/v1/user/issueAccessToken"
        );
        assert_eq!(
            derived.room_upload_session_base,
            "https://api.example.test/room/uploadSession/"
        );
    }
}
