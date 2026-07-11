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
// To translate: lib -> library, dyn -> dynamic library, bin -> binary
pub enum UnitKinds {
    Lib,
    Dyn,
    Bin,
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

// TODO: Separate this into the build system AND version. This was done for the MVP to keep things simple, as well as focusing only on keystone versions right after a major policy or edition change
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
impl CEditions {
    pub fn to_string(&self) -> &str {
        match self {
            Self::C89 => "c89",
            Self::C99 => "c99",
            Self::C11 => "c11",
            Self::C17 => "c17",
            Self::C23 => "c23",
        }
    }
}

impl BuildSystems {
    pub fn from_string(s: &str) -> Result<Self, &'static str> {
        match s {
            "cmake" => Ok(Self::CMake),
            _ => Err("unknown build system"),
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            Self::CMake => "cmake",
        }
    }

    pub fn generate_command(&self) -> Command {
        match self {
            Self::CMake => Command::new("cmake"),
        }
    }
}

impl CompilerBackend {
    pub fn from_string(s: &str) -> Result<Self, &'static str> {
        match s {
            "clang" => Ok(Self::Clang),
            "gcc" => Ok(Self::Gcc),
            "zig" => Ok(Self::Zig),
            "cl" => Ok(Self::Msvc),
            "clang-cl" => Ok(Self::ClangCl),
            _ => Err("unknown backend"),
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            Self::Clang => "clang",
            Self::Gcc => "gcc",
            Self::Zig => "zig",
            Self::Msvc => "cl",
            Self::ClangCl => "clang-cl",
        }
    }

    pub fn generate_command(&self) -> Command {
        match self {
            Self::Clang => Command::new("clang"),
            Self::Gcc => Command::new("gcc"),
            Self::Zig => Command::new("zig"),
            Self::Msvc => Command::new("cl"),
            Self::ClangCl => Command::new("clang-cl"),
        }
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
    pub main: PathBuf,

    // Make sure it accepts non-recursive directories AND single files
    pub src: Vec<PathBuf>,
    #[serde(default)]
    pub include: Option<Vec<PathBuf>>,
    #[serde(default)]
    pub deps: Option<Vec<String>>,

    // TODO: Research flags for include directories (include), library search paths, and library files for dynamic translation later.
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
    pub fn validate(&self) -> Result<(), String> {
        // 1. Ensure the package name isn't blank
        if self.package.name.trim().is_empty() {
            return Err("Package name cannot be empty in Salt.toml".into());
        }

        if self.unit.is_empty() {
            return Err("At least one unit must be defined in Salt.toml".into());
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
                return Err(format!("Unit name cannot be empty in Salt.toml"));
            }
            if declared_libs.contains(target.name.trim()) {
                return Err(format!("Duplicate unit name found: '{}'", target.name));
            }
            if !target.main.exists() {
                return Err("Every [[unit]] bin target must specify a 'main' entry file".into());
            }
            if !(target.main.ends_with(".c") || target.main.ends_with(".csal")) {
                return Err(format!(
                    "The main target '{}' must be a valid C source file (.c or .csal)",
                    target.main.to_string_lossy()
                ));
            }
            if target.src.is_empty() {
                return Err(format!(
                    "Unit '{}' must specify at least one source file or directory",
                    target.name
                ));
            }

            match target.kind {
                UnitKinds::Lib | UnitKinds::Dyn => {
                    if seen_bin {
                        return Err(format!(
                            "The {:?} unit '{}' must come before Bin targets",
                            target.kind, target.name
                        ));
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
                        return Err(format!("Dependency '{}' is not declared before use", dep));
                    }
                }
            }

            if let Some(includes) = &target.include {
                for include in includes {
                    if include.is_file() {
                        return Err(format!(
                            "Include '{}' is a file, not a directory",
                            include.display()
                        ));
                    }
                }
            }
        }

        // 3. Validate build system and compiler
        if self.build.edition == CEditions::C89 {
            if self.build.compiler == CompilerBackend::Msvc {
                return Err("C89 is not supported with MSVC".to_string());
            }
        }

        if self.build.build_sys == BuildSystems::CMake {
            if self.build.build_sys_ver != "3.15" || self.build.build_sys_ver != "3.28" {
                return Err("CMake version must be a baseline/milestone version".to_string());
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
