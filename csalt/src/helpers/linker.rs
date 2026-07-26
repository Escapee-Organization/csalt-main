// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::helpers;
use anyhow::Context;

pub enum LinkerDriver {
    GnuLike {
        backend: crate::config::CompilerBackend,
        zig_target: Option<String>,
    },
    ClLike,
}

impl LinkerDriver {
    pub fn new(backend: &crate::config::CompilerBackend, zig_target: Option<String>) -> Self {
        match backend {
            crate::config::CompilerBackend::Clang
            | crate::config::CompilerBackend::Gcc
            | crate::config::CompilerBackend::Zig => Self::GnuLike {
                backend: backend.clone(),
                zig_target,
            },
            #[cfg(feature = "experimental")]
            crate::config::CompilerBackend::Msvc | crate::config::CompilerBackend::ClangCl => {
                Self::ClLike
            }
        }
    }

    pub fn build_flags(
        &self,
        cmd: &mut std::process::Command,
        unit: &crate::PreparedUnit,
        out_dyn: &std::path::Path,
        dyn_name: &str,
        out_exe: &std::path::Path,
    ) -> anyhow::Result<()> {
        match self {
            LinkerDriver::GnuLike {
                backend,
                zig_target,
            } => {
                helpers::attach_zig_target_arg(backend.clone(), cmd, zig_target.clone());

                if unit.kind == crate::UnitKinds::Dyn {
                    cmd.arg("-shared").arg("-fPIC").arg("-o").arg(out_dyn);
                    if cfg!(target_os = "macos") {
                        cmd.arg("-Xlinker")
                            .arg("-install_name")
                            .arg("-Xlinker")
                            .arg(format!("@rpath/{}", dyn_name));
                    }
                } else {
                    cmd.arg("-o").arg(out_exe);
                    if cfg!(target_os = "macos") {
                        cmd.arg("-Xlinker")
                            .arg("-rpath")
                            .arg("-Xlinker")
                            .arg("@executable_path");
                    }
                }

                // Global user linker flags
                cmd.arg("-L.").args(&unit.linker_flags);
                cmd.args(&unit.unpack_linker_flags);

                // Process dependent libraries
                for (dep_name, dep_kind, dep_path) in &unit.resolved_deps {
                    match dep_kind {
                        crate::UnitKinds::Lib | crate::UnitKinds::Dyn => {
                            cmd.arg(format!("-l{}", dep_name));
                        }
                        crate::UnitKinds::ExtLib | crate::UnitKinds::ExtDyn => {
                            let path = dep_path
                                .as_ref()
                                .context(format!("Missing path for: {}", dep_name))?;
                            cmd.arg(path.canonicalize()?);

                            if unit.kind == crate::UnitKinds::ExtDyn && cfg!(target_os = "macos") {
                                cmd.arg("-Xlinker")
                                    .arg("-rpath")
                                    .arg("-Xlinker")
                                    .arg("@executable_path")
                                    .arg(path);
                            }
                        }
                        _ => {}
                    }
                }
            }
            LinkerDriver::ClLike => {}
        }
        Ok(())
    }
}
