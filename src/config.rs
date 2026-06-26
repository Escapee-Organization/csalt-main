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
