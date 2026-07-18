// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use serde::{Deserialize, Serialize};
use std::collections;
use std::path::PathBuf;
use std::process::Command;

// ================== SALT.TOML ==================

// --------------- DATA STRUCTURES ---------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UnitKinds {
    Lib,
    Dyn,
    Bin,
    /// Pre-compiled library
    ExtLib,
    /// Pre-compiled dynamic library
    ExtDyn,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CEditions {
    C89,
    C99,
    C11,
    C17,
    C23,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BuildSystems {
    #[serde(rename = "cmake")]
    CMake,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CompilerBackend {
    Clang,
    Gcc,
    Zig,
    Msvc,
    ClangCl,
}

// --------- DATA STRUCTURES -> FUNCTIONS ---------
impl std::fmt::Display for CEditions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::C89 => "c89",
            Self::C99 => "c99",
            Self::C11 => "c11",
            Self::C17 => "c17",
            Self::C23 => "c23",
        };
        write!(f, "{}", s)
    }
}

impl BuildSystems {
    pub fn generate_command(&self) -> Command {
        Command::new(self.to_string())
    }
}

impl std::fmt::Display for BuildSystems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::CMake => "cmake",
        };
        write!(f, "{}", s)
    }
}

impl TryFrom<&str> for BuildSystems {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "cmake" => Ok(Self::CMake),
            _ => anyhow::bail!("unknown build system"),
        }
    }
}

impl CompilerBackend {
    pub fn generate_command(&self) -> Command {
        Command::new(self.to_string())
    }
}

impl TryFrom<&str> for CompilerBackend {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "clang" => Ok(Self::Clang),
            "gcc" => Ok(Self::Gcc),
            "zig" => Ok(Self::Zig),
            "cl" => Ok(Self::Msvc),
            "clang-cl" => Ok(Self::ClangCl),
            _ => anyhow::bail!("unknown backend"),
        }
    }
}

impl std::fmt::Display for CompilerBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Clang => "clang",
            Self::Gcc => "gcc",
            Self::Zig => "zig",
            Self::Msvc => "cl",
            Self::ClangCl => "clang-cl",
        };
        write!(f, "{}", s)
    }
}

// --------------- CONFIG SECTIONS ---------------
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PackageSection {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildSection {
    pub build_sys: BuildSystems,
    pub build_sys_ver: String,
    pub build_dir: Option<PathBuf>,
    pub edition: CEditions,
    pub compiler: CompilerBackend,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnitVector {
    pub name: String,
    pub kind: UnitKinds,
    // Main is now implicitly the first file in src.
    pub src: Vec<PathBuf>,
    #[serde(default)]
    pub include: Option<Vec<PathBuf>>,
    #[serde(default)]
    pub deps: Option<Vec<String>>,
    #[serde(default)]
    pub compiler_flags: Option<Vec<String>>,
    #[serde(default)]
    pub linker_flags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaltToml {
    pub package: PackageSection,
    pub build: BuildSection,
    pub unit: Vec<UnitVector>,
}

// ------------------ FUNCTIONS ------------------
impl SaltToml {
    pub fn validate(&self, base_dir: &std::path::Path) -> anyhow::Result<()> {
        // 1. Ensure the package name isn't blank
        if self.package.name.trim().is_empty() {
            anyhow::bail!("Package name cannot be empty in Salt.toml");
        }

        if self.unit.is_empty() {
            anyhow::bail!("At least one unit must be defined in Salt.toml");
        }

        // 2. Verify target definitions aren't broken and are in the correct order
        /*
         * We must ensure that lib and dyn are before bin in the unit vector
         * Also ensure any deps are declared before their use
         */

        let mut seen_bin = false;
        let mut declared_libs: collections::HashSet<String> = collections::HashSet::new();

        for target in &self.unit {
            if target.name.trim().is_empty() {
                anyhow::bail!("Unit name cannot be empty in Salt.toml");
            }
            if declared_libs.contains(target.name.trim()) {
                anyhow::bail!("Duplicate unit name found: '{}'", target.name);
            }

            if target.src.is_empty() {
                anyhow::bail!(
                    "Unit '{}' must specify at least one source file or directory",
                    target.name
                );
            }

            match target.kind {
                UnitKinds::Lib | UnitKinds::Dyn | UnitKinds::ExtLib | UnitKinds::ExtDyn => {
                    if seen_bin {
                        anyhow::bail!(
                            "The {:?} unit '{}' must come before Bin targets",
                            target.kind,
                            target.name
                        );
                    }

                    declared_libs.insert(target.name.trim().to_string());
                }
                UnitKinds::Bin => {
                    seen_bin = true;
                }
            }

            if let Some(deps) = &target.deps {
                for dep in deps {
                    if !declared_libs.contains(dep.trim()) {
                        anyhow::bail!("Dependency '{}' is not declared before use", dep);
                    }
                }
            }

            if let Some(includes) = &target.include {
                for include in includes.iter().map(|i| base_dir.join(i)) {
                    if !include.exists() {
                        anyhow::bail!("Include '{}' does not exist", include.display());
                    }
                    if include.is_file() {
                        anyhow::bail!("Include '{}' is a file, not a directory", include.display());
                    }
                }
            }
        }

        // 3. Validate build system and compiler
        if self.build.edition == CEditions::C89 && self.build.compiler == CompilerBackend::Msvc {
            anyhow::bail!("C89 is not supported with MSVC");
        }

        if self.build.build_sys == BuildSystems::CMake {
            let version = crate::util::normalize_semver(&self.build.build_sys_ver)?;

            let minimum_required = semver::Version::parse("3.15.0")?;
            if version < minimum_required {
                anyhow::bail!(
                    "C-Salt requires CMake version 3.15.0 or higher. Found: {}",
                    self.build.build_sys_ver
                );
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
    pub manifest: SaltToml,
    pub files: collections::BTreeMap<String, FileState>,
}
