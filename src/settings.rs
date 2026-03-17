use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

const DEFAULT_SETTINGS_PATH: &str = "dropship-settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSettings {
    pub use_og_preview: bool,
    pub use_keyword: bool,
    pub default_expired: u8,
    pub sharing_period: String,
    pub key_use_quick_sending: bool,
    pub key_use_auto_copy_link: bool,
    pub key_use_auto_push_message: bool,
    pub key_last_receive_tab: i32,
    pub key_last_sort_filter: i32,
    pub key_last_sort_order: i32,
    pub key_last_list_mode: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DropshipSettings {
    pub local: LocalSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareSettingsView {
    pub preview_content: bool,
    pub address_type: String,
    pub valid_period: String,
    pub valid_hours: u64,
    pub quick_sending: bool,
    pub auto_copy_link: bool,
    pub auto_push_message: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SharingPeriodSpec {
    pub name: &'static str,
    pub hours: u64,
    pub code: u8,
}

const SHARING_PERIODS: &[SharingPeriodSpec] = &[
    SharingPeriodSpec {
        name: "oneHour",
        hours: 1,
        code: 0,
    },
    SharingPeriodSpec {
        name: "threeHour",
        hours: 3,
        code: 1,
    },
    SharingPeriodSpec {
        name: "sixHour",
        hours: 6,
        code: 2,
    },
    SharingPeriodSpec {
        name: "twelveHour",
        hours: 12,
        code: 3,
    },
    SharingPeriodSpec {
        name: "oneDay",
        hours: 24,
        code: 4,
    },
    SharingPeriodSpec {
        name: "threeDay",
        hours: 72,
        code: 5,
    },
    SharingPeriodSpec {
        name: "sevenDay",
        hours: 168,
        code: 6,
    },
    SharingPeriodSpec {
        name: "oneMonth",
        hours: 720,
        code: 7,
    },
];

pub fn default_settings_path() -> PathBuf {
    PathBuf::from(DEFAULT_SETTINGS_PATH)
}

pub fn default_settings() -> DropshipSettings {
    DropshipSettings {
        local: LocalSettings {
            use_og_preview: false,
            use_keyword: false,
            default_expired: 4,
            sharing_period: "oneDay".to_string(),
            key_use_quick_sending: false,
            key_use_auto_copy_link: false,
            key_use_auto_push_message: false,
            key_last_receive_tab: 0,
            key_last_sort_filter: 0,
            key_last_sort_order: 0,
            key_last_list_mode: 1,
        },
    }
}

pub fn load_settings(path: &Path) -> Result<DropshipSettings> {
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read settings file {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse settings file {}", path.display()))
}

pub fn load_settings_optional(path: &Path) -> Result<Option<DropshipSettings>> {
    if !path.exists() {
        return Ok(None);
    }
    load_settings(path).map(Some)
}

pub fn write_settings(path: &Path, settings: &DropshipSettings) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(settings).context("failed to serialize settings JSON")?;
    fs::write(path, bytes)
        .with_context(|| format!("failed to write settings file {}", path.display()))?;
    Ok(())
}

pub fn apply_key_value(settings: &mut DropshipSettings, key: &str, value: &str) -> Result<()> {
    match key {
        "sharing_period" | "valid_period" => {
            let period = period_from_name(value)
                .with_context(|| format!("unknown sharing period: {value}"))?;
            settings.local.sharing_period = period.name.to_string();
            settings.local.default_expired = period.code;
        }
        "use_og_preview" | "preview_content" => settings.local.use_og_preview = parse_bool_like(value)?,
        "use_keyword" => {
            let parsed = parse_bool(value)?;
            settings.local.use_keyword = parsed;
        }
        "address_type" => {
            let parsed = parse_address_type(value)?;
            settings.local.use_keyword = parsed;
        }
        "quick_sending" => settings.local.key_use_quick_sending = parse_bool(value)?,
        "auto_copy_link" => settings.local.key_use_auto_copy_link = parse_bool(value)?,
        "auto_push_message" => settings.local.key_use_auto_push_message = parse_bool(value)?,
        "last_receive_tab" => settings.local.key_last_receive_tab = parse_i32(value)?,
        "last_sort_filter" => settings.local.key_last_sort_filter = parse_i32(value)?,
        "last_sort_order" => settings.local.key_last_sort_order = parse_i32(value)?,
        "last_list_mode" => settings.local.key_last_list_mode = parse_i32(value)?,
        other => bail!("unknown settings key: {other}"),
    }
    Ok(())
}

pub fn share_settings_view(settings: &DropshipSettings) -> Result<ShareSettingsView> {
    Ok(ShareSettingsView {
        preview_content: settings.local.use_og_preview,
        address_type: inferred_address_type(settings).to_string(),
        valid_period: settings.local.sharing_period.clone(),
        valid_hours: resolve_expiry_hours(settings)?,
        quick_sending: settings.local.key_use_quick_sending,
        auto_copy_link: settings.local.key_use_auto_copy_link,
        auto_push_message: settings.local.key_use_auto_push_message,
    })
}

pub fn period_from_name(name: &str) -> Result<SharingPeriodSpec> {
    SHARING_PERIODS
        .iter()
        .find(|period| period.name.eq_ignore_ascii_case(name))
        .copied()
        .context("sharing period name was not recognized")
}

pub fn sharing_periods() -> &'static [SharingPeriodSpec] {
    SHARING_PERIODS
}

pub fn resolve_expiry_hours(settings: &DropshipSettings) -> Result<u64> {
    let period = SHARING_PERIODS
        .iter()
        .find(|period| period.code == settings.local.default_expired)
        .copied()
        .or_else(|| period_from_name(&settings.local.sharing_period).ok())
        .context("settings did not contain a valid sharing period")?;
    Ok(period.hours)
}

pub fn validate_settings(settings: &DropshipSettings) -> Result<()> {
    let period = period_from_name(&settings.local.sharing_period)
        .with_context(|| format!("unknown sharing period {}", settings.local.sharing_period))?;
    if settings.local.default_expired != period.code {
        bail!(
            "local.defaultExpired={} does not match sharing period {}",
            settings.local.default_expired,
            settings.local.sharing_period
        );
    }
    Ok(())
}

fn parse_bool(value: &str) -> Result<bool> {
    value
        .parse::<bool>()
        .with_context(|| format!("invalid bool value: {value}"))
}

fn parse_bool_like(value: &str) -> Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "on" | "yes" | "1" => Ok(true),
        "false" | "off" | "no" | "0" => Ok(false),
        _ => parse_bool(value),
    }
}

fn parse_i32(value: &str) -> Result<i32> {
    value
        .parse::<i32>()
        .with_context(|| format!("invalid i32 value: {value}"))
}

fn parse_address_type(value: &str) -> Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "short" => Ok(true),
        "long" => Ok(false),
        other => bail!("invalid address_type value: {other}"),
    }
}

fn inferred_address_type(settings: &DropshipSettings) -> &'static str {
    if settings.local.use_keyword {
        "short"
    } else {
        "long"
    }
}
