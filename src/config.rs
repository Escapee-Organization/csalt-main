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
    pub main: String,
    pub shared_src: Vec<String>,
    pub shared_include: Vec<String>,
    pub custom: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BinVector {
    pub main: String,
    pub src: Vec<String>,
    pub include: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaltToml {
    pub package: PackageSection,
    pub build: BuildSection,
    pub bin: Vec<BinVector>,
}

impl SaltToml {
    pub fn validate(&self) -> Result<(), String> {
        // 1. Ensure the package name isn't blank
        if self.package.name.trim().is_empty() {
            return Err("Package name cannot be empty in Salt.toml".into());
        }

        // 2. Verify target definitions aren't broken
        for target in &self.bin {
            if target.main.trim().is_empty() {
                return Err("Every [[bin]] target must specify a 'main' entry file".into());
            }
            if !target.main.ends_with(".c") {
                return Err(format!(
                    "The main target '{}' must be a valid C source file (.c)",
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
    pub fingerprint: String,
    pub shadow_path: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaltLock {
    pub lock_version: String,
    pub manifest: SaltToml,
    pub files: std::collections::BTreeMap<String, FileState>,
}
