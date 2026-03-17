use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateModelSet {
    pub samsung_token_info: SamsungTokenInfoShape,
    pub samsung_user_profile_info: GenericCandidateShape,
    pub login_state: GenericCandidateShape,
    pub login_waiting_result: GenericCandidateShape,
    pub samsung_sign_in_gate_request: GenericCandidateShape,
    pub samsung_issue_access_token_request: GenericCandidateShape,
    pub create_upload_session_request: GenericCandidateShape,
    pub download_urls_request: GenericCandidateShape,
    pub upload_storage_info: UploadStorageInfoShape,
    pub room_download_urls_response: RoomDownloadUrlsResponseShape,
    pub room_info_response: GenericCandidateShape,
    pub room_bubble_info_response: GenericCandidateShape,
    pub invitation_info_response: GenericCandidateShape,
    pub save_room_request: GenericCandidateShape,
    pub room_member_info_response: GenericCandidateShape,
    pub room_file_info_response: GenericCandidateShape,
    pub push_device_info: GenericCandidateShape,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateFlowSet {
    pub samsung_sign_in_gate: CandidateFlow,
    pub samsung_token_exchange: CandidateFlow,
    pub create_upload_session: CandidateFlow,
    pub get_download_urls: CandidateFlow,
    pub request_my_upload_info: CandidateFlow,
    pub room_info: CandidateFlow,
    pub invitation_join_flow: CandidateFlow,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateFlow {
    pub status: &'static str,
    pub endpoint_kind: &'static str,
    pub default_method: &'static str,
    pub evidence: Vec<&'static str>,
    pub request_fields: Vec<FieldShape>,
    pub response_fields: Vec<FieldShape>,
    pub companion_endpoints: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SamsungTokenInfoShape {
    pub status: &'static str,
    pub evidence: Vec<&'static str>,
    pub fields: Vec<FieldShape>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadStorageInfoShape {
    pub status: &'static str,
    pub evidence: Vec<&'static str>,
    pub fields: Vec<FieldShape>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomDownloadUrlsResponseShape {
    pub status: &'static str,
    pub evidence: Vec<&'static str>,
    pub fields: Vec<FieldShape>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericCandidateShape {
    pub status: &'static str,
    pub evidence: Vec<&'static str>,
    pub fields: Vec<FieldShape>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldShape {
    pub name: &'static str,
    pub confidence: &'static str,
    pub source: &'static str,
}

pub fn candidate_models() -> CandidateModelSet {
    CandidateModelSet {
        samsung_token_info: SamsungTokenInfoShape {
            status: "confirmed_from_native_plugin",
            evidence: vec![
                "jadx-out/sources/db/a.java maps Samsung account token bundle fields into a result map",
                "libapp.so string table contains storeAccessToken, storeRefreshToken, auth_server_url, api_server_url",
            ],
            fields: vec![
                field("access_token", "confirmed", "db/a.java"),
                field("access_token_expires_in", "confirmed", "db/a.java"),
                field("userId", "confirmed", "db/a.java"),
                field("refresh_token", "confirmed", "db/a.java"),
                field("refresh_token_expires_in", "confirmed", "db/a.java"),
                field(
                    "auth_server_url",
                    "confirmed",
                    "db/a.java and libapp.so strings",
                ),
                field(
                    "api_server_url",
                    "confirmed",
                    "db/a.java and libapp.so strings",
                ),
            ],
        },
        samsung_user_profile_info: GenericCandidateShape {
            status: "confirmed_from_native_plugin_and_content_provider",
            evidence: vec![
                "jadx-out/sources/db/a.java getUserProfile maps Samsung account bundle fields into Flutter result map",
                "jadx-out/sources/db/a.java also reads local Samsung profile provider fields account_nickname, account_given_name, account_family_name",
            ],
            fields: vec![
                field("userId", "confirmed", "db/a.java"),
                field("countryCode", "confirmed", "db/a.java"),
                field("birthData", "confirmed", "db/a.java"),
                field("email", "confirmed", "db/a.java"),
                field("preferredUserName", "confirmed", "db/a.java"),
                field("nickname", "confirmed", "db/a.java local profile query"),
                field("givenName", "confirmed", "db/a.java local profile query"),
                field("familyName", "confirmed", "db/a.java local profile query"),
            ],
        },
        login_state: GenericCandidateShape {
            status: "high_confidence_from_aot_strings",
            evidence: vec![
                "libapp.so contains LoginState.initializing(), LoginState.loggingIn(), LoginState.loggedIn(), LoginState.notLoggedIn(hasConnectivity:), LoginState.loginFailed(hasConnectivity:), LoginState.loggingOut(dueToTokenExpiration:)",
            ],
            fields: vec![
                field("initializing", "high", "libapp.so strings"),
                field("loggingIn", "high", "libapp.so strings"),
                field("loggedIn", "high", "libapp.so strings"),
                field("notLoggedIn", "high", "libapp.so strings"),
                field("loginFailed", "high", "libapp.so strings"),
                field("loggingOut", "high", "libapp.so strings"),
                field("hasConnectivity", "medium", "libapp.so strings"),
                field("dueToTokenExpiration", "medium", "libapp.so strings"),
            ],
        },
        login_waiting_result: GenericCandidateShape {
            status: "inferred_from_aot_strings",
            evidence: vec![
                "libapp.so contains LoginWaitingResult, LoginWaitingState(result:), authCode, receivedUrl, pleaseSignIn, checkingUserInfo, getUserInfo fail",
                "libapp.so contains WebLoginResponseRoute and LoginWaitingRoute",
            ],
            fields: vec![
                field("receivedUrl", "high", "libapp.so strings"),
                field("authCode", "medium", "libapp.so strings"),
                field("pleaseSignIn", "medium", "libapp.so strings"),
                field("checkingUserInfo", "medium", "libapp.so strings"),
                field("userInfo", "low", "libapp.so strings"),
                field("countryCode", "low", "libapp.so strings"),
            ],
        },
        samsung_sign_in_gate_request: GenericCandidateShape {
            status: "inferred_from_aot_strings_and_pigeon_signatures",
            evidence: vec![
                "libapp.so contains /accounts/v1/dropship-web/signInGate",
                "libapp.so contains client_id, redirect_uri, response_type",
                "libapp.so contains dropship://g2sh.me and processReceivedURL",
                "jadx-out/sources/db/u.java signIn and initialize methods require applicationID, redirectURI, codeVerifier, stateCode",
            ],
            fields: vec![
                field("client_id", "high", "libapp.so strings"),
                field(
                    "redirect_uri",
                    "high",
                    "libapp.so strings and manifest deep link",
                ),
                field("response_type", "high", "libapp.so strings"),
                field("state", "medium", "processReceivedURL(stateCode) signature"),
                field("code_verifier", "medium", "db/u.java signature"),
            ],
        },
        samsung_issue_access_token_request: GenericCandidateShape {
            status: "inferred_from_aot_strings_and_pigeon_signatures",
            evidence: vec![
                "libapp.so contains dropship-web/v1/user/issueAccessToken, auth/oauth2/token, and grant_type",
                "libapp.so contains access_token, refresh_token, auth_server_url, api_server_url",
                "jadx-out/sources/db/e.java exposes AUTHCODE, CODEANDREDIRECTURI, REFRESHTOKEN access-token modes",
                "jadx-out/sources/db/u.java getAccessToken method requires getAccessTokenType and codeVerifier",
            ],
            fields: vec![
                field("grant_type", "high", "libapp.so strings"),
                field("client_id", "medium", "shared auth request inference"),
                field("redirect_uri", "medium", "shared auth request inference"),
                field("code", "medium", "AUTHCODE flow and receivedUrl inference"),
                field("refresh_token", "medium", "db/e.java REFRESHTOKEN mode"),
                field("code_verifier", "medium", "db/u.java signature"),
            ],
        },
        create_upload_session_request: GenericCandidateShape {
            status: "inferred_from_aot_strings",
            evidence: vec![
                "libapp.so contains createUploadSession and /room/uploadSession/",
                "libapp.so contains roomId, shareMessage, password, duration, fileCount in room/group-share string clusters",
                "UploadStorageInfo(uploadUrl:, thumbnailUploadUrl, pinString) appears in the same upload-session flow",
            ],
            fields: vec![
                field("roomId", "high", "libapp.so strings"),
                field("shareMessage", "medium", "libapp.so strings"),
                field("password", "medium", "libapp.so strings"),
                field("duration", "medium", "libapp.so strings"),
                field("fileCount", "medium", "libapp.so strings"),
            ],
        },
        download_urls_request: GenericCandidateShape {
            status: "inferred_from_aot_strings",
            evidence: vec![
                "libapp.so contains getDownloadUrls and /downloadUrls",
                "receive flow strings include password card, invalid-pin-code, and pin-check transitions",
                "thumbnail lookup already uses a code/password body for another transfer endpoint",
            ],
            fields: vec![
                field("code", "high", "libapp.so strings"),
                field("password", "medium", "libapp.so strings"),
            ],
        },
        upload_storage_info: UploadStorageInfoShape {
            status: "inferred_from_aot_strings",
            evidence: vec![
                "libapp.so contains UploadStorageInfo(uploadUrl:",
                "libapp.so contains field fragments thumbnailUploadUrl and pinString adjacent to UploadStorageInfo references",
                "libapp.so contains get:uploadStorageInfo and createUploadSession",
            ],
            fields: vec![
                field("uploadUrl", "high", "libapp.so strings"),
                field("thumbnailUploadUrl", "high", "libapp.so strings"),
                field("pinString", "high", "libapp.so strings"),
            ],
        },
        room_download_urls_response: RoomDownloadUrlsResponseShape {
            status: "inferred_from_aot_strings",
            evidence: vec![
                "libapp.so contains RoomDownloadUrlsResponse(code:",
                "libapp.so contains getDownloadUrls and /downloadUrls",
                "libapp.so contains shareURL, expireAt, creatorProfileImage, creatorNickName, roomCodeInfoList, roomStatus near response references",
            ],
            fields: vec![
                field("code", "high", "libapp.so strings"),
                field("downloadUrls", "high", "libapp.so strings"),
                field("shareURL", "medium", "libapp.so strings"),
                field("expireAt", "medium", "libapp.so strings"),
                field("creatorProfileImage", "medium", "libapp.so strings"),
                field("creatorNickName", "medium", "libapp.so strings"),
                field("roomCodeInfoList", "medium", "libapp.so strings"),
                field("roomStatus", "medium", "libapp.so strings"),
            ],
        },
        room_info_response: GenericCandidateShape {
            status: "inferred_from_aot_strings",
            evidence: vec![
                "libapp.so contains RoomInfoResponse(roomId:",
                "libapp.so contains roomTitle, shareURL, profileImgUrl, members, files, owner, pushDevices, memberCount, status, joinStatus fragments near room-info references",
                "libapp.so contains /room/info and /room/info/joined",
            ],
            fields: vec![
                field("roomId", "high", "libapp.so strings"),
                field("roomTitle", "medium", "libapp.so strings"),
                field("shareURL", "medium", "libapp.so strings"),
                field("profileImgUrl", "medium", "libapp.so strings"),
                field("members", "medium", "libapp.so strings"),
                field("files", "medium", "libapp.so strings"),
                field("pushDevices", "medium", "libapp.so strings"),
                field("memberCount", "medium", "libapp.so strings"),
                field("status", "medium", "libapp.so strings"),
                field("joinStatus", "low", "libapp.so strings"),
            ],
        },
        room_bubble_info_response: GenericCandidateShape {
            status: "inferred_from_aot_strings",
            evidence: vec![
                "libapp.so contains RoomBubbleInfoResponse(code:",
                "libapp.so contains owner, filesCount, imgUrl, joinStatus, status fragments near room bubble references",
            ],
            fields: vec![
                field("code", "high", "libapp.so strings"),
                field("owner", "medium", "libapp.so strings"),
                field("filesCount", "medium", "libapp.so strings"),
                field("imgUrl", "medium", "libapp.so strings"),
                field("joinStatus", "low", "libapp.so strings"),
                field("status", "low", "libapp.so strings"),
            ],
        },
        invitation_info_response: GenericCandidateShape {
            status: "inferred_from_aot_strings",
            evidence: vec![
                "libapp.so contains InvitationInfoResponse(roomId:",
                "libapp.so contains roomTitle and roomId field fragments near invitation references",
            ],
            fields: vec![
                field("roomId", "high", "libapp.so strings"),
                field("roomTitle", "medium", "libapp.so strings"),
            ],
        },
        save_room_request: GenericCandidateShape {
            status: "inferred_from_aot_strings",
            evidence: vec![
                "libapp.so contains SaveRoomRequest(roomId:",
                "libapp.so contains joinedAt near uploadStorageInfo and room persistence strings",
            ],
            fields: vec![
                field("roomId", "high", "libapp.so strings"),
                field("joinedAt", "medium", "libapp.so strings"),
                field(
                    "favorite",
                    "low",
                    "app behavior inference from saveRoom/saveRoomInfos strings",
                ),
            ],
        },
        room_member_info_response: GenericCandidateShape {
            status: "inferred_from_aot_strings",
            evidence: vec![
                "libapp.so contains RoomMemberInfoResponse(userId:",
                "libapp.so contains userId and nickName field fragments",
            ],
            fields: vec![
                field("userId", "high", "libapp.so strings"),
                field("nickName", "medium", "libapp.so strings"),
            ],
        },
        room_file_info_response: GenericCandidateShape {
            status: "inferred_from_aot_strings",
            evidence: vec![
                "libapp.so contains RoomFileInfoResponse(fileName:",
                "libapp.so contains fileName, fileSize, mimeType, thumbnailUrl fragments",
            ],
            fields: vec![
                field("fileName", "high", "libapp.so strings"),
                field("fileSize", "medium", "libapp.so strings"),
                field("mimeType", "medium", "libapp.so strings"),
                field("thumbnailUrl", "medium", "libapp.so strings"),
            ],
        },
        push_device_info: GenericCandidateShape {
            status: "inferred_from_aot_strings",
            evidence: vec![
                "libapp.so contains PushDeviceInfo(deviceId:",
                "libapp.so contains creatorProfileImage, deviceId, senderDeviceId style fragments in push-related code",
            ],
            fields: vec![
                field("deviceId", "high", "libapp.so strings"),
                field("deviceName", "low", "name fragment not yet confirmed"),
                field(
                    "pushToken",
                    "low",
                    "token field not yet confirmed in Dart AOT strings",
                ),
            ],
        },
    }
}

pub fn candidate_flows() -> CandidateFlowSet {
    CandidateFlowSet {
        samsung_sign_in_gate: CandidateFlow {
            status: "inferred_from_aot_strings_and_manifest",
            endpoint_kind: "samsung-sign-in-gate",
            default_method: "GET",
            evidence: vec![
                "libapp.so offset 0xfad49 contains /accounts/v1/dropship-web/signInGate",
                "libapp.so contains client_id, redirect_uri, response_type, processReceivedURL, receivedUrl",
                "apktool-out/AndroidManifest.xml declares dropship://g2sh.me and https://g2sh.me deep links",
                "jadx-out/sources/db/u.java signIn signature includes applicationID, redirectURI, codeVerifier, stateCode",
            ],
            request_fields: vec![
                field("client_id", "high", "libapp.so strings"),
                field("redirect_uri", "high", "manifest and libapp.so strings"),
                field("response_type", "high", "libapp.so strings"),
                field("state", "medium", "processReceivedURL(stateCode)"),
                field("code_verifier", "medium", "db/u.java signature"),
            ],
            response_fields: vec![
                field(
                    "receivedUrl",
                    "high",
                    "processReceivedURL and receivedUrl strings",
                ),
                field(
                    "code",
                    "medium",
                    "authCode string and callback parsing inference",
                ),
                field("state", "medium", "stateCode callback parsing inference"),
            ],
            companion_endpoints: vec![
                "samsung-user-verify-service-gate",
                "samsung-oauth-token",
                "samsung-issue-access-token",
                "samsung-user-info",
            ],
        },
        samsung_token_exchange: CandidateFlow {
            status: "inferred_from_aot_strings_and_pigeon_signatures",
            endpoint_kind: "samsung-issue-access-token",
            default_method: "POST",
            evidence: vec![
                "libapp.so offset 0x5a3de contains dropship-web/v1/user/issueAccessToken",
                "libapp.so offset 0xf2f47 contains auth/oauth2/token",
                "libapp.so contains grant_type, access_token, refresh_token, auth_server_url, api_server_url",
                "jadx-out/sources/db/e.java exposes AUTHCODE, CODEANDREDIRECTURI, REFRESHTOKEN modes",
            ],
            request_fields: vec![
                field("grant_type", "high", "libapp.so strings"),
                field("client_id", "medium", "shared auth flow inference"),
                field("redirect_uri", "medium", "shared auth flow inference"),
                field("code", "medium", "AUTHCODE flow inference"),
                field("refresh_token", "medium", "REFRESHTOKEN mode"),
                field("code_verifier", "medium", "db/u.java signature"),
            ],
            response_fields: vec![
                field("access_token", "high", "db/a.java"),
                field("access_token_expires_in", "high", "db/a.java"),
                field("userId", "high", "db/a.java"),
                field("refresh_token", "high", "db/a.java field normalization"),
                field("auth_server_url", "high", "db/a.java"),
                field("api_server_url", "high", "db/a.java"),
            ],
            companion_endpoints: vec![
                "samsung-sign-in-gate",
                "samsung-oauth-token",
                "samsung-user-info",
                "samsung-logout",
            ],
        },
        create_upload_session: CandidateFlow {
            status: "inferred_from_aot_strings",
            endpoint_kind: "room-upload-session",
            default_method: "POST",
            evidence: vec![
                "libapp.so offset 0xfc8f2 contains createUploadSession",
                "libapp.so offset 0x8bed1 contains /room/uploadSession/",
                "libapp.so offset 0x11a6bc contains UploadStorageInfo(uploadUrl:",
                "libapp.so offset 0xd7479 contains requestMyUploadInfo",
            ],
            request_fields: vec![
                field(
                    "roomId",
                    "medium",
                    "SaveRoomRequest(roomId:) and room flow strings",
                ),
                field("files", "low", "files fragments around room flows"),
                field(
                    "pushDevices",
                    "low",
                    "PushDeviceInfo(deviceId:) and pushDevices strings",
                ),
                field("duration", "low", "duration fragment near SaveRoomRequest"),
                field(
                    "password",
                    "low",
                    "password fragments near join/share flows",
                ),
            ],
            response_fields: vec![
                field(
                    "uploadStorageInfo.uploadUrl",
                    "high",
                    "UploadStorageInfo(uploadUrl:)",
                ),
                field(
                    "uploadStorageInfo.thumbnailUploadUrl",
                    "high",
                    "thumbnailUploadUrl fragment near UploadStorageInfo",
                ),
                field(
                    "uploadStorageInfo.pinString",
                    "high",
                    "pinString fragment near UploadStorageInfo",
                ),
                field(
                    "roomId",
                    "low",
                    "roomId is likely present in surrounding room objects",
                ),
            ],
            companion_endpoints: vec![
                "requestMyUploadInfo",
                "room-info",
                "room-invitation",
                "user-profile",
            ],
        },
        get_download_urls: CandidateFlow {
            status: "inferred_from_aot_strings",
            endpoint_kind: "download-urls",
            default_method: "POST",
            evidence: vec![
                "libapp.so offset 0x97912 contains getDownloadUrls",
                "libapp.so offset 0x10f77e contains /downloadUrls",
                "libapp.so offset 0xbf335 contains RoomDownloadUrlsResponse(code:",
            ],
            request_fields: vec![
                field("code", "medium", "response model and download flow naming"),
                field(
                    "password",
                    "low",
                    "RoomEntryEvent.joinGroupRequested(password:)",
                ),
                field("roomId", "low", "room/info and invitation flow overlap"),
            ],
            response_fields: vec![
                field("code", "high", "RoomDownloadUrlsResponse(code:)"),
                field("downloadUrls", "high", "downloadUrls fragment"),
                field("shareURL", "medium", "shareURL fragment"),
                field("expireAt", "medium", "expireAt fragment"),
                field(
                    "creatorProfileImage",
                    "medium",
                    "creatorProfileImage fragment",
                ),
                field("creatorNickName", "medium", "creatorNickName fragment"),
                field("roomCodeInfoList", "medium", "roomCodeInfoList fragment"),
                field("roomStatus", "medium", "roomStatus fragment"),
            ],
            companion_endpoints: vec![
                "room-info",
                "room-info-joined",
                "room-invitation",
                "room-member",
            ],
        },
        request_my_upload_info: CandidateFlow {
            status: "inferred_from_aot_strings",
            endpoint_kind: "room-upload-session",
            default_method: "GET",
            evidence: vec![
                "libapp.so offset 0xd7479 contains requestMyUploadInfo",
                "libapp.so offset 0x8bed1 contains /room/uploadSession/",
                "libapp.so offset 0x11a6bc contains UploadStorageInfo(uploadUrl:",
            ],
            request_fields: vec![
                field(
                    "roomId",
                    "medium",
                    "room/uploadSession path and room model strings",
                ),
                field("userId", "low", "RoomMemberInfoResponse(userId:)"),
            ],
            response_fields: vec![
                field(
                    "uploadStorageInfo",
                    "high",
                    "uploadStorageInfo getter string",
                ),
                field("uploadUrl", "high", "UploadStorageInfo(uploadUrl:)"),
                field("thumbnailUploadUrl", "high", "thumbnailUploadUrl fragment"),
                field("pinString", "high", "pinString fragment"),
            ],
            companion_endpoints: vec!["create-upload-session", "room-info"],
        },
        room_info: CandidateFlow {
            status: "inferred_from_aot_strings",
            endpoint_kind: "room-info",
            default_method: "GET",
            evidence: vec![
                "libapp.so offset 0xa61a2 contains /room/info",
                "libapp.so offset 0x12115e contains /room/info/joined",
                "libapp.so offset 0x55550 contains RoomInfoResponse(roomId:",
            ],
            request_fields: vec![
                field("roomId", "medium", "RoomEntryEvent.infoRequested(roomId:)"),
                field("joinedAt", "low", "SaveRoomRequest(joinedAt)"),
            ],
            response_fields: vec![
                field("roomId", "high", "RoomInfoResponse(roomId:)"),
                field("members", "medium", "members fragment"),
                field("files", "medium", "files fragment"),
                field("pushDevices", "medium", "pushDevices fragment"),
                field("shareURL", "medium", "shareURL fragment"),
                field("status", "medium", "status fragment"),
            ],
            companion_endpoints: vec!["room-member", "room-invitation", "download-urls"],
        },
        invitation_join_flow: CandidateFlow {
            status: "inferred_from_aot_strings",
            endpoint_kind: "room-invitation",
            default_method: "POST",
            evidence: vec![
                "libapp.so offset 0x10c172 contains /room/invitation",
                "libapp.so offset 0x5c9b2 contains joinRoom",
                "libapp.so offset 0x10e1a5 contains cannotJoinRoom",
                "libapp.so offset 0x86fd9 contains InvitationInfoResponse(roomId:",
                "libapp.so offset 0xf6157 contains SaveRoomRequest(roomId:",
            ],
            request_fields: vec![
                field(
                    "roomId",
                    "high",
                    "InvitationInfoResponse and SaveRoomRequest",
                ),
                field(
                    "password",
                    "medium",
                    "RoomEntryEvent.joinGroupRequested(password:)",
                ),
                field("joinedAt", "low", "SaveRoomRequest(joinedAt)"),
            ],
            response_fields: vec![
                field("roomId", "high", "InvitationInfoResponse(roomId:)"),
                field("roomTitle", "medium", "roomTitle fragment"),
                field("cannotJoinRoom", "medium", "cannotJoinRoom string"),
            ],
            companion_endpoints: vec![
                "room-info",
                "room-info-joined",
                "saveRoomInfo",
                "saveRoomInfos",
                "download-urls",
            ],
        },
    }
}

fn field(name: &'static str, confidence: &'static str, source: &'static str) -> FieldShape {
    FieldShape {
        name,
        confidence,
        source,
    }
}
