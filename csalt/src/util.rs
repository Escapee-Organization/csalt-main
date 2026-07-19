// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

#[cfg(feature = "experimental")]
use sha2::{Digest, Sha256};

#[cfg(feature = "experimental")]
pub fn compute_hash(file_string: &str) -> String {
    let hash_bytes = Sha256::digest(file_string.as_bytes());
    hash_bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

pub fn normalize_semver(version: &str) -> anyhow::Result<semver::Version> {
    let dirty_sys_ver = version.trim();
    if dirty_sys_ver.starts_with(['^', '~', '<', '>', '=']) {
        anyhow::bail!(
            "Ranges like '{}' are not supported yet. Please provide a concrete version (e.g., '3.15').",
            version
        );
    }

    let mut clean_sys_ver = dirty_sys_ver.to_string();
    match clean_sys_ver.split('.').count() {
        1 => clean_sys_ver.push_str(".0.0"),
        2 => clean_sys_ver.push_str(".0"),
        _ => {}
    }

    let normalized_semver = semver::Version::parse(&clean_sys_ver).map_err(|_| {
        anyhow::anyhow!(
            "Invalid semver version: '{}'. Normalized internally to '{}'.",
            version,
            clean_sys_ver
        )
    })?;

    Ok(normalized_semver)
}

pub fn clean_windows_path(path: std::path::PathBuf) -> std::path::PathBuf {
    let path_str = path.to_string_lossy();
    if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
        return std::path::PathBuf::from(stripped);
    }
    path
}
