// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::config::UnitKinds;

pub mod linker;

// ---------------------- DATA ----------------------

pub const LOCK_VERSION: &str = "0.1.0";

pub struct PreparedUnit {
    pub name: String,
    pub kind: UnitKinds,
    pub src: Vec<std::path::PathBuf>,
    pub include: Option<Vec<std::path::PathBuf>>,
    pub resolved_deps: Vec<(String, UnitKinds, Option<std::path::PathBuf>)>,
    pub compiler_flags: Vec<String>,
    pub linker_flags: Vec<String>,
    pub unpack_compiler_flags: Vec<String>,
    pub unpack_linker_flags: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum BuildMode {
    Managed,
    Fresh,
}

// ---------------- DATA -> FUNCTIONS ----------------

impl TryFrom<&str> for BuildMode {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "managed" => Ok(BuildMode::Managed),
            "fresh" => Ok(BuildMode::Fresh),
            _ => anyhow::bail!("Invalid build mode: {}", s),
        }
    }
}

// -------------------- FUNCTIONS --------------------

/// Verifies that the given command is available on the system.
///
/// ### Examples
/// ```
/// csalt::helpers::verify_command("mkdir").unwrap();
/// ```
pub fn verify_command(command_name: &str) -> anyhow::Result<()> {
    match std::process::Command::new(command_name).spawn() {
        Ok(mut child) => {
            // Kill the child! Kill the child!
            let _ = child.kill();
            let _ = child.wait();
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // The binary is definitively missing from the system
            Err(anyhow::anyhow!("Command '{}' not found", command_name))
        }
        Err(_) => {
            // It exists, but we ran into a permission/OS blockade (which counts as existing!)
            Ok(())
        }
    }
}

/// Attaches the Zig target argument to the command if the backend is Zig.
///
/// ### Example
///
/// ```
/// let mut command = std::process::Command::new("zig");
/// csalt::helpers::attach_zig_target_arg(csalt::config::CompilerBackend::Zig, &mut command, Some("x86_64-pc-windows-msvc".to_string()));
/// ```
pub fn attach_zig_target_arg(
    compiler_backend: crate::config::CompilerBackend,
    command: &mut std::process::Command,
    zig_target: Option<String>,
) {
    if compiler_backend == crate::config::CompilerBackend::Zig {
        command.arg("cc");
        if let Some(target) = zig_target {
            command.arg("-target").arg(target);
        }
    }
}

fn save_flag(flag: &str, compiler: &mut Vec<String>, linker: &mut Vec<String>) {
    let trimmed = flag.trim();
    if trimmed.is_empty() {
        return;
    }

    if trimmed.starts_with("-I") {
        compiler.push(trimmed.to_string());
    } else if trimmed.starts_with("-L") || trimmed.starts_with("-l") {
        linker.push(trimmed.to_string());
    }
}

/// Parses compiler/linker flags from the raw stdout of a `pkg-config` call.
///
/// This function parses flags in a linear, character-by-character scan.
/// It saves the current flag to the compiler/linker vector when reaching a new flag boundary.
/// ### Example
/// ```
/// let raw_stdout = "Compilation flags: -I/usr/include -L/usr/lib -l";
/// let mut true_compiler_flags = Vec::new();
/// let mut true_linker_flags = Vec::new();
/// csalt::helpers::parse_flags_linear(raw_stdout, &mut true_compiler_flags, &mut true_linker_flags);
///
/// assert_eq!(true_compiler_flags, vec!["-I/usr/include"]);
/// assert_eq!(true_linker_flags, vec!["-L/usr/lib", "-l"]);
/// ```
pub fn parse_flags_linear(
    raw_stdout: &str,
    true_compiler_flags: &mut Vec<String>,
    true_linker_flags: &mut Vec<String>,
) {
    let chars: Vec<char> = raw_stdout.chars().collect();
    let mut current_flag = String::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '-'
            && i + 1 < chars.len()
            && (chars[i + 1] == 'I' || chars[i + 1] == 'L' || chars[i + 1] == 'l')
        {
            let is_new_flag = i == 0 || chars[i - 1].is_whitespace();

            if is_new_flag {
                save_flag(&current_flag, true_compiler_flags, true_linker_flags);
                current_flag.clear();

                current_flag.push(chars[i]);
                current_flag.push(chars[i + 1]);
                i += 2;
                continue;
            }
        }

        current_flag.push(chars[i]);
        i += 1;
    }

    save_flag(&current_flag, true_compiler_flags, true_linker_flags);
}

fn suggest_installation_commands(dep: &str) {
    println!("`pkg` '{}' could not be found", dep);

    let managers = [
        ("apt", format!("apt search {} | grep dev", dep)),
        ("dnf", format!("dnf search {}", dep)),
        ("pacman", format!("pacman -Ss {}", dep)),
        ("brew", format!("brew search {}", dep)),
        ("nix", format!("nix-env -qaP '.*{}.*'", dep)),
        ("winget", format!("winget search {}", dep)),
        ("vcpkg", format!("vcpkg search {}", dep)),
        ("scoop", format!("scoop search {}", dep)),
    ];

    let mut found_any = false;
    for (cli_name, search_command) in managers {
        if verify_command(cli_name).is_ok() {
            if !found_any {
                println!("\n[help] To find '{}', run a command below:", dep);
            }
            println!("    {}", search_command);
            found_any = true;
        }
    }
    if found_any {
        println!(
            "\n[info] Note that the `pkg` '{}' may be listed under a different name in any of the above commands",
            dep
        );
    }

    println!("[help] You may install it using alternative methods or manually");
}

/// Calls `pkg-config --libs --cflags <dep>` and takes stdout to add it to the newly generated compiler and linker flags.
pub fn call_and_record_pkg_config(
    dep: &String,
    new_compiler_flags: &mut Vec<String>,
    new_linker_flags: &mut Vec<String>,
    known_packages: &mut std::collections::HashMap<String, (Vec<String>, Vec<String>)>,
) -> anyhow::Result<()> {
    let output = std::process::Command::new("pkg-config")
        .arg("--libs")
        .arg("--cflags")
        .arg(dep)
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let raw_stdout = String::from_utf8_lossy(&out.stdout);

            parse_flags_linear(&raw_stdout, new_compiler_flags, new_linker_flags);
            known_packages.insert(
                dep.clone(),
                (new_compiler_flags.clone(), new_linker_flags.clone()),
            );
        } else {
            suggest_installation_commands(dep);
            anyhow::bail!("Failed to call `pkg-config`");
        }
    } else {
        anyhow::bail!("Failed to call `pkg-config`");
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
pub fn prepare_build_plan(
    lock: &crate::config::SaltLock,
    base_dir: &std::path::Path,
) -> anyhow::Result<Vec<PreparedUnit>> {
    let mut plan = Vec::new();

    let known_units: std::collections::HashMap<String, UnitKinds> = lock
        .manifest
        .unit
        .iter()
        .map(|u| (u.name.clone(), u.kind.clone()))
        .collect();

    // A map of package names to their compiler and linker flags.
    let mut known_packages: std::collections::HashMap<String, (Vec<String>, Vec<String>)> =
        std::collections::HashMap::new();

    for unit in &lock.manifest.unit {
        let mut gathered_src_files = std::collections::BTreeSet::new();

        for src_path in &unit.src {
            let target = crate::util::clean_windows_path(if src_path.is_absolute() {
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
                for entry in std::fs::read_dir(target)? {
                    let path = entry?.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "c") {
                        gathered_src_files.insert(path);
                    }
                }
            }
        }

        // TODO: Refactor directory scanning into a shared path resolution engine to remove duplicate code blocks. FORGIVE ME.

        let mut gathered_include_files = std::collections::BTreeSet::new();
        let include = &unit.include;

        for include_path in include.iter().flatten() {
            let target = crate::util::clean_windows_path(if include_path.is_absolute() {
                include_path.to_path_buf()
            } else {
                base_dir.join(include_path)
            });

            if target.exists() {
                gathered_include_files.insert(target);
                continue;
            }

            anyhow::bail!(
                "File '{}' not found for include in unit '{}'",
                target.display(),
                unit.name
            );
        }

        let mut resolved_dependencies = Vec::new();
        let deps = &unit.deps;
        let mut new_compiler_flags = Vec::new();
        let mut new_linker_flags = Vec::new();

        for dep in deps.iter().flatten() {
            let Some(kind) = known_units.get(dep) else {
                continue;
            };

            let mut dep_path: Option<std::path::PathBuf> = None;
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

                dep_path = Some(crate::util::clean_windows_path(if src.is_absolute() {
                    src.to_path_buf()
                } else {
                    base_dir.join(src)
                }));
            }

            if *kind == UnitKinds::Pkg {
                if let Some((compiler_flags, linker_flags)) = known_packages.get(dep) {
                    new_compiler_flags.extend(compiler_flags.clone());
                    new_linker_flags.extend(linker_flags.clone());
                } else if verify_command("pkg-config").is_ok() {
                    call_and_record_pkg_config(
                        dep,
                        &mut new_compiler_flags,
                        &mut new_linker_flags,
                        &mut known_packages,
                    )?;
                } else {
                    anyhow::bail!("Could not find `pkg-config` for unit `{}`", unit.name);
                }
            }
            resolved_dependencies.push((dep.clone(), kind.clone(), dep_path));
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
            unpack_compiler_flags: new_compiler_flags.clone(),
            unpack_linker_flags: new_linker_flags.clone(),
        });
    }

    Ok(plan)
}
