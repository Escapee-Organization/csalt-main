// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::cli::CompileArgs;
use crate::config::{CEditions, CompilerBackend, SaltLock, SaltToml, UnitKinds};
use serde_json;
#[cfg(feature = "experimental")]
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

fn load_or_init_lock(current_toml: &SaltToml) -> Result<SaltLock, Box<dyn std::error::Error>> {
    let lock_path = Path::new("Salt.lock");
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

pub fn emit_project(base_dir: &Path, cache_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs_utils::verify_workspace(&base_dir)?;
    fs_utils::copy_project_files(&base_dir, &cache_dir)?;
    Ok(())
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
                    gathered_include_files.push(target);
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

    let salt_toml_str = fs::read_to_string(base_dir.join("Salt.toml"))?;
    let current_toml: SaltToml = toml::from_str(&salt_toml_str)?;

    let lock = load_or_init_lock(&current_toml)?;
    emit_project(&base_dir, &cache_dir)?;

    // TODO: Consider a more professional output directory
    let out_bin_dir = base_dir.join(match &lock.manifest.build.build_dir {
        Some(dir) => dir,
        None => Path::new("build/"),
    });
    fs::create_dir_all(&out_bin_dir)?;

    // Read in the target compiler from `Salt.lock`. CLI flag overrides
    let compiler_backend: CompilerBackend = if let Some(backend) = &args.backend {
        CompilerBackend::from_string(backend.as_str())?
    } else {
        lock.manifest.build.compiler.clone()
    };

    verify_command(&compiler_backend.to_string())?;
    let build_plan = prepare_build_plan(&lock, &base_dir)?;
    let debug_on = false; // Disable debug output from existing, but keep the code so it can be enabled later

    for unit in build_plan {
        println!(
            "[info] Processing target unit: {} ({:?})",
            unit.name, unit.kind
        );

        let mut target_compiler = compiler_backend.generate_command();
        let output_executable = if cfg!(target_os = "windows") {
            out_bin_dir.join(&unit.name).with_extension("exe")
        } else {
            out_bin_dir.join(&unit.name)
        };
        let obj_ext = match compiler_backend {
            CompilerBackend::Gcc | CompilerBackend::Zig | CompilerBackend::Clang => "o",
            CompilerBackend::Msvc | CompilerBackend::ClangCl => "obj",
        };

        // --- DEBUG ---
        let mut debug_output_text = String::new();
        if debug_on {
            debug_output_text.push_str("[DEBUG COMMAND] ");
            debug_output_text.push_str(&compiler_backend.to_string());
        }

        match compiler_backend {
            CompilerBackend::Gcc | CompilerBackend::Clang | CompilerBackend::Zig => {
                if compiler_backend == CompilerBackend::Zig {
                    target_compiler.arg("cc");

                    // --- DEBUG ---
                    if debug_on {
                        debug_output_text.push_str(" cc");
                    }
                }

                match lock.manifest.build.edition {
                    CEditions::C89 => target_compiler.arg("-std=c89"),
                    CEditions::C99 => target_compiler.arg("-std=c99"),
                    CEditions::C11 => target_compiler.arg("-std=c11"),
                    CEditions::C17 => target_compiler.arg("-std=c17"),
                    CEditions::C23 => target_compiler.arg("-std=c23"),
                };

                match unit.kind {
                    UnitKinds::Bin => {
                        target_compiler.arg("-o").arg(&output_executable);

                        // --- DEBUG ---
                        if debug_on {
                            debug_output_text.push_str(" -o ");
                            debug_output_text.push_str(&output_executable.to_string_lossy());
                        }
                    }
                    UnitKinds::Dyn => {
                        let dyn_ext = if cfg!(target_os = "windows") {
                            "dll"
                        } else if cfg!(target_os = "macos") {
                            "dylib"
                        } else {
                            "so"
                        };
                        let out_dyn = cache_dir.join(format!("lib{}.{}", unit.name, dyn_ext));
                        target_compiler
                            .arg("-shared")
                            .arg("-fPIC")
                            .arg("-o")
                            .arg(&out_dyn);

                        // --- DEBUG ---
                        if debug_on {
                            debug_output_text.push_str(" -shared -fPIC -o ");
                            debug_output_text.push_str(&out_dyn.to_string_lossy());
                        }
                    }
                    UnitKinds::Lib => {
                        // Static libraries are archives of individual object (.o) files
                        // We instruct GCC to compile source targets to relocatable objects first (-c)
                        target_compiler.arg("-c");

                        // --- DEBUG ---
                        if debug_on {
                            debug_output_text.push_str(" -c");
                        }
                    }
                }
            }
            CompilerBackend::Msvc | CompilerBackend::ClangCl => {
                match lock.manifest.build.edition {
                    CEditions::C89 => {}
                    CEditions::C99 => {}
                    CEditions::C11 => {
                        target_compiler.arg("/std:c11");
                    }
                    CEditions::C17 => {
                        target_compiler.arg("/std:c17");
                    }
                    CEditions::C23 => {
                        target_compiler.arg("/std:clatest");
                    }
                };

                match unit.kind {
                    UnitKinds::Bin => {
                        target_compiler.arg(format!("/Fe:{}", output_executable.to_str().unwrap()));

                        // --- DEBUG ---
                        if debug_on {
                            debug_output_text.push_str(
                                format!("/Fe:{}", output_executable.to_str().unwrap()).as_str(),
                            );
                        }
                    }
                    UnitKinds::Dyn => {
                        let out_dyn = cache_dir.join(format!("{}.dll", unit.name));
                        target_compiler
                            .arg("/LD")
                            .arg(format!("/Fe:{}", out_dyn.to_str().unwrap()));

                        // --- DEBUG ---
                        if debug_on {
                            debug_output_text.push_str(
                                format!("/LD /Fe:{}", out_dyn.to_str().unwrap()).as_str(),
                            );
                        }
                    }
                    UnitKinds::Lib => {
                        target_compiler.arg("/c");

                        // --- DEBUG ---
                        if debug_on {
                            debug_output_text.push_str(" /c");
                        }
                    }
                }
            }
        }

        if let Some(include_paths) = &unit.include {
            for include_path in include_paths {
                if let Ok(absolute_inc) = include_path.canonicalize() {
                    match compiler_backend {
                        CompilerBackend::Msvc | CompilerBackend::ClangCl => {
                            target_compiler.arg(format!("/I{}", absolute_inc.to_string_lossy()));

                            // --- DEBUG ---
                            if debug_on {
                                debug_output_text.push_str(
                                    format!(" /I{}", absolute_inc.to_string_lossy()).as_str(),
                                );
                            }
                        }
                        _ => {
                            target_compiler.arg("-I").arg(&absolute_inc);

                            // --- DEBUG ---
                            if debug_on {
                                debug_output_text.push_str(
                                    format!(" -I{}", absolute_inc.to_string_lossy()).as_str(),
                                );
                            }
                        }
                    }
                }
            }
        }
        for src_file in &unit.src {
            let relative_src = src_file.strip_prefix(&base_dir)?;
            target_compiler.arg(relative_src);

            // --- DEBUG ---
            if debug_on {
                debug_output_text.push_str(format!(" {}", relative_src.to_str().unwrap()).as_str());
            }
        }

        if unit.kind == UnitKinds::Lib {
            match compiler_backend {
                CompilerBackend::Msvc | CompilerBackend::ClangCl => {}
                CompilerBackend::Clang | CompilerBackend::Gcc | CompilerBackend::Zig => {
                    if let Some(build_dir) = &lock.manifest.build.build_dir {
                        fs::create_dir_all(cache_dir.join(build_dir))?;
                        let mut unified_o = PathBuf::from(build_dir);
                        unified_o.push(format!("{}.{}", unit.name, obj_ext));
                        target_compiler.arg("-o").arg(unified_o);
                    }
                }
            }
        }

        for (dep_name, _dep_kind) in &unit.resolved_deps {
            // Tell the compiler to look right inside our current scratchpad dir for dependencies
            match compiler_backend {
                CompilerBackend::Msvc | CompilerBackend::ClangCl => {
                    target_compiler.arg(format!("{}.lib", dep_name));

                    // --- DEBUG ---
                    if debug_on {
                        debug_output_text.push_str(format!(" {}.lib", dep_name).as_str());
                    }
                }
                _ => {
                    target_compiler.arg("-L.").arg(format!("-l{}", dep_name));

                    // --- DEBUG ---
                    if debug_on {
                        debug_output_text.push_str(format!(" -l{}", dep_name).as_str());
                    }
                }
            }
        }

        let status = target_compiler.current_dir(&cache_dir).status()?;
        if !status.success() {
            return Err(format!("Failed to compile unit '{}'", unit.name).into());
        }

        // If this unit was a Static Library, we must pack the resulting object files into a .a container
        if unit.kind == UnitKinds::Lib {
            println!(
                "[info] Packing static archive for library unit: lib{}.a",
                unit.name
            );

            let mut ar_command = match compiler_backend {
                CompilerBackend::Msvc | CompilerBackend::ClangCl => {
                    let mut cmd = std::process::Command::new("lib");
                    cmd.arg(format!(
                        "/OUT:{}",
                        cache_dir
                            .join(format!("{}.lib", unit.name))
                            .to_str()
                            .unwrap()
                    ));

                    // --- DEBUG ---
                    if debug_on {
                        debug_output_text.push_str(
                            format!(
                                "lib /OUT:{}",
                                cache_dir
                                    .join(format!("{}.lib", unit.name))
                                    .to_str()
                                    .unwrap()
                            )
                            .as_str(),
                        );
                    }

                    cmd
                }
                CompilerBackend::Gcc | CompilerBackend::Zig | CompilerBackend::Clang => {
                    let mut cmd = std::process::Command::new("ar");
                    cmd.arg("rcs");
                    cmd.arg(format!("lib{}.a", unit.name));

                    // --- DEBUG ---
                    if debug_on {
                        debug_output_text.push_str(format!("ar rcs lib{}.a", unit.name).as_str());
                    }

                    cmd
                }
            };

            if let Some(build_dir) = &lock.manifest.build.build_dir {
                let mut unified_o = cache_dir.join(build_dir);
                unified_o.push(format!("{}.{}", unit.name, obj_ext));
                ar_command.arg(unified_o);
            }

            let ar_status = ar_command.current_dir(&cache_dir).status()?;
            if !ar_status.success() {
                return Err(format!(
                    "Failed to execute static library archiver on unit: {}",
                    unit.name
                )
                .into());
            }
        }

        // --- DEBUG ---
        if debug_on {
            dbg!("{}", debug_output_text);
        }
    }

    // TODO: Transpile the input files, and refactor above loop to use this.
    // NOTE: .c goes to a check function, .csal goes to a transpile function. The check function should have a bool to turn off features for .c
    // transpile::transpile(...)?;

    let updated_lock = serde_json::to_string(&lock)?;
    fs::write(base_dir.join("Salt.lock"), updated_lock)?;

    if args.run {
        let mut run_command = std::process::Command::new(&out_bin_dir);
        let status = run_command.status()?;
        if !status.success() {
            return Err(format!(
                "Failed to run executable: {}",
                out_bin_dir.to_str().unwrap()
            )
            .into());
        }
    }

    Ok(())
}
