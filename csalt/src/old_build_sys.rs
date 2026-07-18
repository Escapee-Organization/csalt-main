// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::PreparedUnit;
use crate::config::{BuildSystems, UnitKinds};
use std::fmt::Write;

pub fn emit_build_file_output(
    build_plan: Vec<PreparedUnit>,
    base_dir: &std::path::Path,
    build_dir: &std::path::Path,
    lock: &crate::SaltLock,
) -> anyhow::Result<String> {
    let mut output = String::new();

    let chosen_build_sys_ver = crate::util::normalize_semver(&lock.manifest.build.build_sys_ver)?;
    match lock.manifest.build.build_sys {
        BuildSystems::CMake => {
            let ver_3_15 = semver::VersionReq::parse(">=3.15.0")?;
            if ver_3_15.matches(&chosen_build_sys_ver) {
                writeln!(
                    output,
                    "cmake_minimum_required(VERSION {})",
                    lock.manifest.build.build_sys_ver
                )?;
                writeln!(
                    output,
                    "project({} LANGUAGES C)",
                    lock.manifest.package.name
                )?;
                writeln!(
                    output,
                    "# !![ REMOVE the following line (output directory) if moving to main directory ]!!"
                )?;
                writeln!(
                    output,
                    "set(CMAKE_RUNTIME_OUTPUT_DIRECTORY \"{}\")",
                    &build_dir.to_string_lossy().replace('\\', "/")
                )?;
                writeln!(
                    output,
                    "set(CMAKE_C_STANDARD {})\nset(CMAKE_C_STANDARD_REQUIRED ON)",
                    lock.manifest.build.edition.to_string().replace('c', "")
                )?;
                writeln!(output)?;

                for unit in &build_plan {
                    let src_paths = &unit
                        .src
                        .iter()
                        .map(|p| {
                            let clean_path = p.strip_prefix(base_dir).unwrap_or(p);
                            format!("\"{}\"", clean_path.to_string_lossy().replace('\\', "/"))
                        })
                        .collect::<Vec<_>>()
                        .join(" ");

                    match unit.kind {
                        UnitKinds::Bin => {
                            writeln!(output, "# === UNIT: bin {} ===", unit.name)?;
                            writeln!(output, "add_executable({} {})", unit.name, src_paths)?;
                        }
                        UnitKinds::Lib => {
                            writeln!(output, "# === UNIT: lib {} ===", unit.name)?;
                            writeln!(output, "add_library({} STATIC {})", unit.name, src_paths)?;
                        }
                        UnitKinds::Dyn => {
                            writeln!(output, "# === UNIT: dyn {} ===", unit.name)?;
                            writeln!(output, "add_library({} SHARED {})", unit.name, src_paths)?;
                        }
                        UnitKinds::ExtLib => {
                            writeln!(output, "# === UNIT: extlib {} ===", unit.name)?;
                            // 1. Declare the target as an IMPORTED STATIC library
                            writeln!(output, "add_library({} STATIC IMPORTED GLOBAL)", unit.name)?;
                            // 2. Set the property pointing directly to the pre-compiled file path
                            // (Assuming `src_paths` contains the single path to your .a/.lib file)
                            writeln!(
                                output,
                                "set_target_properties({} PROPERTIES IMPORTED_LOCATION \"${{CMAKE_CURRENT_SOURCE_DIR}}/{}\")",
                                unit.name,
                                src_paths.replace('"', "")
                            )?;
                        }
                        UnitKinds::ExtDyn => {
                            writeln!(output, "# === UNIT: extdyn {} ===", unit.name)?;
                            // 1. Declare the target as an IMPORTED SHARED library
                            writeln!(output, "add_library({} SHARED IMPORTED GLOBAL)", unit.name)?;
                            // 2. Set the property pointing directly to the pre-compiled file path
                            writeln!(
                                output,
                                "set_target_properties({} PROPERTIES IMPORTED_LOCATION \"${{CMAKE_CURRENT_SOURCE_DIR}}/{}\")",
                                unit.name,
                                src_paths.replace('"', "")
                            )?;
                        }
                    }
                    if let Some(includes) = &unit.include {
                        for inc in includes {
                            let relative_include_path = inc.strip_prefix(base_dir).unwrap_or(inc);
                            writeln!(
                                output,
                                "target_include_directories({} PRIVATE {})",
                                unit.name,
                                relative_include_path.to_string_lossy()
                            )?;
                        }
                    }

                    if !unit.resolved_deps.is_empty() {
                        writeln!(output, "# === DEPS: {} ===", &unit.name)?;
                        write!(output, "target_link_libraries({} PRIVATE ", unit.name)?;
                        let unit_deps = &unit
                            .resolved_deps
                            .iter()
                            .map(|(dep_name, _, _)| dep_name.clone())
                            .collect::<Vec<_>>()
                            .join(" ");
                        writeln!(output, "{})", unit_deps)?;
                    }
                    writeln!(output)?;
                }
            } else {
                anyhow::bail!(
                    "Unsupported version of CMake: {:?}",
                    lock.manifest.build.build_sys_ver
                )
            }
        }
    }
    Ok(output.trim_end().to_string())
}
