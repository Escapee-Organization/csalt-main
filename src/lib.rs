// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::cli::CompileArgs;
use crate::config::SaltLock;
use serde_json;
#[cfg(feature = "experimental")]
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(feature = "experimental")]
use std::sync::LockResult;
#[cfg(feature = "experimental")]
use toml;

pub mod cli;
pub mod config;
pub mod fs_utils;
pub mod transpile;

enum CompilerBackend {
    Clang,
    Gcc,
    Zig,
    Msvc,
    ClangCl,
}

impl CompilerBackend {
    fn from_string(s: &str) -> Result<Self, &'static str> {
        match s {
            "clang" => Ok(Self::Clang),
            "gcc" => Ok(Self::Gcc),
            "zig" => Ok(Self::Zig),
            "msvc" => Ok(Self::Msvc),
            "clang-cl" => Ok(Self::ClangCl),
            _ => Err("unknown backend"),
        }
    }

    fn generate_command(&self) -> Command {
        match self {
            Self::Clang => Command::new("clang"),
            Self::Gcc => Command::new("gcc"),
            Self::Zig => Command::new("zig"),
            Self::Msvc => Command::new("cl"),
            Self::ClangCl => Command::new("clang-cl"),
        }
    }
}

#[cfg(feature = "experimental")]
const LOCK_FILE_PATH: &str = "./Salt.lock";
#[cfg(feature = "experimental")]
const LOCK_VERSION: &str = "0.1.0";

// TODO: Implement GitHub release tags and actions
#[cfg(feature = "experimental")]
pub fn update_csalt() -> Result<(), Box<dyn std::error::Error>> {
    println!("[info]\nChecking for updates...");
    self_update::backends::github::Update::configure()
        .repo_owner("Escapee-Organization")
        .repo_name("csalt-main")
        .bin_name("csalt")
        .current_version(env!("CARGO_PKG_VERSION"))
        .show_download_progress(true)
        .build()?;

    println!("[info]\nUpdate completed successfully.");
    Ok(())
}

#[cfg(feature = "experimental")]
fn compute_hash(file_string: &str) -> String {
    let hash_bytes = Sha256::digest(file_string.as_bytes());
    hash_bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

#[cfg(feature = "experimental")]
fn load_or_init_lock(current_toml: &SaltToml) -> Result<SaltLock, Box<dyn std::error::Error>> {
    let lock_path = Path::new(LOCK_FILE_PATH);
    if lock_path.exists() {
        if let Ok(contents) = fs::read_to_string(lock_path) {
            if let Ok(lock) = serde_json::from_str::<SaltLock>(&contents) {
                if lock.manifest == *current_toml {
                    return Ok(lock);
                }
            }
        }
    }

    let manifest_hash = compute_hash(&toml::to_string_pretty(current_toml)?.as_str());

    Ok(SaltLock {
        lock_version: LOCK_VERSION.to_string(),
        manifest_hash,
        manifest: current_toml.clone(),
        files: std::collections::BTreeMap::new(),
    })
}

#[cfg(feature = "experimental")]
pub fn sync_workspace(
    current_toml: &SaltToml,
    detected_source_files: Vec<String>,
) -> Result<std::collections::BTreeMap<String, FileState>, Box<dyn std::error::Error>> {
    let mut lock = load_or_init_lock(current_toml)?;
    let mut updated_files = std::collections::BTreeMap::new();
    let cache_dir = Path::new(".csalt");

    for file_paths in detected_source_files {
        let file = Path::new(&file_paths);
        if !file.exists() {
            continue;
        }
    }

    Ok(updated_files)
}

/*  To compile a project manually (assuming the default mode), we must follow specific steps:
 *  1. Check Salt.lock's cache to see what changed
 *  2. Run the header file engine
 *  3. Transpile the source code
 *  4. Link the transpiled code with the backend compiler
 *  5. Output the compiled binary to build/
 */
pub fn build_manual_project(args: &CompileArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("[info]\nCompiling project...");

    let base_dir = std::env::current_dir()?;
    fs_utils::verify_workspace(&base_dir)?;
    let cache_dir = base_dir.join(".csalt");
    let src_dir = base_dir.join("src");
    // TODO: Consider a more professional output directory
    let out_bin_dir = base_dir.join("build").join("bin");
    fs::create_dir_all(&out_bin_dir)?;

    let lock_file = Path::new(LOCK_FILE_PATH);

    fs_utils::copy_project_files(&base_dir, &cache_dir)?;

    // FIXME: Update file compilation section to use and work with `Salt.lock`.
    let mut files_to_compile: Vec<PathBuf> = Vec::new();
    if src_dir.exists() && src_dir.is_dir() {
        for entry in fs::read_dir(src_dir)? {
            let entry = entry?;
            let path = entry.path();
            // Only capture files that end with the .c extension for now
            if path.is_file() && path.extension().map_or(false, |ext| ext == "c") {
                files_to_compile.push(path);
            }
        }
    }

    if files_to_compile.is_empty() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No files to compile",
        )));
    }

    // TODO: Transpile the input files
    // transpile::transpile(...)?;

    // Read in the target compiler from `Salt.lock`. CLI flag overrides
    let compiler_backend: CompilerBackend = if let Some(backend) = &args.backend {
        CompilerBackend::from_string(backend.as_str())?
    } else {
        // Get `Salt.lock`'s manifest's compiler to then call upon
        if let Ok(lock) = serde_json::from_str::<SaltLock>(fs::read_to_string(&lock_file)?.as_str())
        {
            CompilerBackend::from_string(lock.manifest.build.compiler.as_str())?
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Salt.lock file format",
            )
            .into());
        }
    };
    let mut target_compiler = compiler_backend.generate_command();

    // If the user provided no flags, we are just going to compile the files as-is.
    // Otherwise, the user provided flags will be passed through to the target compiler
    if args.backend_flags.is_empty() {
        let output_executable = out_bin_dir.join(
            base_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("a.out"),
        );
        match compiler_backend {
            CompilerBackend::Clang | CompilerBackend::Gcc => {
                target_compiler.arg("-o");
                target_compiler.arg(&output_executable);
                for file in &files_to_compile {
                    target_compiler.arg(file.to_str().unwrap());
                }
            }
            CompilerBackend::Zig => {
                target_compiler.arg("cc");
                target_compiler.arg("-o");
                target_compiler.arg(&output_executable);
                for file in &files_to_compile {
                    target_compiler.arg(file.to_str().unwrap());
                }
            }
            CompilerBackend::ClangCl | CompilerBackend::Msvc => {
                target_compiler.arg(format!("/Fe:{}", output_executable.to_str().unwrap()));
                for file in &files_to_compile {
                    target_compiler.arg(file.to_str().unwrap());
                }
            }
        }

        let status = target_compiler.current_dir(cache_dir).status()?;
        if !status.success() {
            return Err("Failed to compile".into());
        }
    } else {
        // NOTE: We pass the flags directly to the target compiler as-is, without any processing of our own
        for flag in &args.backend_flags {
            target_compiler.arg(flag);
        }

        let status = target_compiler.status()?;
        if !status.success() {
            return Err("Failed to compile".into());
        }
    }

    Ok(())
}
