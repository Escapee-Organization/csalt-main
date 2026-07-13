// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::cli::{BuildArgs, CompileArgs};
use crate::config::{BuildSystems, CEditions, CompilerBackend, SaltLock, SaltToml, UnitKinds};
use anyhow::Context;
#[cfg(feature = "experimental")]
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{ErrorKind, Write};
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

#[derive(Debug, PartialEq)]
pub enum BuildMode {
    Managed,
    Fresh,
}

// -------------------- FUNCTIONS --------------------

fn verify_command(command_name: &str) -> anyhow::Result<()> {
    match Command::new(command_name).spawn() {
        Ok(mut child) => {
            // Kill the child! Kill the child!
            let _ = child.kill();
            let _ = child.wait();
            Ok(())
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            // The binary is definitively missing from the system
            Err(anyhow::anyhow!("Compiler '{}' not found", command_name))
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

fn load_or_init_lock(current_toml: &SaltToml) -> anyhow::Result<SaltLock> {
    // NOTE: Boilerplate right now is kept to avoid function abstraction hell
    let lock_path = Path::new("Salt.lock");
    if !lock_path.is_file() {
        return Ok(SaltLock {
            lock_version: LOCK_VERSION.to_string(),
            manifest: current_toml.clone(),
            files: std::collections::BTreeMap::new(),
        });
    }

    let contents = fs::read_to_string(lock_path)?;
    if contents.trim().is_empty() {
        return Ok(SaltLock {
            lock_version: LOCK_VERSION.to_string(),
            manifest: current_toml.clone(),
            files: std::collections::BTreeMap::new(),
        });
    }

    let lock =
        serde_json::from_str::<SaltLock>(&contents).context("`Salt.lock` contains invalid JSON")?;
    if lock.manifest != *current_toml {
        return Ok(SaltLock {
            lock_version: LOCK_VERSION.to_string(),
            manifest: current_toml.clone(),
            files: std::collections::BTreeMap::new(),
        });
    }
    Ok(lock)
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

pub fn emit_project(base_dir: &Path, cache_dir: &Path) -> anyhow::Result<()> {
    fs_utils::verify_workspace(base_dir)?;
    fs_utils::copy_project_files(base_dir, cache_dir)?;
    Ok(())
}

pub fn prepare_build_plan(lock: &SaltLock, base_dir: &Path) -> anyhow::Result<Vec<PreparedUnit>> {
    let mut plan = Vec::new();

    let mut known_units: HashMap<String, UnitKinds> = collections::HashMap::new();
    for unit in &lock.manifest.unit {
        known_units.insert(unit.name.clone(), unit.kind.clone());
    }

    for unit in &lock.manifest.unit {
        let mut gathered_src_files = Vec::new();

        for src_path in &unit.src {
            let main_absolute = if unit.main.is_absolute() {
                unit.main.clone()
            } else {
                base_dir.join(&unit.main)
            };

            if !main_absolute.is_file() {
                anyhow::bail!(
                    "Main entry file '{}' does not exist for unit '{}'",
                    main_absolute.to_string_lossy(),
                    unit.name
                );
            }

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
                        if path.is_file() && path.extension().is_some_and(|ext| ext == "c") {
                            gathered_src_files.push(path);
                        }
                    }
                }
            } else {
                anyhow::bail!(
                    "File '{}' not found for sources in unit '{}'",
                    target.to_string_lossy(),
                    unit.name
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
                    anyhow::bail!(
                        "File '{}' not found for include in unit '{}'",
                        target.to_string_lossy(),
                        unit.name
                    );
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
pub fn build_manual_project(args: &CompileArgs) -> anyhow::Result<()> {
    println!("[info]\nCompiling project...");

    let base_dir = std::env::current_dir()?;
    fs_utils::verify_workspace(&base_dir)?;
    let cache_dir = base_dir.join(".csalt");

    let salt_toml_str = fs::read_to_string(base_dir.join("Salt.toml"))?;
    let current_toml: SaltToml = toml::from_str(&salt_toml_str)?;

    let lock = load_or_init_lock(&current_toml)?;
    emit_project(&base_dir, &cache_dir)?;

    if !args.backend_flags.is_empty() {
        let target_compiler = if let Some(backend) = &args.backend {
            CompilerBackend::from_string(backend.as_str())?
        } else {
            lock.manifest.build.compiler
        };

        let mut actual_compiler = target_compiler.generate_command();
        actual_compiler.args(args.backend_flags.iter());
        actual_compiler.current_dir(&cache_dir);
        let status = actual_compiler.status()?;
        if !status.success() {
            anyhow::bail!("Failed to compile");
        }
        return Ok(());
    }

    // TODO: Consider a more professional output directory
    let out_bin_dir = base_dir.join(match &lock.manifest.build.build_dir {
        Some(dir) => dir,
        None => Path::new("build/"),
    });
    fs::create_dir_all(&out_bin_dir)?;
    let in_bin_dir = cache_dir.join(
        out_bin_dir
            .strip_prefix(&base_dir)
            .context("Failed to create cache mirror of build directory")?,
    );
    fs::create_dir_all(&in_bin_dir)?;

    // Read in the target compiler from `Salt.lock`. CLI flag overrides
    let compiler_backend: CompilerBackend = if let Some(backend) = &args.backend {
        CompilerBackend::from_string(backend.as_str())?
    } else {
        lock.manifest.build.compiler.clone()
    };

    verify_command(compiler_backend.to_string())?;
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
        let lib_ext = match compiler_backend {
            CompilerBackend::Msvc | CompilerBackend::ClangCl => "lib",
            CompilerBackend::Clang | CompilerBackend::Gcc | CompilerBackend::Zig => "a",
        };
        let lib_name = match compiler_backend {
            CompilerBackend::Msvc | CompilerBackend::ClangCl => {
                format!("{}.{}", unit.name, lib_ext)
            }
            CompilerBackend::Clang | CompilerBackend::Gcc | CompilerBackend::Zig => {
                format!("lib{}.{}", unit.name, lib_ext)
            }
        };
        let out_lib = cache_dir.join(&lib_name);

        let dyn_ext = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };
        let dyn_name = if cfg!(target_os = "windows") {
            format!("{}.{}", unit.name, dyn_ext)
        } else {
            format!("lib{}.{}", unit.name, dyn_ext)
        };
        let out_dyn = out_bin_dir.join(&dyn_name);

        // --- DEBUG ---
        let mut debug_output_text = String::new();
        if debug_on {
            debug_output_text.push_str("[DEBUG COMMAND] ");
            debug_output_text.push_str(compiler_backend.to_string());
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

                target_compiler.arg(format!("-std={}", lock.manifest.build.edition.to_string()));

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
                    CEditions::C11 => {
                        target_compiler.arg("/std:c11");
                    }
                    CEditions::C17 => {
                        target_compiler.arg("/std:c17");
                    }
                    CEditions::C23 => {
                        target_compiler.arg("/std:clatest");
                    }
                    _ => {} // Unsupported editions are ignored
                };

                match unit.kind {
                    UnitKinds::Bin => {
                        target_compiler.arg(format!("/Fe:{}", output_executable.to_string_lossy()));

                        // --- DEBUG ---
                        if debug_on {
                            debug_output_text.push_str(
                                format!("/Fe:{}", output_executable.to_string_lossy()).as_str(),
                            );
                        }
                    }
                    UnitKinds::Dyn => {
                        target_compiler
                            .arg("/LD")
                            .arg(format!("/Fe:{}", out_dyn.to_string_lossy()));

                        // --- DEBUG ---
                        if debug_on {
                            debug_output_text.push_str(
                                format!("/LD /Fe:{}", out_dyn.to_string_lossy()).as_str(),
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
                debug_output_text.push_str(format!(" {}", relative_src.to_string_lossy()).as_str());
            }
        }

        for (dep_name, _dep_kind) in &unit.resolved_deps {
            // Tell the compiler to look right inside our current scratchpad dir for dependencies
            match compiler_backend {
                CompilerBackend::Msvc | CompilerBackend::ClangCl => {
                    target_compiler.arg(format!("{}.{}", dep_name, dyn_ext));

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
            anyhow::bail!("Failed to compile unit '{}'", unit.name);
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
                    cmd.arg(format!("/OUT:{}", out_lib.to_string_lossy()));

                    // --- DEBUG ---
                    if debug_on {
                        debug_output_text.push_str(
                            format!(
                                "lib /OUT:{}",
                                cache_dir
                                    .join(format!("{}.lib", unit.name))
                                    .to_string_lossy()
                            )
                            .as_str(),
                        );
                    }

                    cmd
                }
                CompilerBackend::Gcc | CompilerBackend::Zig | CompilerBackend::Clang => {
                    let mut cmd = std::process::Command::new("ar");
                    cmd.arg("rcs");
                    cmd.arg(&lib_name);

                    // --- DEBUG ---
                    if debug_on {
                        debug_output_text.push_str(format!("ar rcs lib{}.a", unit.name).as_str());
                    }

                    cmd
                }
            };

            for src_file in &unit.src {
                let path = Path::new(src_file);
                if let Some(file_stem) = path.file_stem() {
                    let object_file_name = format!("{}.{}", file_stem.to_string_lossy(), obj_ext);

                    let object_path = cache_dir.join(object_file_name);
                    ar_command.arg(object_path);
                }
            }

            let ar_status = ar_command.current_dir(&cache_dir).status()?;
            if !ar_status.success() {
                anyhow::bail!(
                    "Failed to execute static library archiver on unit: {}",
                    unit.name
                );
            }

            fs::copy(cache_dir.join(lib_name.clone()), out_bin_dir.join(lib_name))?;
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
            anyhow::bail!(
                "Failed to run executable: {}",
                out_bin_dir.to_string_lossy()
            );
        }
    }

    Ok(())
}

pub fn build_managed_project(build_args: &BuildArgs) -> anyhow::Result<()> {
    println!("[info] Building project...");

    let base_dir = std::env::current_dir()?;
    fs_utils::verify_workspace(&base_dir)?;
    let cache_dir = base_dir.join(".csalt");

    let salt_toml_str = fs::read_to_string(base_dir.join("Salt.toml"))?;
    let current_toml: SaltToml = toml::from_str(&salt_toml_str)?;

    let lock = load_or_init_lock(&current_toml)?;
    emit_project(&base_dir, &cache_dir)?;
    let mut build_dir = base_dir.join(
        lock.manifest
            .build
            .build_dir
            .as_deref()
            .unwrap_or(Path::new("build")),
    );
    fs::create_dir_all(&build_dir)?;
    build_dir = build_dir.canonicalize()?;

    let backend = if let Some(backend) = &build_args.backend {
        BuildSystems::from_string(backend)?
    } else {
        lock.manifest.build.build_sys.clone()
    };

    if !build_args.backend_flags.is_empty() {
        let mut target_build = backend.generate_command();
        target_build
            .args(&build_args.backend_flags)
            .current_dir(&base_dir);
        let status = target_build.status()?;
        if !status.success() {
            anyhow::bail!("Failed to build project");
        }

        return Ok(());
    }

    match backend {
        BuildSystems::CMake => {
            let user_cmake_path = base_dir.join("CMakeLists.txt");
            let mode = if user_cmake_path.exists() {
                BuildMode::Managed
            } else {
                BuildMode::Fresh
            };

            if mode == BuildMode::Managed {
                println!("[info] Manual CMakeLists.txt detected. Running in Managed Mode...");
                // NOTE: Consider using the compiler option to choose which one to search for first

                let mut cmake_configure = std::process::Command::new("cmake");
                cmake_configure
                    .current_dir(&cache_dir)
                    .arg("-B")
                    .arg(&build_dir);

                let config_status = cmake_configure.status()?;
                if !config_status.success() {
                    anyhow::bail!("CMake configuration failed");
                }

                let mut cmake_build = std::process::Command::new("cmake");
                cmake_build
                    .current_dir(&cache_dir)
                    .arg("--build")
                    .arg(&build_dir);

                let build_status = cmake_build.status()?;
                if !build_status.success() {
                    anyhow::bail!("CMake build step failed");
                }

                println!("[info] Managed Mode build finished successfully!");
            } else {
                println!(
                    "[info] No manual configuration found. Generating Fresh CMakeLists.txt..."
                );

                let plan = prepare_build_plan(&lock, base_dir.as_path())?;

                match fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(cache_dir.join("CMakeLists.txt"))
                {
                    Ok(mut file) => {
                        writeln!(
                            file,
                            "cmake_minimum_required(VERSION {})",
                            lock.manifest.build.build_sys_ver
                        )?;
                        writeln!(file, "project({} LANGUAGES C)", lock.manifest.package.name)?;
                        writeln!(
                            file,
                            "# !![ REMOVE the following line (output directory) if moving to main directory ]!!"
                        )?;
                        writeln!(
                            file,
                            "set(CMAKE_RUNTIME_OUTPUT_DIRECTORY \"{}\")",
                            &build_dir.to_string_lossy().replace('\\', "/")
                        )?;
                        writeln!(
                            file,
                            "set(CMAKE_C_STANDARD {})\nset(CMAKE_C_STANDARD_REQUIRED ON)",
                            lock.manifest.build.edition.to_string().replace('c', "")
                        )?;
                        writeln!(file)?;

                        for unit in &plan {
                            let mut unit_paths = Vec::new();
                            for path in &unit.src {
                                if let Ok(relative_path) = path.strip_prefix(&base_dir) {
                                    unit_paths
                                        .push(relative_path.to_string_lossy().replace('\\', "/"));
                                }
                            }
                            let src_paths = unit_paths.join(" ");

                            match unit.kind {
                                UnitKinds::Bin => {
                                    writeln!(file, "# === UNIT: bin {} ===", unit.name)?;
                                    writeln!(file, "add_executable({} {})", unit.name, src_paths)?;
                                }
                                UnitKinds::Lib => {
                                    writeln!(file, "# === UNIT: lib {} ===", unit.name)?;
                                    writeln!(
                                        file,
                                        "add_library({} STATIC {})",
                                        unit.name, src_paths
                                    )?;
                                }
                                UnitKinds::Dyn => {
                                    writeln!(file, "# === UNIT: dyn {} ===", unit.name)?;
                                    writeln!(
                                        file,
                                        "add_library({} SHARED {})",
                                        unit.name, src_paths
                                    )?;
                                }
                            }
                            if let Some(includes) = &unit.include {
                                for inc in includes {
                                    if let Ok(rel_inc) = inc.strip_prefix(&base_dir) {
                                        writeln!(
                                            file,
                                            "target_include_directories({} PRIVATE {})",
                                            unit.name,
                                            rel_inc.to_string_lossy()
                                        )?;
                                    }
                                }
                            }

                            if !unit.resolved_deps.is_empty() {
                                writeln!(file, "# === DEPS: {} ===", &unit.name)?;
                                write!(file, "target_link_libraries({} PRIVATE ", unit.name)?;
                                let mut unit_deps = Vec::new();
                                for (dep_name, _dep_kind) in &unit.resolved_deps {
                                    unit_deps.push(dep_name.clone());
                                }
                                let dep_str = unit_deps.join(" ");
                                writeln!(file, "{})", dep_str)?;
                            }
                            writeln!(file)?;
                        }

                        let mut cmake_configure = std::process::Command::new("cmake");
                        cmake_configure
                            .current_dir(&cache_dir)
                            .arg("-B")
                            .arg(&build_dir);
                        if !cmake_configure.status()?.success() {
                            anyhow::bail!("CMake configuration failed in Fresh Mode");
                        }

                        let mut cmake_build = std::process::Command::new("cmake");
                        cmake_build
                            .current_dir(&cache_dir)
                            .arg("--build")
                            .arg(&build_dir);
                        if !cmake_build.status()?.success() {
                            anyhow::bail!("CMake build step failed in Fresh Mode");
                        }

                        println!("[info] Fresh Mode build finished successfully!");
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        }
    }
    Ok(())
}
