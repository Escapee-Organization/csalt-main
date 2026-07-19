// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::cli::{BuildArgs, CompileArgs};
use crate::config::{BuildSystems, CEditions, CompilerBackend, SaltLock, SaltToml, UnitKinds};
use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(feature = "experimental")]
use std::sync::LockResult;
#[cfg(feature = "experimental")]
use toml;

pub mod cli;
pub mod config;
pub mod fs_utils;
pub mod old_build_sys;
pub mod transpile;
pub mod util;

// ---------------------- DATA ----------------------
const LOCK_VERSION: &str = "0.1.0";

pub struct PreparedUnit {
    pub name: String,
    pub kind: UnitKinds,
    pub src: Vec<PathBuf>,
    pub include: Option<Vec<PathBuf>>,
    pub resolved_deps: Vec<(String, UnitKinds, Option<PathBuf>)>,
    pub compiler_flags: Vec<String>,
    pub linker_flags: Vec<String>,
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
            Err(anyhow::anyhow!("Command '{}' not found", command_name))
        }
        Err(_) => {
            // It exists, but we ran into a permission/OS blockade (which counts as existing!)
            Ok(())
        }
    }
}

// TODO: Implement GitHub release tags and actions
#[cfg(feature = "experimental")]
pub fn update_csalt() -> Result<(), Box<dyn std::error::Error>> {
    println!("[info] Checking for updates...");
    self_update::backends::github::Update::configure()
        .repo_owner("Escapee-Organization")
        .repo_name("csalt-main")
        .bin_name("csalt")
        .current_version(env!("CARGO_PKG_VERSION"))
        .show_download_progress(true)
        .build()?;

    println!("[info] Update completed successfully.");
    Ok(())
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

/// Emits all generated assets to the cache directory.
pub fn emit_project(
    base_dir: &Path,
    cache_dir: &Path,
    build_dir: &Path,
    build_file: bool,
) -> anyhow::Result<()> {
    fs_utils::verify_workspace(base_dir)?;
    fs_utils::copy_project_files(base_dir, cache_dir)?;
    let salt_toml_str = fs::read_to_string(base_dir.join("Salt.toml"))?;
    let current_toml: SaltToml = toml::from_str(&salt_toml_str)?;

    let lock = fs_utils::load_or_init_lock(&current_toml)?;

    if build_file {
        let plan = prepare_build_plan(&lock, base_dir)?;

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(cache_dir.join("CMakeLists.txt"))?;

        let output = old_build_sys::emit_build_file_output(plan, base_dir, build_dir, &lock)?;
        writeln!(file, "{}", output)?;
    }
    Ok(())
}

/// Prepares the build plan for the project.
///
/// This function takes the lock file and prepares a list of units to correct into a plan.
/// It first collects all source paths for each unit, both single file and non-recursive directories.
/// Then, if the user wrote some, it adds all the include directory paths.
/// Finally, it resolves all dependencies and adds them to the plan.
///
/// The way it resolves dependencies is by checking the units defined before it and getting the `kind` of it.
/// If the kind is `extlib` or `extdyn`, it also adds in the path to link it.
pub fn prepare_build_plan(lock: &SaltLock, base_dir: &Path) -> anyhow::Result<Vec<PreparedUnit>> {
    let mut plan = Vec::new();

    let known_units: HashMap<String, UnitKinds> = lock
        .manifest
        .unit
        .iter()
        .map(|u| (u.name.clone(), u.kind.clone()))
        .collect();

    for unit in &lock.manifest.unit {
        let mut gathered_src_files = std::collections::BTreeSet::new();

        for src_path in &unit.src {
            let target = util::clean_windows_path(if src_path.is_absolute() {
                src_path.to_path_buf()
            } else {
                base_dir.join(src_path)
            });

            if !target.exists() {
                anyhow::bail!(
                    "File '{}' not found for sources in unit '{}'",
                    target.display(),
                    unit.name
                );
            }
            if target.is_file() {
                gathered_src_files.insert(target);
            } else if target.is_dir() {
                for entry in fs::read_dir(target)? {
                    let path = entry?.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "c") {
                        gathered_src_files.insert(path);
                    }
                }
            }
        }

        // TODO: Refactor directory scanning into a shared path resolution engine to remove duplicate code blocks. FORGIVE ME.

        let mut gathered_include_files = std::collections::BTreeSet::new();

        if let Some(include) = &unit.include {
            for include_path in include {
                let target = util::clean_windows_path(if include_path.is_absolute() {
                    include_path.to_path_buf()
                } else {
                    base_dir.join(include_path)
                });

                if target.exists() {
                    gathered_include_files.insert(target);
                } else {
                    anyhow::bail!(
                        "File '{}' not found for include in unit '{}'",
                        target.display(),
                        unit.name
                    );
                }
            }
        }

        let mut resolved_dependencies = Vec::new();
        if let Some(deps) = &unit.deps {
            for dep in deps {
                let Some(kind) = known_units.get(dep) else {
                    continue;
                };

                let mut dep_path: Option<PathBuf> = None;
                if *kind == UnitKinds::ExtLib || *kind == UnitKinds::ExtDyn {
                    /* NOTE: Since we treat the first file in 'src' as the pre-compiled binary,
                     * we use it as the dependency path if available.
                     */
                    let dep_unit = lock.manifest.unit.iter().find(|u| u.name == *dep);
                    let first_src = dep_unit.and_then(|u| u.src.first());

                    let Some(src) = first_src else {
                        anyhow::bail!(
                            "External library unit '{}' must specify the pre-compiled binary file path in 'src'",
                            dep
                        );
                    };

                    dep_path = Some(util::clean_windows_path(if src.is_absolute() {
                        src.to_path_buf()
                    } else {
                        base_dir.join(src)
                    }));
                }
                resolved_dependencies.push((dep.clone(), kind.clone(), dep_path));
            }
        }

        plan.push(PreparedUnit {
            name: unit.name.clone(),
            kind: unit.kind.clone(),
            src: gathered_src_files.into_iter().collect(),
            include: (!gathered_include_files.is_empty())
                .then(|| gathered_include_files.into_iter().collect()),
            resolved_deps: resolved_dependencies,
            compiler_flags: unit.compiler_flags.clone().unwrap_or_default(),
            linker_flags: unit.linker_flags.clone().unwrap_or_default(),
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
    println!("[info] Compiling project...");

    let raw_base_dir = match &args.path {
        Some(path) => path.canonicalize()?,
        None => std::env::current_dir()?,
    };

    let base_dir = util::clean_windows_path(raw_base_dir);
    fs_utils::verify_workspace(&base_dir)?;
    let cache_dir = base_dir.join(".csalt");

    let salt_toml_str = fs::read_to_string(base_dir.join("Salt.toml"))?;
    let current_toml: SaltToml = toml::from_str(&salt_toml_str)?;

    let lock = fs_utils::load_or_init_lock(&current_toml)?;
    // TODO: Consider a more professional output directory
    let out_bin_dir = base_dir.join(match &lock.manifest.build.build_dir {
        Some(dir) => dir,
        None => Path::new("build"),
    });
    emit_project(
        &base_dir,
        &cache_dir,
        &out_bin_dir,
        /* build_file */ false,
    )?;

    if !args.backend_flags.is_empty() {
        let target_compiler = if let Some(backend) = &args.backend {
            CompilerBackend::try_from(backend.as_str())?
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

    fs::create_dir_all(&out_bin_dir)?;

    let in_bin_dir = cache_dir.join(
        util::clean_windows_path(out_bin_dir.clone())
            .strip_prefix(util::clean_windows_path(base_dir.clone()))
            .context("Failed to create cache mirror of build directory")?,
    );
    fs::create_dir_all(&in_bin_dir)?;

    // Read in the target compiler from `Salt.lock`. CLI flag overrides
    let compiler_backend: CompilerBackend = if let Some(backend) = &args.backend {
        CompilerBackend::try_from(backend.as_str())?
    } else {
        lock.manifest.build.compiler.clone()
    };

    verify_command(compiler_backend.to_string().as_str())?;
    let build_plan = prepare_build_plan(&lock, &base_dir)?;
    let debug_on = args.debug;

    for unit in build_plan {
        if unit.kind == UnitKinds::ExtLib || unit.kind == UnitKinds::ExtDyn {
            println!(
                "[info] Skipping pre-compiled unit: {} ({:?})",
                unit.name, unit.kind
            );
            continue;
        }
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
        let obj_ext = compiler_backend.get_object_extension();
        let lib_name = compiler_backend.get_library_name(&unit.name);
        let out_lib = cache_dir.join(&lib_name);

        let dyn_ext = util::get_dynamic_library_extension();
        let dyn_name = util::get_dynamic_library_name(&unit.name);
        let out_dyn = cache_dir.join(&dyn_name);

        match compiler_backend {
            CompilerBackend::Gcc | CompilerBackend::Clang | CompilerBackend::Zig => {
                if compiler_backend == CompilerBackend::Zig {
                    target_compiler.arg("cc");
                    if let Some(target) = &args.zig_target {
                        target_compiler.arg("-target").arg(target);
                    }
                }

                target_compiler
                    .arg(format!("-std={}", lock.manifest.build.edition))
                    .args(unit.compiler_flags);

                match unit.kind {
                    UnitKinds::ExtLib | UnitKinds::ExtDyn => {}
                    UnitKinds::Bin => {
                        target_compiler.arg("-c");
                    }
                    UnitKinds::Dyn => {
                        target_compiler.arg("-c");
                    }
                    UnitKinds::Lib => {
                        // Static libraries are archives of individual object (.o) files
                        // We instruct GCC to compile source targets to relocatable objects first (-c)
                        target_compiler.arg("-c");
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
                    UnitKinds::ExtLib | UnitKinds::ExtDyn => {}
                    UnitKinds::Bin => {
                        target_compiler.arg(format!("/Fe:{}", output_executable.to_string_lossy()));
                    }
                    UnitKinds::Dyn => {
                        target_compiler
                            .arg("/LD")
                            .arg(format!("/Fe:{}", out_dyn.to_string_lossy()));
                    }
                    UnitKinds::Lib => {
                        target_compiler.arg("/c");
                    }
                }
            }
        }

        if let Some(include_paths) = &unit.include {
            for include_path in include_paths {
                if let Ok(absolute_inc) = include_path.canonicalize() {
                    match compiler_backend {
                        CompilerBackend::Msvc | CompilerBackend::ClangCl => {
                            target_compiler.arg(format!("/I{}", absolute_inc.display()));
                        }
                        _ => {
                            target_compiler.arg("-I").arg(&absolute_inc);
                        }
                    }
                }
            }
        }
        for src_file in &unit.src {
            let relative_src = src_file.strip_prefix(&base_dir)?;
            target_compiler.arg(relative_src);
        }

        // --- DEBUG ---
        if debug_on {
            println!("[DEBUG compiler] {:?}", target_compiler);
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

                    cmd
                }
                CompilerBackend::Gcc | CompilerBackend::Zig | CompilerBackend::Clang => {
                    let mut cmd = std::process::Command::new("ar");
                    cmd.arg("rcs");
                    cmd.arg(&lib_name);

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

            // --- DEBUG --
            if debug_on {
                println!("[DEBUG archiver] {:?}", ar_command);
            }

            let ar_status = ar_command.current_dir(&cache_dir).status()?;
            if !ar_status.success() {
                anyhow::bail!(
                    "Failed to execute static library archiver on unit: {}",
                    unit.name
                );
            }

            fs::copy(
                cache_dir.join(lib_name.clone()),
                out_bin_dir.join(&lib_name),
            )?;
        }

        if unit.kind == UnitKinds::Dyn || unit.kind == UnitKinds::Bin {
            let mut link_command = compiler_backend.generate_command();

            for src_file in &unit.src {
                if let Some(file_stem) = src_file.file_stem() {
                    let obj_name = format!("{}.{}", file_stem.to_string_lossy(), obj_ext);
                    link_command.arg(obj_name);
                }
            }
            match compiler_backend {
                CompilerBackend::Clang | CompilerBackend::Gcc | CompilerBackend::Zig => {
                    if compiler_backend == CompilerBackend::Zig {
                        link_command.arg("cc");
                        if let Some(target) = &args.zig_target {
                            link_command.arg("-target").arg(target);
                        }
                    }

                    if unit.kind == UnitKinds::Dyn {
                        link_command
                            .arg("-shared")
                            .arg("-fPIC")
                            .arg("-o")
                            .arg(&out_dyn);
                        if cfg!(target_os = "macos") {
                            let install_name = format!("@rpath/{}", lib_name);
                            link_command.arg("-install_name").arg(install_name);
                        }
                    } else {
                        link_command.arg("-o").arg(&output_executable);
                    }

                    link_command.arg("-L.").args(unit.linker_flags);

                    for (dep_name, dep_kind, dep_path) in &unit.resolved_deps {
                        match dep_kind {
                            UnitKinds::Lib | UnitKinds::Dyn => match compiler_backend {
                                CompilerBackend::Msvc | CompilerBackend::ClangCl => {
                                    link_command.arg(format!("{}.{}", dep_name, dyn_ext));
                                }
                                _ => {
                                    link_command.arg(format!("-l{}", dep_name));
                                }
                            },
                            UnitKinds::ExtLib | UnitKinds::ExtDyn => {
                                if let Some(path) = dep_path {
                                    let clean_path = path.canonicalize()?;
                                    link_command.arg(&clean_path);

                                    if unit.kind == UnitKinds::ExtDyn && cfg!(target_os = "macos") {
                                        link_command
                                            .arg("-Xlinker")
                                            .arg("-rpath")
                                            .arg("-Xlinker")
                                            .arg("@executable_path");
                                    }
                                } else {
                                    anyhow::bail!(
                                        "Missing pre-resolved path for external dependency: {}",
                                        dep_name
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }
                CompilerBackend::Msvc | CompilerBackend::ClangCl => {}
            }

            // --- DEBUG --
            if debug_on {
                println!("[DEBUG linker] {:?}", link_command);
            }
            let status = link_command.current_dir(&cache_dir).status()?;
            if !status.success() {
                anyhow::bail!("Failed to link unit '{}'", unit.name);
            }

            if unit.kind == UnitKinds::Dyn {
                fs::copy(&out_dyn, out_bin_dir.join(&dyn_name))?;
            }
        }
    }

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

    let base_dir = match &build_args.path {
        Some(path) => path.canonicalize()?,
        None => std::env::current_dir()?,
    };
    fs_utils::verify_workspace(&base_dir)?;
    let cache_dir = base_dir.join(".csalt");

    let salt_toml_str = fs::read_to_string(base_dir.join("Salt.toml"))?;
    let current_toml: SaltToml = toml::from_str(&salt_toml_str)?;

    let lock = fs_utils::load_or_init_lock(&current_toml)?;
    let mut build_dir = base_dir.join(
        lock.manifest
            .build
            .build_dir
            .as_deref()
            .unwrap_or(Path::new("build")),
    );
    fs::create_dir_all(&build_dir)?;
    build_dir = build_dir.canonicalize()?;

    let build_file_bool = false;
    emit_project(&base_dir, &cache_dir, &build_dir, build_file_bool)?;

    let backend = if let Some(backend) = &build_args.backend {
        BuildSystems::try_from(backend.as_str())?
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
            }
            if mode == BuildMode::Fresh {
                println!(
                    "[info] No manual configuration found. Generating Fresh CMakeLists.txt..."
                );

                let build_file_bool = true;
                emit_project(&base_dir, &cache_dir, &build_dir, build_file_bool)?;
            }

            let mut cmake_configure = std::process::Command::new("cmake");
            cmake_configure
                .current_dir(&cache_dir)
                .arg("-B")
                .arg(&build_dir)
                .arg(format!(
                    "-DCMAKE_C_COMPILER={}",
                    lock.manifest.build.compiler
                ));

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

            if mode == BuildMode::Managed {
                println!("[info] Managed Mode build finished successfully!");
            }
            if mode == BuildMode::Fresh {
                println!("[info] Fresh Mode build finished successfully!");
            }
        }
    }
    Ok(())
}
