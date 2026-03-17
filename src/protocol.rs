use std::collections::BTreeMap;

use anyhow::{Context, Result, bail};

use crate::models::{
    ApnAlert, ApnApsData, ApnData, ApnFcmOptions, ApnHeaders, ApnPayload, CompletedItem,
    CompletedItemPair, FcmBody, FcmData, FcmRequest, FileUploadInfo, HeadersMetadata,
    NotificationMetadata, OgUploadInfo, PushMessageMeta, ServerFileInfo, UploadItem,
    UploadMetadata,
};

pub fn build_upload_headers(headers: &HeadersMetadata) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("Authorization".to_string(), headers.authorization.clone()),
        ("User-Agent".to_string(), headers.user_agent.clone()),
        ("appType".to_string(), headers.app_type.clone()),
        ("device-type".to_string(), headers.device_type.clone()),
        ("auth_url".to_string(), headers.auth_url.clone()),
        ("version".to_string(), headers.version.clone()),
    ])
}

pub fn build_upload_work_input(pin: &str, metadata: &UploadMetadata) -> BTreeMap<String, String> {
    let mut work_input = build_upload_core(pin, metadata);
    if let Some(push) = &metadata.push_message {
        extend_push_meta(&mut work_input, push);
    }
    work_input
}

pub fn build_og_work_input(
    pin: &str,
    og_local_path: &str,
    og_info: &OgUploadInfo,
    metadata: &UploadMetadata,
) -> BTreeMap<String, String> {
    let mut work_input = build_upload_work_input(pin, metadata);
    work_input.insert("ogLocalPath".to_string(), og_local_path.to_string());
    work_input.insert("ogServerId".to_string(), og_info.server_id.clone());
    work_input.insert("ogUploadUrl".to_string(), og_info.upload_url.clone());
    work_input
}

pub fn build_download_work_input(
    pin: &str,
    notification: &NotificationMetadata,
) -> BTreeMap<String, String> {
    let mut work_input = BTreeMap::new();
    work_input.insert("pin".to_string(), pin.to_string());
    extend_notification(&mut work_input, notification);
    work_input
}

pub fn pick_thumbnail_url(server_files: &[ServerFileInfo]) -> Option<String> {
    server_files
        .iter()
        .find(|file| file.file_type.to_ascii_lowercase().contains("image"))
        .or_else(|| server_files.last())
        .map(|file| file.thumbnail_url.clone())
}

pub fn build_upload_report(
    upload_item: &UploadItem,
    thumbnail_file_info: Option<&FileUploadInfo>,
) -> CompletedItemPair {
    CompletedItemPair {
        file: build_completed_item(upload_item, &upload_item.file_info, None, false),
        thumbnail: thumbnail_file_info.map(|file_info| {
            build_completed_item(
                upload_item,
                file_info,
                Some(upload_item.thumbnail_server_id.clone()),
                false,
            )
        }),
    }
}

pub fn build_og_completed_item(
    og_info: &OgUploadInfo,
    file_info: &FileUploadInfo,
) -> CompletedItem {
    CompletedItem {
        file_name: file_info.name.clone(),
        file_type: file_info.file_type.clone(),
        file_size: file_info.size,
        object_id: og_info.server_id.clone(),
        t_object_id: None,
        is_og_image: true,
        extra: None,
    }
}

pub fn build_push_request(
    push: &PushMessageMeta,
    pin: &str,
    thumbnail_url: Option<&str>,
) -> FcmRequest {
    let thumbnail_url = thumbnail_url.unwrap_or_default().to_string();

    FcmRequest {
        fcm: FcmBody {
            topic: push.room_id.clone(),
            data: FcmData {
                room_id: push.room_id.clone(),
                room_title: push.room_title.clone(),
                sender: push.sender.clone(),
                sender_device_id: push.sender_device_id.clone(),
                pin: pin.to_string(),
                files_count: push.file_counts.clone(),
                room_img_url: push.room_img_url.clone(),
                sender_profile_url: push.sender_profile_url.clone(),
                thumbnail_url: thumbnail_url.clone(),
            },
            apns: ApnData {
                headers: ApnHeaders {
                    priority: "10".to_string(),
                    push_type: "alert".to_string(),
                },
                payload: ApnPayload {
                    aps: ApnApsData {
                        alert: ApnAlert {
                            title: push.room_title.clone(),
                            body: push.message.clone(),
                        },
                        sound: "default".to_string(),
                        mutable_content: "1".to_string(),
                    },
                },
                fcm_options: ApnFcmOptions {
                    image: (!thumbnail_url.is_empty()).then_some(thumbnail_url),
                },
            },
        },
    }
}

pub fn require_push_message(metadata: &UploadMetadata) -> Result<&PushMessageMeta> {
    metadata
        .push_message
        .as_ref()
        .context("upload metadata does not contain pushMessage")
}

pub fn require_non_empty_pin(pin: &str) -> Result<()> {
    if pin.trim().is_empty() {
        bail!("pin must not be empty");
    }
    Ok(())
}

fn build_upload_core(pin: &str, metadata: &UploadMetadata) -> BTreeMap<String, String> {
    let mut work_input = BTreeMap::new();
    work_input.insert("pin".to_string(), pin.to_string());
    work_input.extend(build_upload_headers(&metadata.headers));
    work_input.insert(
        "authorization".to_string(),
        metadata.headers.authorization.clone(),
    );
    work_input.insert("appType".to_string(), metadata.headers.app_type.clone());
    work_input.insert("authUrl".to_string(), metadata.headers.auth_url.clone());
    work_input.insert("userAgent".to_string(), metadata.headers.user_agent.clone());
    work_input.insert(
        "deviceType".to_string(),
        metadata.headers.device_type.clone(),
    );
    work_input.insert("version".to_string(), metadata.headers.version.clone());
    work_input.insert(
        "completeApiUrl".to_string(),
        metadata.complete_api_url.clone(),
    );
    work_input.insert("cancelApiUrl".to_string(), metadata.cancel_api_url.clone());
    extend_notification(&mut work_input, &metadata.notification);
    work_input
}

fn extend_notification(
    work_input: &mut BTreeMap<String, String>,
    notification: &NotificationMetadata,
) {
    work_input.insert(
        "notificationUri".to_string(),
        notification.content_uri.clone(),
    );
    work_input.insert("notificationTitle".to_string(), notification.title.clone());
    work_input.insert(
        "notificationProgressChannelId".to_string(),
        notification.progress_channel_id.clone(),
    );
    work_input.insert(
        "notificationProgressContentText".to_string(),
        notification.progress_content_text.clone(),
    );
    work_input.insert(
        "notificationResultChannelId".to_string(),
        notification.result_channel_id.clone(),
    );
    work_input.insert(
        "notificationCompletedContentText".to_string(),
        notification.completed_content_text.clone(),
    );
    work_input.insert(
        "notificationFailedContentText".to_string(),
        notification.failed_content_text.clone(),
    );
    work_input.insert(
        "notificationCancelButtonText".to_string(),
        notification.cancel_button_text.clone(),
    );
}

fn extend_push_meta(work_input: &mut BTreeMap<String, String>, push: &PushMessageMeta) {
    work_input.insert("pushApiUrl".to_string(), push.push_api_url.clone());
    work_input.insert(
        "pushThumbnailApiUrl".to_string(),
        push.thumbnail_api_url.clone(),
    );
    work_input.insert(
        "pushPassword".to_string(),
        push.password.clone().unwrap_or_default(),
    );
    work_input.insert("pushRoomId".to_string(), push.room_id.clone());
    work_input.insert("pushRoomTitle".to_string(), push.room_title.clone());
    work_input.insert("pushSender".to_string(), push.sender.clone());
    work_input.insert(
        "pushSenderDeviceId".to_string(),
        push.sender_device_id.clone(),
    );
    work_input.insert(
        "pushSenderProfile".to_string(),
        push.sender_profile_url.clone(),
    );
    work_input.insert("pushMessage".to_string(), push.message.clone());
    work_input.insert("pushRoomImgUrl".to_string(), push.room_img_url.clone());
    work_input.insert("pushApiKey".to_string(), push.api_key.clone());
    work_input.insert("pushProjectId".to_string(), push.project_id.clone());
    work_input.insert("pushFileCounts".to_string(), push.file_counts.clone());
}

fn build_completed_item(
    upload_item: &UploadItem,
    file_info: &FileUploadInfo,
    thumbnail_object_id: Option<String>,
    is_og_image: bool,
) -> CompletedItem {
    CompletedItem {
        file_name: file_info.name.clone(),
        file_type: file_info.file_type.clone(),
        file_size: file_info.size,
        object_id: upload_item.server_id.clone(),
        t_object_id: thumbnail_object_id,
        is_og_image,
        extra: upload_item.extra.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        DownloadMetadata, NotificationMetadata, PushMessageMeta, UploadItem, UploadMetadata,
    };

    fn sample_notification() -> NotificationMetadata {
        NotificationMetadata {
            content_uri: "content://example/item".to_string(),
            title: "Dropship".to_string(),
            progress_channel_id: "progress".to_string(),
            progress_content_text: "Uploading".to_string(),
            result_channel_id: "result".to_string(),
            completed_content_text: "Done".to_string(),
            failed_content_text: "Failed".to_string(),
            cancel_button_text: "Cancel".to_string(),
        }
    }

    fn sample_headers() -> HeadersMetadata {
        HeadersMetadata {
            authorization: "Bearer token".to_string(),
            app_type: "android".to_string(),
            auth_url: "https://auth.example.test".to_string(),
            user_agent: "Dropship/1.2.7".to_string(),
            device_type: "phone".to_string(),
            version: "1.2.7".to_string(),
        }
    }

    fn sample_push() -> PushMessageMeta {
        PushMessageMeta {
            push_api_url: "https://push.example.test".to_string(),
            thumbnail_api_url: "https://thumb.example.test".to_string(),
            password: Some("1234".to_string()),
            room_id: "room-1".to_string(),
            room_title: "Room".to_string(),
            sender: "Alice".to_string(),
            sender_device_id: "device-1".to_string(),
            sender_profile_url: "https://profile.example.test/alice".to_string(),
            message: "File incoming".to_string(),
            room_img_url: "https://img.example.test/room".to_string(),
            api_key: "api-key".to_string(),
            project_id: "project-id".to_string(),
            file_counts: "2".to_string(),
        }
    }

    fn sample_upload_item() -> UploadItem {
        UploadItem {
            server_id: "server-1".to_string(),
            thumbnail_server_id: "thumb-1".to_string(),
            file_info: FileUploadInfo {
                local_path: "/tmp/file.txt".to_string(),
                upload_url: "https://upload.example.test/file".to_string(),
                name: "file.txt".to_string(),
                file_type: "text/plain".to_string(),
                size: 7,
            },
            thumbnail_uplink: "https://upload.example.test/thumb".to_string(),
            extra: Some(BTreeMap::from([("key".to_string(), "value".to_string())])),
        }
    }

    #[test]
    fn pick_thumbnail_prefers_image_entry() {
        let files = vec![
            ServerFileInfo {
                file_type: "video/mp4".to_string(),
                thumbnail_url: "https://example.test/video".to_string(),
            },
            ServerFileInfo {
                file_type: "image/jpeg".to_string(),
                thumbnail_url: "https://example.test/image".to_string(),
            },
        ];

        let selected = pick_thumbnail_url(&files);

        assert_eq!(selected.as_deref(), Some("https://example.test/image"));
    }

    #[test]
    fn build_push_request_defaults_thumbnail_to_empty_string() {
        let request = build_push_request(&sample_push(), "123456", None);

        assert_eq!(request.fcm.data.pin, "123456");
        assert_eq!(request.fcm.data.thumbnail_url, "");
        assert_eq!(request.fcm.apns.fcm_options.image, None);
    }

    #[test]
    fn build_upload_work_input_contains_push_fields() {
        let metadata = UploadMetadata {
            complete_api_url: "https://complete.example.test".to_string(),
            cancel_api_url: "https://cancel.example.test".to_string(),
            headers: sample_headers(),
            notification: sample_notification(),
            push_message: Some(sample_push()),
        };

        let work_input = build_upload_work_input("654321", &metadata);

        assert_eq!(work_input.get("pin").map(String::as_str), Some("654321"));
        assert_eq!(
            work_input.get("completeApiUrl").map(String::as_str),
            Some("https://complete.example.test")
        );
        assert_eq!(
            work_input.get("pushApiKey").map(String::as_str),
            Some("api-key")
        );
    }

    #[test]
    fn build_upload_report_keeps_main_and_thumbnail_ids_separate() {
        let upload_item = sample_upload_item();
        let thumbnail_file = FileUploadInfo {
            local_path: "/tmp/file-thumb.jpg".to_string(),
            upload_url: "https://upload.example.test/thumb-file".to_string(),
            name: "file-thumb.jpg".to_string(),
            file_type: "image/jpeg".to_string(),
            size: 13,
        };

        let report = build_upload_report(&upload_item, Some(&thumbnail_file));

        assert_eq!(report.file.object_id, "server-1");
        assert_eq!(report.file.t_object_id, None);
        assert_eq!(
            report
                .thumbnail
                .as_ref()
                .and_then(|item| item.t_object_id.as_deref()),
            Some("thumb-1")
        );
    }

    #[test]
    fn build_download_work_input_preserves_notification_fields() {
        let metadata = DownloadMetadata {
            notification: sample_notification(),
        };

        let work_input = build_download_work_input("777777", &metadata.notification);

        assert_eq!(work_input.get("pin").map(String::as_str), Some("777777"));
        assert_eq!(
            work_input
                .get("notificationResultChannelId")
                .map(String::as_str),
            Some("result")
        );
    }
}
