// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::config::{BuildSystems, CompilerBackend, SaltLock, SaltToml, UnitKinds};
use crate::helpers::{
    BuildMode, PreparedUnit, linker::LinkerDriver, prepare_build_plan, verify_command,
};
use anyhow::Context;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(feature = "experimental")]
use std::sync::LockResult;

pub mod cli;
pub mod config;
pub mod fs_utils;
pub mod helpers;
pub mod old_build_sys;
pub mod util;

// -------------------- FUNCTIONS --------------------

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
    plan: Option<Vec<PreparedUnit>>,
    verbose: bool,
) -> anyhow::Result<()> {
    fs_utils::verify_workspace(base_dir)?;
    fs_utils::copy_project_files(base_dir, cache_dir, build_dir, verbose)?;
    let salt_toml_str = fs::read_to_string(base_dir.join("Salt.toml"))?;
    let current_toml: SaltToml = toml::from_str(&salt_toml_str)?;

    let lock = fs_utils::load_or_init_lock(&current_toml)?;

    if let Some(plan) = plan {
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

pub fn build_manual_project(
    backend: &Option<String>,
    path: &Option<PathBuf>,
    _mode: &Option<String>,
    run: bool,
    zig_target: &Option<String>,
    verbose_on: bool,
    backend_flags: &[String],
) -> anyhow::Result<()> {
    println!("[info] Compiling project...");

    let raw_base_dir = match path {
        Some(path) => path.canonicalize()?,
        None => std::env::current_dir()?,
    };

    let base_dir = util::clean_windows_path(raw_base_dir);
    fs_utils::verify_workspace(&base_dir)?;
    let cache_dir = base_dir.join(".csalt");

    let salt_toml_str = fs::read_to_string(base_dir.join("Salt.toml"))?;
    let current_toml: SaltToml = toml::from_str(&salt_toml_str)?;

    let lock = fs_utils::load_or_init_lock(&current_toml)?;
    let out_bin_dir = base_dir.join(match &lock.manifest.build.build_dir {
        Some(dir) => dir,
        None => Path::new("build"),
    });
    emit_project(&base_dir, &cache_dir, &out_bin_dir, None, verbose_on)?;

    if !backend_flags.is_empty() {
        let target_compiler = if let Some(backend) = &backend {
            CompilerBackend::try_from(backend.as_str())?
        } else {
            match lock.manifest.build.compiler {
                Some(backend) => backend,
                None => CompilerBackend::attempt_find_compiler()?,
            }
        };

        let mut actual_compiler = target_compiler.generate_command();
        actual_compiler.args(backend_flags.iter());
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

    let compiler_backend: CompilerBackend = if let Some(backend) = &backend {
        CompilerBackend::try_from(backend.as_str())?
    } else {
        match lock.manifest.build.compiler {
            Some(ref backend) => backend.clone(),
            None => CompilerBackend::attempt_find_compiler()?,
        }
    };

    verify_command(compiler_backend.to_string().as_str())?;
    let build_plan = prepare_build_plan(&lock, &base_dir)?;

    for unit in build_plan {
        if unit.kind == UnitKinds::ExtLib
            || unit.kind == UnitKinds::ExtDyn
            || unit.kind == UnitKinds::Pkg
        {
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

        let output_executable = if cfg!(target_os = "windows") {
            out_bin_dir.join(&unit.name).with_extension("exe")
        } else {
            out_bin_dir.join(&unit.name)
        };
        let obj_ext = compiler_backend.get_object_extension();
        let lib_name = compiler_backend.get_library_name(&unit.name);
        #[cfg(feature = "experimental")]
        let out_lib = cache_dir.join(&lib_name);
        #[cfg(feature = "experimental")]
        let dyn_ext = util::get_dynamic_library_extension();
        let dyn_name = util::get_dynamic_library_name(&unit.name);
        let out_dyn = cache_dir.join(&dyn_name);

        for src_file in &unit.src {
            println!("[info] Compiling source file: {}", src_file.display());
            let mut target_compiler = compiler_backend.generate_command();
            helpers::attach_zig_target_arg(
                compiler_backend.clone(),
                &mut target_compiler,
                zig_target.clone(),
            );

            let include_paths = unit.include.clone().unwrap_or_default();
            for include_path in include_paths {
                if let Ok(absolute_inc) = include_path.canonicalize() {
                    match compiler_backend {
                        #[cfg(feature = "experimental")]
                        CompilerBackend::Msvc | CompilerBackend::ClangCl => {
                            target_compiler.arg(format!("/I{}", absolute_inc.display()));
                        }
                        _ => {
                            target_compiler.arg("-I").arg(&absolute_inc);
                        }
                    }
                }
            }
            target_compiler.args(&unit.unpack_compiler_flags);

            match compiler_backend {
                CompilerBackend::Gcc | CompilerBackend::Clang | CompilerBackend::Zig => {
                    target_compiler
                        .arg(format!("-std={}", lock.manifest.build.edition))
                        .args(&unit.compiler_flags);

                    if unit.kind == UnitKinds::Lib
                        || unit.kind == UnitKinds::Bin
                        || unit.kind == UnitKinds::Dyn
                    {
                        target_compiler.arg("-c");
                    }
                }
                #[cfg(feature = "experimental")]
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

                    // NOTE: This has NOT been touched to work the same as Gcc-like compilers
                    match unit.kind {
                        UnitKinds::ExtLib | UnitKinds::ExtDyn => {}
                        UnitKinds::Bin => {
                            target_compiler
                                .arg(format!("/Fe:{}", output_executable.to_string_lossy()));
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
            let relative_src = src_file.strip_prefix(&base_dir)?;
            target_compiler.arg(relative_src);
            let mut obj_output = cache_dir.join(relative_src);
            obj_output.set_extension(obj_ext);

            match compiler_backend {
                #[cfg(feature = "experimental")]
                CompilerBackend::Msvc | CompilerBackend::ClangCl => {
                    target_compiler.arg(format!("/Fo:{}", obj_output.to_string_lossy()));
                }
                _ => {
                    target_compiler.arg("-o").arg(&obj_output);
                }
            }

            // --- VERBOSE ---
            if verbose_on {
                println!("[cmd compiler] {:?}", target_compiler);
            }

            let status = target_compiler
                .current_dir(&cache_dir)
                .status()
                .with_context(|| {
                    format!("Failed to compile source file '{}'", relative_src.display())
                })?;
            if !status.success() {
                anyhow::bail!("Failed to compile source file '{}'", relative_src.display());
            }
        }

        println!("[info] Compiled unit: {}", unit.name);

        // If this unit was a Static Library, we must pack the resulting object files into a .a container
        if unit.kind == UnitKinds::Lib {
            println!(
                "[info] Packing static archive for library unit: {}",
                lib_name
            );

            let mut ar_command = match compiler_backend {
                #[cfg(feature = "experimental")]
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
                let relative_src = src_file.strip_prefix(&base_dir)?;
                let mut object_path = cache_dir.join(relative_src);
                object_path.set_extension(obj_ext);
                ar_command.arg(&object_path);
            }

            // --- VERBOSE ---
            if verbose_on {
                println!("[cmd archiver] {:?}", ar_command);
            }

            let ar_status = ar_command
                .current_dir(&cache_dir)
                .status()
                .with_context(|| {
                    format!(
                        "Failed to execute static library archiver on unit: {}",
                        unit.name
                    )
                })?;
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
                let relative_src = src_file.strip_prefix(&base_dir)?;
                let mut object_path = cache_dir.join(relative_src);
                object_path.set_extension(obj_ext);
                link_command.arg(&object_path);
            }

            let linker_driver = LinkerDriver::new(&compiler_backend, zig_target.clone());
            linker_driver.build_flags(
                &mut link_command,
                &unit,
                &out_dyn,
                &dyn_name,
                &output_executable,
            )?;

            // --- VERBOSE --
            if verbose_on {
                println!("[cmd linker] {:?}", link_command);
            }
            let status = link_command
                .current_dir(&cache_dir)
                .status()
                .with_context(|| format!("Failed to link unit '{}'", unit.name))?;
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

    if run {
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

pub fn build_managed_project(
    backend: &Option<String>,
    path: &Option<PathBuf>,
    mode: &Option<String>,
    backend_flags: &Vec<String>,
    zig_target: &Option<String>,
    verbose: bool,
) -> anyhow::Result<()> {
    println!("[info] Building project...");

    let base_dir = match path {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir()?,
    };
    fs_utils::verify_workspace(&base_dir)?;

    let cache_dir = base_dir.join(".csalt");

    let salt_toml_str = fs::read_to_string(base_dir.join("Salt.toml"))?;
    let current_toml: SaltToml = toml::from_str(&salt_toml_str)?;

    let lock = fs_utils::load_or_init_lock(&current_toml)?;

    let floating_build_dir = lock
        .manifest
        .build
        .build_dir
        .as_deref()
        .unwrap_or(Path::new("build"));
    let build_dir = &base_dir.join(floating_build_dir);
    fs::create_dir_all(build_dir)?;

    emit_project(&base_dir, &cache_dir, build_dir, None, verbose)?;

    let backend = if let Some(backend) = backend {
        BuildSystems::try_from(backend.as_str())?
    } else {
        lock.manifest
            .build
            .build_sys
            .clone()
            .ok_or(anyhow::anyhow!("no build system specified"))?
    };

    if !backend_flags.is_empty() {
        let plan = prepare_build_plan(&lock, &base_dir)?;
        emit_project(&base_dir, &cache_dir, build_dir, Some(plan), verbose)?;
        let mut target_build = backend.generate_command();
        target_build.args(backend_flags).current_dir(&base_dir);

        // --- VERBOSE ---
        if verbose {
            println!("[cmd build] {:?}", target_build);
        }

        let status = target_build.status()?;
        if !status.success() {
            anyhow::bail!("Failed to build project");
        }

        return Ok(());
    }

    let plan = prepare_build_plan(&lock, &base_dir)?;
    match backend {
        BuildSystems::CMake => {
            let user_cmake_path = base_dir.join("CMakeLists.txt");
            let mode = if let Some(mode) = mode {
                BuildMode::try_from(mode.as_str())?
            } else if user_cmake_path.exists() {
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

                emit_project(
                    &base_dir,
                    &cache_dir,
                    floating_build_dir,
                    Some(plan),
                    verbose,
                )?;
            }

            let mut cmake_configure = std::process::Command::new("cmake");
            cmake_configure
                .current_dir(&cache_dir)
                .arg("-B")
                .arg(floating_build_dir);
            if let Some(compiler) = &lock.manifest.build.compiler {
                // NOTE: Why is it this way and not the other way?
                if verify_command(compiler.to_string().as_str()).is_err() {
                    eprintln!("[warning] Compiler not found: '{}'", compiler);
                } else {
                    cmake_configure.arg(format!("-DCMAKE_C_COMPILER={}", compiler));
                    if *compiler == CompilerBackend::Zig {
                        if let Some(target) = zig_target {
                            cmake_configure
                                .arg(format!("-DCMAKE_C_COMPILER_ARG1=\"cc -target {}\"", target));
                        } else {
                            cmake_configure.arg("-DCMAKE_C_COMPILER_ARG1=cc");
                        }
                    }
                }
            }

            let config_status = cmake_configure.status()?;
            if !config_status.success() {
                anyhow::bail!("CMake configuration failed");
            }

            let mut cmake_build = std::process::Command::new("cmake");
            cmake_build
                .current_dir(&cache_dir)
                .arg("--build")
                .arg(floating_build_dir);

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
