// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::cli::CompileArgs;
use crate::config::{CompilerBackend, SaltLock, SaltToml, UnitKinds};
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{collections, fs};

#[cfg(feature = "experimental")]
use std::sync::LockResult;
#[cfg(feature = "experimental")]
use toml;

pub mod cli;
pub mod config;
pub mod fs_utils;
pub mod transpile;

// ----------------- DATA STRUCTURES -----------------

pub struct PreparedUnit {
    pub name: String,
    pub kind: UnitKinds,
    pub src: Vec<PathBuf>,
    pub include: Option<Vec<PathBuf>>,
    pub resolved_deps: Vec<(String, UnitKinds)>,
}

// -------------------- FUNCTIONS --------------------

fn verify_command(command_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    match Command::new(command_name).spawn() {
        Ok(mut child) => {
            // Kill the child! Kill the child!
            let _ = child.kill();
            let _ = child.wait();
            Ok(())
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            // The binary is definitively missing from the system
            Err(Box::new(Error::new(
                ErrorKind::NotFound,
                format!("Compiler '{}' not found", command_name),
            )))
        }
        Err(_) => {
            // It exists, but we ran into a permission/OS blockade (which counts as existing!)
            Ok(())
        }
    }
}

const LOCK_FILE_PATH: &str = "Salt.lock";
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

fn compute_hash(file_string: &str) -> String {
    let hash_bytes = Sha256::digest(file_string.as_bytes());
    hash_bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

fn load_or_init_lock(current_toml: &SaltToml) -> Result<SaltLock, Box<dyn std::error::Error>> {
    let lock_path = Path::new(LOCK_FILE_PATH);
    if lock_path.exists() {
        if let Ok(contents) = fs::read_to_string(lock_path) {
            if !contents.trim().is_empty() {
                if let Ok(lock) = serde_json::from_str::<SaltLock>(&contents) {
                    if lock.manifest == *current_toml {
                        return Ok(lock);
                    }
                }
            }
        }
    }

    Ok(SaltLock {
        lock_version: LOCK_VERSION.to_string(),
        manifest: current_toml.clone(),
        files: std::collections::BTreeMap::new(),
    })
}

#[cfg(feature = "experimental")]
pub fn sync_workspace(
    current_toml: &mut SaltToml,
    base_dir: &Path,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut proposed_files_cache = std::collections::BTreeMap::new();
    let cache_dir = Path::new(".csalt");

    for file_paths in detected_source_files {
        let file = Path::new(&file_paths);
        if !file.exists() {
            continue;
        }
    }

    Ok(updated_files)
}

pub fn prepare_build_plan(
    lock: &SaltLock,
    base_dir: &Path,
) -> Result<Vec<PreparedUnit>, Box<dyn std::error::Error>> {
    let mut plan = Vec::new();

    let mut known_units: HashMap<String, UnitKinds> = collections::HashMap::new();
    for unit in &lock.manifest.unit {
        known_units.insert(unit.name.clone(), unit.kind.clone());
    }

    for unit in &lock.manifest.unit {
        let mut gathered_src_files = Vec::new();

        for src_path in &unit.src {
            let target = if src_path.is_absolute() {
                src_path.clone()
            } else {
                base_dir.join(src_path)
            };

            if target.exists() {
                if target.is_file() {
                    gathered_src_files.push(target);
                } else if target.is_dir() {
                    for entry in fs::read_dir(target)? {
                        let path = entry?.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "c") {
                            gathered_src_files.push(path);
                        }
                    }
                }
            } else {
                return Err(
                    format!("File not found for sources: {}", target.to_string_lossy()).into(),
                );
            }
        }

        // TODO: Refactor directory scanning into a shared path resolution engine to remove duplicate code blocks. FORGIVE ME.

        let mut gathered_include_files = Vec::new();

        if let Some(include) = &unit.include {
            for include_path in include {
                let target = if include_path.is_absolute() {
                    include_path.clone()
                } else {
                    base_dir.join(include_path)
                };

                if target.exists() {
                    if target.is_file() {
                        gathered_include_files.push(target);
                    } else if target.is_dir() {
                        for entry in fs::read_dir(target)? {
                            let path = entry?.path();
                            if path.is_file() && path.extension().map_or(false, |ext| ext == "c") {
                                gathered_include_files.push(path);
                            }
                        }
                    }
                } else {
                    return Err(format!(
                        "File not found for include: {}",
                        target.to_string_lossy()
                    )
                    .into());
                }
            }
        }

        let mut resolved_dependencies = Vec::new();
        if let Some(deps) = &unit.deps {
            for dep in deps {
                if let Some(kind) = known_units.get(dep) {
                    resolved_dependencies.push((dep.clone(), kind.clone()));
                }
            }
        }

        plan.push(PreparedUnit {
            name: unit.name.clone(),
            kind: unit.kind.clone(),
            src: gathered_src_files,
            include: if gathered_include_files.is_empty() {
                None
            } else {
                Some(gathered_include_files)
            },
            resolved_deps: resolved_dependencies,
        });
    }

    Ok(plan)
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

    let salt_toml = fs::read_to_string(base_dir.join("Salt.toml"))?;
    let current_toml: SaltToml = toml::from_str(&salt_toml)?;
    let lock_file = base_dir.join("Salt.lock");

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
            lock.manifest.build.compiler
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Salt.lock file format",
            )
            .into());
        }
    };
    verify_command(&compiler_backend.to_string())?;
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
