use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct SmokeTargets {
    pub health_endpoint: String,
    pub db_health_endpoint: String,
    pub static_entry: String,
    pub required_release_files: Vec<String>,
    pub required_headers: Vec<(String, String)>,
}

const REQUIRED_HEADER_PREFIX: &str = "SMOKE_REQUIRED_HEADER__";

pub fn smoke_targets_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("deploy")
        .join("smoke.targets")
}

pub fn load_smoke_targets() -> SmokeTargets {
    let path = smoke_targets_path();
    let content = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "读取 smoke 契约文件失败: path={}, error={error}",
            path.display()
        )
    });
    let values = parse_env_like(&content);

    SmokeTargets {
        health_endpoint: required_value(&values, "SMOKE_HEALTH_ENDPOINT"),
        db_health_endpoint: required_value(&values, "SMOKE_DB_HEALTH_ENDPOINT"),
        static_entry: required_value(&values, "SMOKE_STATIC_ENTRY"),
        required_release_files: required_value(&values, "SMOKE_REQUIRED_FILES")
            .split_whitespace()
            .map(ToString::to_string)
            .collect(),
        required_headers: collect_required_headers(&values),
    }
}

fn parse_env_like(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }

            let (key, value) = trimmed.split_once('=')?;
            Some((key.trim().to_string(), strip_quotes(value.trim())))
        })
        .collect()
}

fn required_value(values: &HashMap<String, String>, key: &str) -> String {
    values
        .get(key)
        .cloned()
        .unwrap_or_else(|| panic!("smoke 契约缺少字段: {key}"))
}

fn strip_quotes(value: &str) -> String {
    value.trim_matches('"').trim_matches('\'').to_string()
}

fn collect_required_headers(values: &HashMap<String, String>) -> Vec<(String, String)> {
    let mut headers = values
        .iter()
        .filter_map(|(key, value)| {
            key.strip_prefix(REQUIRED_HEADER_PREFIX)
                .map(|suffix| (suffix.to_ascii_lowercase().replace('_', "-"), value.clone()))
        })
        .collect::<Vec<_>>();

    headers.sort_by(|left, right| left.0.cmp(&right.0));
    headers
}
