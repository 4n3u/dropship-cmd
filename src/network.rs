use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use reqwest::{
    StatusCode,
    blocking::Body,
    header::{CONTENT_LENGTH, CONTENT_TYPE, HeaderName, HeaderValue, RANGE},
};
use serde::Serialize;

use crate::{
    http,
    models::{
        CompletedItem, CompletedItemPair, FileDownloadInfo, FileUploadInfo, OgUploadInfo,
        ReportItem, ThumbnailRequest, UploadItem, UploadMetadata,
    },
    protocol,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadSummary {
    pub upload_url: String,
    pub local_path: String,
    pub size: u64,
    pub status: u16,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSummary {
    pub download_url: String,
    pub local_path: String,
    pub resumed: bool,
    pub starting_offset: u64,
    pub final_size: u64,
    pub status: u16,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteUploadSummary {
    pub main_upload: UploadSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_upload: Option<UploadSummary>,
    pub report_status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_status: Option<u16>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteOgUploadSummary {
    pub upload: UploadSummary,
    pub report_status: u16,
}

pub fn upload_file(file_info: &FileUploadInfo) -> Result<UploadSummary> {
    let client = http::data_client()?;
    let file = File::open(&file_info.local_path)
        .with_context(|| format!("failed to open upload file {}", file_info.local_path))?;
    let actual_size = file
        .metadata()
        .map(|meta| meta.len())
        .unwrap_or(file_info.size);

    let response = client
        .put(&file_info.upload_url)
        .header(CONTENT_TYPE, file_info.file_type.as_str())
        .header(CONTENT_LENGTH, actual_size)
        .body(Body::new(file))
        .send()
        .with_context(|| format!("PUT upload failed for {}", file_info.upload_url))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .unwrap_or_else(|_| "<unreadable body>".to_string());
        bail!("upload failed with status {}: {}", status, body);
    }

    Ok(UploadSummary {
        upload_url: file_info.upload_url.clone(),
        local_path: file_info.local_path.clone(),
        size: actual_size,
        status: status.as_u16(),
    })
}

pub fn download_file(file_info: &FileDownloadInfo, resume: bool) -> Result<DownloadSummary> {
    let client = http::data_client()?;
    let path = Path::new(&file_info.local_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory {}", parent.display()))?;
    }

    let existing_len = if resume && path.exists() {
        fs::metadata(path)
            .with_context(|| format!("failed to stat {}", path.display()))?
            .len()
    } else {
        0
    };
    let should_resume = existing_len > 0 && existing_len < file_info.size;

    let mut request = client.get(&file_info.download_url);
    if should_resume {
        request = request.header(RANGE, format!("bytes={existing_len}-"));
    }

    let mut response = request
        .send()
        .with_context(|| format!("GET download failed for {}", file_info.download_url))?;
    let status = response.status();

    let append = should_resume && status == StatusCode::PARTIAL_CONTENT;
    if should_resume && status != StatusCode::PARTIAL_CONTENT && status != StatusCode::OK {
        bail!("resume download failed with unexpected status {}", status);
    }
    if !status.is_success() {
        let body = response
            .text()
            .unwrap_or_else(|_| "<unreadable body>".to_string());
        bail!("download failed with status {}: {}", status, body);
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(!append)
        .append(append)
        .open(path)
        .with_context(|| format!("failed to open output file {}", path.display()))?;

    io::copy(&mut response, &mut file)
        .with_context(|| format!("failed to write {}", path.display()))?;
    file.flush()
        .with_context(|| format!("failed to flush {}", path.display()))?;

    let final_size = fs::metadata(path)
        .with_context(|| format!("failed to stat {}", path.display()))?
        .len();
    if final_size != file_info.size {
        bail!(
            "download size mismatch for {}: expected {}, got {}",
            path.display(),
            file_info.size,
            final_size
        );
    }

    Ok(DownloadSummary {
        download_url: file_info.download_url.clone(),
        local_path: file_info.local_path.clone(),
        resumed: append,
        starting_offset: if append { existing_len } else { 0 },
        final_size,
        status: status.as_u16(),
    })
}

pub fn cancel_transfer(metadata: &UploadMetadata) -> Result<u16> {
    let client = http::auth_client()?;
    let response = client
        .delete(&metadata.cancel_api_url)
        .headers(http::auth_headers(&metadata.headers)?)
        .send()
        .with_context(|| format!("DELETE failed for {}", metadata.cancel_api_url))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .unwrap_or_else(|_| "<unreadable body>".to_string());
        bail!("cancel failed with status {}: {}", status, body);
    }

    Ok(status.as_u16())
}

pub fn report_upload(pin: &str, metadata: &UploadMetadata, items: &[CompletedItem]) -> Result<u16> {
    let client = http::auth_client()?;
    let response = client
        .post(&metadata.complete_api_url)
        .headers(http::json_auth_headers(&metadata.headers)?)
        .json(&ReportItem {
            code: pin.to_string(),
            file_info_list: items.to_vec(),
        })
        .send()
        .with_context(|| format!("POST failed for {}", metadata.complete_api_url))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .unwrap_or_else(|_| "<unreadable body>".to_string());
        bail!("upload report failed with status {}: {}", status, body);
    }

    Ok(status.as_u16())
}

pub fn fetch_thumbnail_url(pin: &str, metadata: &UploadMetadata) -> Result<String> {
    let push = protocol::require_push_message(metadata)?;
    let client = http::auth_client()?;

    let mut headers = http::json_auth_headers(&metadata.headers)?;
    headers.insert(
        HeaderName::from_static("accesstype"),
        HeaderValue::from_static("UPLOAD"),
    );

    let response = client
        .post(&push.thumbnail_api_url)
        .headers(headers)
        .json(&ThumbnailRequest {
            code: pin.to_string(),
            password: push.password.clone().unwrap_or_default(),
        })
        .send()
        .with_context(|| format!("POST failed for {}", push.thumbnail_api_url))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .unwrap_or_else(|_| "<unreadable body>".to_string());
        bail!("thumbnail lookup failed with status {}: {}", status, body);
    }

    let server_files: crate::models::ServerFilesInput = response
        .json()
        .context("failed to parse thumbnail response body")?;
    protocol::pick_thumbnail_url(&server_files.into_vec())
        .ok_or_else(|| anyhow!("thumbnail response did not contain a usable thumbnail URL"))
}

pub fn send_push(pin: &str, metadata: &UploadMetadata, thumbnail_url: Option<&str>) -> Result<u16> {
    let push = protocol::require_push_message(metadata)?;
    let client = http::auth_client()?;

    let mut headers = http::json_auth_headers(&metadata.headers)?;
    headers.insert(
        HeaderName::from_static("aid"),
        HeaderValue::from_str(&push.api_key).context("invalid push api key header value")?,
    );
    headers.insert(
        HeaderName::from_static("projectid"),
        HeaderValue::from_str(&push.project_id).context("invalid push project id header value")?,
    );

    let response = client
        .post(&push.push_api_url)
        .headers(headers)
        .json(&protocol::build_push_request(push, pin, thumbnail_url))
        .send()
        .with_context(|| format!("POST failed for {}", push.push_api_url))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .unwrap_or_else(|_| "<unreadable body>".to_string());
        bail!("push send failed with status {}: {}", status, body);
    }

    Ok(status.as_u16())
}

pub fn execute_upload_item(
    pin: &str,
    metadata: &UploadMetadata,
    upload_item: &UploadItem,
    thumbnail_file_info: Option<&FileUploadInfo>,
    send_push_after_report: bool,
) -> Result<ExecuteUploadSummary> {
    let main_upload = upload_file(&upload_item.file_info)?;
    let thumbnail_upload = thumbnail_file_info.map(upload_file).transpose()?;

    let report = protocol::build_upload_report(upload_item, thumbnail_file_info);
    let report_items = flatten_report_pair(report);
    let report_status = report_upload(pin, metadata, &report_items)?;

    let (thumbnail_url, push_status) = if send_push_after_report {
        let thumb = fetch_thumbnail_url(pin, metadata)?;
        let status = send_push(pin, metadata, Some(&thumb))?;
        (Some(thumb), Some(status))
    } else {
        (None, None)
    };

    Ok(ExecuteUploadSummary {
        main_upload,
        thumbnail_upload,
        report_status,
        thumbnail_url,
        push_status,
    })
}

pub fn execute_og_upload(
    pin: &str,
    metadata: &UploadMetadata,
    og_info: &OgUploadInfo,
    og_local_path: Option<&str>,
) -> Result<ExecuteOgUploadSummary> {
    let temp_path;
    let og_path = match og_local_path {
        Some(path) => PathBuf::from(path),
        None => {
            let generated = write_temp_og_file(og_info)?;
            temp_path = generated;
            temp_path.clone()
        }
    };

    let local_meta = fs::metadata(&og_path)
        .with_context(|| format!("failed to stat OG local path {}", og_path.display()))?;
    let file_info = FileUploadInfo {
        local_path: og_path.to_string_lossy().into_owned(),
        upload_url: og_info.upload_url.clone(),
        name: og_info.file_name.clone(),
        file_type: "image/png".to_string(),
        size: local_meta.len(),
    };

    let upload = upload_file(&file_info)?;
    let completed = protocol::build_og_completed_item(og_info, &file_info);
    let report_status = report_upload(pin, metadata, &[completed])?;

    Ok(ExecuteOgUploadSummary {
        upload,
        report_status,
    })
}

fn flatten_report_pair(pair: CompletedItemPair) -> Vec<CompletedItem> {
    let mut items = vec![pair.file];
    if let Some(thumbnail) = pair.thumbnail {
        items.push(thumbnail);
    }
    items
}

fn write_temp_og_file(og_info: &OgUploadInfo) -> Result<PathBuf> {
    let mut path = std::env::temp_dir();
    path.push("dropship-cmd");
    fs::create_dir_all(&path)
        .with_context(|| format!("failed to create temp dir {}", path.display()))?;
    path.push(&og_info.file_name);

    let mut file = File::create(&path)
        .with_context(|| format!("failed to create temp OG file {}", path.display()))?;
    file.write_all(&og_info.buffer)
        .with_context(|| format!("failed to write temp OG file {}", path.display()))?;
    file.flush()
        .with_context(|| format!("failed to flush temp OG file {}", path.display()))?;

    Ok(path)
}
