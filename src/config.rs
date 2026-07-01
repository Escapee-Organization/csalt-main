// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use serde::{Deserialize, Serialize};

// Salt.toml
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageSection {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildSection {
    pub build: String,
    pub edition: String,
    pub compiler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnitVector {
    pub kind: String,
    pub main: String,
    pub src: Vec<String>,
    pub include: Option<Vec<String>>,
    pub compiler_flags: Option<Vec<String>>,
    pub linker_flags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaltToml {
    pub package: PackageSection,
    pub build: BuildSection,
    pub unit: Vec<UnitVector>,
}

// TODO: Validate everything
impl SaltToml {
    pub fn validate(&self) -> Result<(), String> {
        // 1. Ensure the package name isn't blank
        if self.package.name.trim().is_empty() {
            return Err("Package name cannot be empty in Salt.toml".into());
        }

        // 2. Verify target definitions aren't broken
        for target in &self.unit {
            if target.kind != "bin".to_string() && target.kind != "lib".to_string() {
                return Err(format!(
                    "Kind '{}' in Salt.toml should be either 'bin' or 'lib'.",
                    target.kind
                ));
            }
            if target.main.trim().is_empty() && target.kind == "bin".to_string() {
                return Err("Every [[unit]] bin target must specify a 'main' entry file".into());
            }
            if !(target.main.ends_with(".c") || target.main.ends_with(".csal")) {
                return Err(format!(
                    "The main target '{}' must be a valid C source file (.c or .csal)",
                    target.main
                ));
            }
        }

        Ok(())
    }
}

// Salt.lock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    pub shadow_hash: String,
    pub shadow_path: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaltLock {
    pub lock_version: String,
    pub manifest_hash: String,
    pub manifest: SaltToml,
    pub files: std::collections::BTreeMap<String, FileState>,
}
