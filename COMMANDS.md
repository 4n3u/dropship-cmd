# Command Reference

This file describes the commands exposed by `dropship-cmd`.

## Conventions

- Commands print JSON to stdout unless noted otherwise.
- Arguments such as `--request`, `--metadata`, `--headers`, `--input`, and `--token-info` expect JSON file paths.
- Commands with `--session` use `dropship-session.json` by default when the flag is omitted.
- `Confirmed` means the flow has been exercised against the live web path.
- `Partial` means the endpoint exists and the command runs, but the product meaning is still being reconstructed.
- `Candidate` means the command is mainly for reverse engineering and request-shape experiments.

## Recommended Workflow

Share-settings and some high-level receive/room commands are implemented as manual top-level commands. `cargo run -- --help` now appends a short manual-command section, and the detailed behavior is documented here.

### `login`

- Status: Confirmed
- Purpose: Open the Samsung web sign-in flow, exchange the callback code, fetch user data, and save a session file.
- Main options:
  - `--callback-url`: provide the callback URL directly instead of pasting it interactively
  - `--session-out`: write the session JSON to a custom path
  - `--no-browser`: print the URL without opening a browser automatically
  - `--client-id`: override the web `client_id`
  - `--redirect-uri`: override the callback URL
  - `--sign-in-base-url`: override the Samsung account host
  - `--issue-token-url`: override the token exchange endpoint
  - `--user-info-url`: override the web user info endpoint
  - `--user-url`: override the Dropship user endpoint
  - `--stay-signed-in`: send `staySignedIn=true` in the token exchange request

### `whoami`

- Status: Confirmed
- Purpose: Load the saved session and fetch current user/profile details.
- Main options:
  - `--session`: use a session file other than `dropship-session.json`
- Notes:
  - if the access token is rejected, the CLI refreshes the session once and retries automatically

### `session-check`

- Status: Confirmed
- Purpose: Validate the current session by calling the confirmed web user endpoints.
- Main options:
  - `--session`: use a custom session file
- Notes:
  - if the access token is rejected, the CLI refreshes the session once and retries automatically

### `refresh-session`

- Status: Experimental
- Purpose: Try the web `refreshToken` flow using the stored refresh token and rewrite `dropship-session.json`.
- Main options:
  - `--session`: use a custom session file
- Notes:
  - if the backend rejects the stored refresh token, rerun `login`
  - this command is implemented as a manual top-level command and does not appear in generated Clap help

### `usage`

- Status: Confirmed
- Purpose: Fetch daily usage information from the web API.
- Main options:
  - `--session`: use a custom session file
- Notes:
  - if the access token is rejected, the CLI refreshes the session once and retries automatically

### `upload-history`

- Status: Confirmed
- Purpose: Fetch recent upload/share history for the current account.
- Main options:
  - `--session`: use a custom session file
- Notes:
  - if the access token is rejected, the CLI refreshes the session once and retries automatically

### `joined-rooms`

- Status: Confirmed
- Purpose: List joined room entries returned by the current web API.
- Main options:
  - `--session`: use a custom session file
- Notes:
  - if the access token is rejected, the CLI refreshes the session once and retries automatically

### `settings-show`

- Status: Confirmed local persistence
- Purpose: Show the stored local share settings.
- Main options:
  - `--settings-file <PATH>`: optional settings file override
- Alias:
  - `share-settings-show`

### `settings-init`

- Status: Confirmed local persistence
- Purpose: Create `dropship-settings.json` from the app's inferred local share-setting defaults.
- Main options:
  - `--settings-file <PATH>`: optional settings file override
- Notes:
  - local defaults are based on the app keys recovered from Flutter AOT
  - sharing periods are normalized to the app names `oneHour`, `threeHour`, `sixHour`, `twelveHour`, `oneDay`, `threeDay`, `sevenDay`, `oneMonth`
- Alias:
  - `share-settings-init`

### `settings-set`

- Status: Confirmed local persistence
- Purpose: Update one share-setting value in `dropship-settings.json`.
- Main options:
  - `--settings-file <PATH>`: optional settings file override
  - `--key <NAME>`: settings key name
  - `--value <VALUE>`: string value parsed according to the key type
- Recommended keys:
  - `preview_content`
  - `address_type`
  - `valid_period`
- Values:
  - `preview_content`: `on|off|true|false`
  - `address_type`: `short|long`
  - `valid_period`: `oneHour|threeHour|sixHour|twelveHour|oneDay|threeDay|sevenDay|oneMonth`
- Advanced keys:
  - `sharing_period`
  - `use_og_preview`
  - `use_keyword`
  - `quick_sending`
  - `auto_copy_link`
  - `auto_push_message`
  - `last_receive_tab`
  - `last_sort_filter`
  - `last_sort_order`
  - `last_list_mode`
- Alias:
  - `share-settings-set`

### `share-file`

- Status: Confirmed
- Purpose: Create a web upload session, upload one or more local files to the presigned URLs, and call upload completion.
- Main options:
  - `--path`: one or more local file paths to share
  - `--session`: use a custom session file
  - `--expiry`: override the share lifetime in hours; when omitted, the CLI uses `dropship-settings.json` and then falls back to `24`
  - `--keyword`: override the account keyword used in the generated share code
  - `--share-message`: message sent in the upload-session request
  - `--description`: description attached to each file item
  - `--password`: optional share password
  - `--use-keyword`: enable keyword-based sharing behavior
  - `--is-only-me`: request an only-me style share
  - `--no-profile`: omit the profile URL from the request
  - `--use-extra-size`: request extra-size behavior if the account supports it
- Notes:
  - if `dropship-settings.json` exists, `share-file` uses it for local defaults such as address type and valid period
  - keyword, nickname, profile, and similar account data come from the current login session or explicit CLI flags, not from `dropship-settings.json`
  - if the access token is rejected, the CLI refreshes the session once and retries automatically

### `receive-info`

- Status: Confirmed
- Purpose: Resolve a public share code or share URL through `/downloadUrls` and print file metadata without downloading.
- Main options:
  - `--code`: bare code such as `keyword123456`, six-digit code, or a full share URL such as `https://g2sh.me/keyword/123456`
  - `--password`: optional password for protected shares
  - `--session`: optional authenticated session file; omitted by default for public receive
  - `--access-type`: request access type sent to `/downloadUrls`, default `VALIDATE`
- Notes:
  - the output includes `requiresPassword`, `fileCount`, and `suggestedOutputDir`
  - when the share looks protected and no password was supplied, the output includes a `nextAction` hint

### `receive`

- Status: Confirmed
- Purpose: Resolve a public share code or share URL, then download every file into a local directory.
- Main options:
  - `--code`: bare code or full share URL
  - `--password`: optional password for protected shares
  - `--output-dir`: target directory; omitted value defaults to `received/<code>`
  - `--session`: optional authenticated session file
  - `--access-type`: request access type sent to `/downloadUrls`, default `VALIDATE`
  - `--resume`: `true|false`, default `true`
  - `--overwrite`: `true|false`, default `false`
- Notes:
  - the current implementation derives local filenames from the server `fileName`
  - share URLs such as `https://g2sh.me/keyword/123456` are normalized automatically
  - existing files with a matching size are skipped by default
  - conflicting file names are resolved automatically instead of clobbering earlier files
  - when the server marks a share as secure and `--password` is omitted, the CLI prompts for the password interactively
  - the output includes a `plan` section showing whether each file will be downloaded, overwritten, or skipped

### `create-upload-session`

- Status: Confirmed
- Purpose: Low-level web upload-session call without performing the actual file upload.
- Main options:
  - `--request`: path to a `GoodlockWebUploadSessionRequest` JSON file
  - `--session`: use a custom session file
- Use when:
  - you want the raw `code`, presigned URLs, `objectId`s, and expiry before uploading files yourself

### `complete-upload-session`

- Status: Confirmed
- Purpose: Low-level completion call for an already uploaded web session.
- Main options:
  - `--request`: path to a `GoodlockWebUploadSessionCompleteRequest` JSON file
  - `--session`: use a custom session file

## Room And Invitation Commands

### `save-room`

- Status: Partial
- Purpose: Post to `/room/info` using the current authenticated session.
- Main options:
  - `--request`: path to a `GoodlockSaveRoomRequest` JSON file
  - `--session`: use a custom session file
- Note:
  - if `roomId` is an empty string, the backend creates a new room and returns a generated `roomId`
  - if `roomId` points at a missing room, the backend returns `NOT_FOUND_ROOM`
- Alias:
  - `create-room`
- Notes:
  - if the access token is rejected, the CLI refreshes the session once and retries automatically
  - `create-room --title <TEXT>` is the recommended direct form for new room creation; `--request` is still supported for low-level control

### `room-info`

- Status: Partial
- Purpose: Fetch room information for an existing `roomId`.
- Main options:
  - `--room-id`: room identifier
  - `--session`: use a custom session file
- Notes:
  - if the access token is rejected, the CLI refreshes the session once and retries automatically
  - the backend returns `NOT_FOUND_INVITATION` when the room exists but no active invitation is attached
  - the CLI maps that case to `invitationActive=false` instead of a hard failure

### `room-members`

- Status: Partial
- Purpose: Fetch members for an existing `roomId`.
- Main options:
  - `--room-id`: room identifier
  - `--session`: use a custom session file
- Notes:
  - if the access token is rejected, the CLI refreshes the session once and retries automatically

### `room-invite`

- Status: Partial
- Purpose: Post to `/room/invitation` for an existing room using the current authenticated session.
- Main options:
  - `--request`: path to a `GoodlockInvitationRequest` JSON file
  - `--session`: use a custom session file
- Alias:
  - `create-invitation`
- Notes:
  - if the access token is rejected, the CLI refreshes the session once and retries automatically
  - `room-invite --room-id <ROOM_ID> --expiry <HOURS>` is the recommended direct form

### `delete-room-invitation`

- Status: Confirmed
- Purpose: Delete the active invitation for a room via `DELETE /room/{roomId}/invitation`.
- Main options:
  - `--room-id`: room identifier
  - `--session`: use a custom session file
- Notes:
  - after deletion, `room-info` reports `invitationActive=false` until a new invitation is created
  - if the access token is rejected, the CLI refreshes the session once and retries automatically

### `join-room`

- Status: Partial
- Purpose: Join a room via `POST /room/member`.
- Main options:
  - `--room-id`: room identifier
  - `--password`: optional room password
  - `--session`: use a custom session file
- Notes:
  - if the access token is rejected, the CLI refreshes the session once and retries automatically

### `room-codes`

- Status: Partial
- Purpose: Fetch the recent room code list from `GET /room/{roomId}/codes`.
- Main options:
  - `--room-id`: room identifier
  - `--size`: number of codes to request, default `10`
  - `--session`: use a custom session file
- Notes:
  - if the access token is rejected, the CLI refreshes the session once and retries automatically
  - live testing showed this returns a `roomCodeInfoList` wrapper, which may remain empty even when an invitation code exists

### `room-download-urls`

- Status: Partial
- Purpose: Call `/room/{roomId}/downloadUrls` with a room-scoped request body.
- Main options:
  - `--room-id`: room identifier
  - `--request`: path to a `GoodlockDownloadUrlsRequest` JSON file
  - `--session`: use a custom session file
- Notes:
  - if the access token is rejected, the CLI refreshes the session once and retries automatically

## Android Transfer Helper Commands

These commands help reproduce the native Android transfer-agent behavior.

### `build-upload-work-input`

- Status: Helper
- Purpose: Build WorkManager-style upload input from `UploadMetadata`.
- Required arguments:
  - `--pin`: transfer code or pin
  - `--metadata`: path to `UploadMetadata` JSON

### `build-og-work-input`

- Status: Helper
- Purpose: Build WorkManager-style OG upload input.
- Required arguments:
  - `--pin`: transfer code or pin
  - `--metadata`: path to `UploadMetadata` JSON
  - `--og-upload-info`: path to `OgUploadInfo` JSON
  - `--og-local-path`: local OG preview path

### `build-download-work-input`

- Status: Helper
- Purpose: Build WorkManager-style download input from `DownloadMetadata`.
- Required arguments:
  - `--pin`: transfer code or pin
  - `--metadata`: path to `DownloadMetadata` JSON

### `build-push-request`

- Status: Helper
- Purpose: Build the push payload used by the Android upload flow.
- Required arguments:
  - `--pin`: transfer code or pin
  - `--metadata`: path to `UploadMetadata` JSON
- Optional arguments:
  - `--thumbnail-url`: override the thumbnail URL included in the push request

### `build-upload-report`

- Status: Helper
- Purpose: Build the upload completion report item list.
- Required arguments:
  - `--upload-item`: path to `UploadItem` JSON
- Optional arguments:
  - `--thumbnail-file-info`: path to thumbnail `FileUploadInfo` JSON

### `build-og-completed-item`

- Status: Helper
- Purpose: Build the OG completion item used after OG preview upload.
- Required arguments:
  - `--og-upload-info`: path to `OgUploadInfo` JSON
  - `--file-info`: path to `FileUploadInfo` JSON

### `build-headers`

- Status: Helper
- Purpose: Build upload headers from `HeadersMetadata`.
- Required arguments:
  - `--headers`: path to `HeadersMetadata` JSON

### `pick-thumbnail`

- Status: Helper
- Purpose: Select a thumbnail candidate from server file metadata.
- Required arguments:
  - `--server-files`: path to `ServerFilesInput` JSON

### `upload-file`

- Status: Helper
- Purpose: Upload a local file to a presigned URL described by `FileUploadInfo`.
- Required arguments:
  - `--file-info`: path to `FileUploadInfo` JSON

### `execute-upload-item`

- Status: Helper
- Purpose: Run the Android-style upload item flow, including completion reporting and optional push.
- Required arguments:
  - `--pin`: transfer code or pin
  - `--metadata`: path to `UploadMetadata` JSON
  - `--upload-item`: path to `UploadItem` JSON
- Optional arguments:
  - `--thumbnail-file-info`: path to thumbnail `FileUploadInfo` JSON
  - `--send-push`: send the push step after upload

### `execute-og-upload`

- Status: Helper
- Purpose: Run the OG upload flow.
- Required arguments:
  - `--pin`: transfer code or pin
  - `--metadata`: path to `UploadMetadata` JSON
  - `--og-upload-info`: path to `OgUploadInfo` JSON
- Optional arguments:
  - `--og-local-path`: local OG preview path; omitted path can be synthesized from `OgUploadInfo.buffer`

### `download-file`

- Status: Helper
- Purpose: Download a file using `FileDownloadInfo`.
- Required arguments:
  - `--file-info`: path to `FileDownloadInfo` JSON
- Optional arguments:
  - `--resume`: use the resume behavior exposed by the current CLI

### `cancel-transfer`

- Status: Helper
- Purpose: Call the transfer cancel endpoint from `UploadMetadata`.
- Required arguments:
  - `--metadata`: path to `UploadMetadata` JSON

### `fetch-thumbnail-url`

- Status: Helper
- Purpose: Call the transfer thumbnail lookup endpoint.
- Required arguments:
  - `--pin`: transfer code or pin
  - `--metadata`: path to `UploadMetadata` JSON

### `send-push`

- Status: Helper
- Purpose: Send the Android-style push payload from `UploadMetadata`.
- Required arguments:
  - `--pin`: transfer code or pin
  - `--metadata`: path to `UploadMetadata` JSON
- Optional arguments:
  - `--thumbnail-url`: override the thumbnail URL used in the push request

## Endpoint And Model Discovery Commands

### `show-known-endpoints`

- Status: Reference
- Purpose: Print hard-coded Samsung and Dropship endpoint fragments currently known to the project.

### `build-endpoint`

- Status: Reference
- Purpose: Build a concrete endpoint URL from a base URL and endpoint kind.
- Required arguments:
  - `--base-url`: host or base URL
  - `--kind`: endpoint kind string such as `room-info` or `samsung-sign-in-gate`
- Optional arguments:
  - `--suffix`: extra path segment appended to the endpoint

### `show-candidate-models`

- Status: Candidate
- Purpose: Print the candidate response models recovered from reverse engineering.

### `show-candidate-flows`

- Status: Candidate
- Purpose: Print the current best-guess flow summaries for unresolved APIs.

### `derive-endpoints-from-token`

- Status: Reference
- Purpose: Derive likely backend endpoints from a `SamsungTokenInfo` JSON file.
- Required arguments:
  - `--token-info`: path to `SamsungTokenInfo` JSON

## Web Login And Auth Helper Commands

### `build-samsung-sign-in-gate-url`

- Status: Candidate
- Purpose: Build a Samsung sign-in URL from a JSON request object.
- Required arguments:
  - `--base-url`: Samsung account host
  - `--request`: path to `SamsungSignInGateRequestCandidate` JSON

### `build-samsung-auth-form`

- Status: Candidate
- Purpose: Build a token exchange form for Samsung auth endpoints.
- Required arguments:
  - `--base-url`: auth base URL
  - `--kind`: `samsung-issue-access-token` or `samsung-oauth-token`
  - `--request`: path to `SamsungIssueAccessTokenRequestCandidate` JSON

### `parse-received-url`

- Status: Candidate
- Purpose: Parse a callback URL into structured fields such as `code`, `state`, and `error`.
- Required arguments:
  - `--url`: callback URL to parse

### `classify-dropship-link`

- Status: Candidate
- Purpose: Classify a Dropship deep link or public share URL.
- Required arguments:
  - `--url`: URL to classify

### `evaluate-web-login-state`

- Status: Candidate
- Purpose: Evaluate login progress from a structured snapshot.
- Required arguments:
  - `--input`: path to `WebLoginStateInput` JSON

### `request-samsung-token-candidate`

- Status: Candidate
- Purpose: Perform a token exchange using a candidate Samsung request form.
- Required arguments:
  - `--url`: token endpoint URL
  - `--request`: path to `SamsungIssueAccessTokenRequestCandidate` JSON

### `request-samsung-refresh-token-candidate`

- Status: Candidate
- Purpose: Perform a refresh-token exchange using a candidate Samsung request form.
- Required arguments:
  - `--url`: token endpoint URL
  - `--request`: path to `SamsungRefreshTokenRequestCandidate` JSON

### `request-goodlock-web-issue-access-token`

- Status: Confirmed
- Purpose: Call the confirmed Good Lock web token exchange endpoint directly.
- Required arguments:
  - `--url`: issue-access-token endpoint URL
  - `--request`: path to `GoodlockWebIssueAccessTokenRequest` JSON
- Optional arguments:
  - `--headers`: path to `GoodlockWebHeadersInput` JSON

### `fetch-goodlock-web-user-info`

- Status: Confirmed
- Purpose: Call the confirmed Good Lock web `userInfo` endpoint directly.
- Required arguments:
  - `--url`: user info endpoint URL
- Optional arguments:
  - `--headers`: path to `GoodlockWebHeadersInput` JSON

### `fetch-goodlock-web-user`

- Status: Confirmed
- Purpose: Call the confirmed Dropship `/user` endpoint directly.
- Required arguments:
  - `--url`: user endpoint URL
- Optional arguments:
  - `--headers`: path to `GoodlockWebHeadersInput` JSON

### `complete-goodlock-web-login-confirmed`

- Status: Confirmed
- Purpose: Run confirmed web login steps in one command: token exchange, user info fetch, and optional user fetch.
- Required arguments:
  - `--issue-token-url`: issue-access-token endpoint
  - `--issue-token-request`: path to `GoodlockWebIssueAccessTokenRequest` JSON
  - `--user-info-url`: web user info endpoint
- Optional arguments:
  - `--user-url`: optional Dropship `/user` endpoint
  - `--headers`: path to `GoodlockWebHeadersInput` JSON

### `fetch-samsung-user-profile-candidate`

- Status: Candidate
- Purpose: Fetch a Samsung-style user profile using a stored token.
- Required arguments:
  - `--url`: profile endpoint URL
  - `--token-info`: path to `SamsungTokenInfo` JSON

### `complete-web-login-candidate`

- Status: Candidate
- Purpose: Run the broader candidate login flow from callback URL through profile fetch.
- Required arguments:
  - `--input`: path to `CompleteWebLoginCandidateInput` JSON

## Probe Commands

### `probe-endpoint`

- Status: Candidate
- Purpose: Send a generic authenticated request to a known endpoint kind and optionally parse the response into a candidate model.
- Required arguments:
  - `--token-info`: path to `SamsungTokenInfo` JSON
  - `--kind`: endpoint kind string
  - `--method`: HTTP method
- Optional arguments:
  - `--body`: JSON body file
  - `--session-headers`: path to `SessionHeadersInput` JSON
  - `--suffix`: extra path segment appended to the endpoint
  - `--extract`: candidate parser name, default `auto`

### `request-create-upload-session-candidate`

- Status: Candidate
- Purpose: Call the current best-guess Android-style upload-session request with `HeadersMetadata`.
- Required arguments:
  - `--url`: target endpoint URL
  - `--headers`: path to `HeadersMetadata` JSON
  - `--request`: path to `CreateUploadSessionRequestCandidate` JSON

### `request-get-download-urls-candidate`

- Status: Candidate
- Purpose: Call the current best-guess Android-style download-URLs request with `HeadersMetadata`.
- Required arguments:
  - `--url`: target endpoint URL
  - `--headers`: path to `HeadersMetadata` JSON
  - `--request`: path to `DownloadUrlsRequestCandidate` JSON

## JSON Input Files

- The repository no longer ships bundled `examples/` JSON files.
- Commands that take `--request`, `--metadata`, `--headers`, `--input`, `--body`, or `--token-info` expect you to provide your own JSON file.
- Use the Rust struct names mentioned in this document as the shape reference when preparing those files.
