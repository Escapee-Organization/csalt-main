// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use clap::{ArgAction, Parser};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub mod config;
pub mod fs_utils;
pub mod transpile;

/// csalt - A CLI tool and language that just works with C
#[derive(Parser, Debug)]
#[command(author = "Escapee-Organization", version, about, long_about = None, name = "csalt")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    /// Initialize a new project in the current directory
    #[command(name = "init")]
    Init {
        /// The directory to initialize the project in
        #[arg(default_value = ".")]
        dir: PathBuf,
    },

    /// Create a new project directory
    #[command(name = "new")]
    New {
        /// The name of the new project
        #[arg(required = true)]
        name: String,

        /// The directory to create the new project in
        #[arg(default_value = ".")]
        dir: PathBuf,
    },

    /// Experimental: Update the csalt binary to the latest version
    #[cfg(feature = "experimental")]
    #[command(name = "update")]
    Update,

    /// Build using a compiler
    #[command(name = "compile")]
    Compile(CompileArgs),

    #[cfg(feature = "experimental")]
    /// Build using a build system
    #[command(name = "build")]
    Build(BuildArgs),
}

#[derive(Parser, Debug)]
pub struct CompileArgs {
    /// Choose the host compiler driver backend, such as clang, gcc, zig, etc
    #[arg(short = 'b', long = "backend")]
    backend: Option<String>,

    /// Trailing parameters forwarded completely intact to the backend compiler layer
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, action = ArgAction::Append)]
    backend_flags: Vec<String>,
}

#[cfg(feature = "experimental")]
#[derive(Parser, Debug)]
pub struct BuildArgs {
    /// Choose the host build system backend, such as cmake3_15, zig, etc
    #[arg(short = 'b', long = "backend")]
    backend: Option<String>,

    /// Trailing parameters forwarded completely intact to the backend compiler layer
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, action = ArgAction::Append)]
    backend_flags: Vec<String>,
}

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

/*  To compile a project manually, we must follow specific steps:
 *  1. Check Salt.lock's cache to see what changed
 *  2. Run the header file engine
 *  3. Transpile the source code
 *  4. Link the transpiled code with the backend compiler
 *  5. Output the compiled binary to build/
 */
pub fn build_manual_project(args: &CompileArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("[info]\nCompiling project...");

    let base_dir = std::env::current_dir()?;
    fs_utils::init_project(&base_dir)?;
    let cache_dir = base_dir.join(".csalt");
    let src_dir = base_dir.join("src");
    // TODO: Consider a more professional output directory
    let out_bin_dir = base_dir.join("build").join("bin");
    fs::create_dir_all(&out_bin_dir)?;

    // FIXME: Copy all files in the main directory EXCEPT .csalt/, Salt.toml, and Salt.lock to .csalt/
    for entry in fs::read_dir(&base_dir)? {
        let entry = entry?;
        let path = entry.path();
        // Match against path name to see if we should copy it
        match path.file_name().and_then(|n| n.to_str()) {
            Some(".csalt") => {}
            Some("Salt.toml") => {}
            Some("Salt.lock") => {}
            Some(file_name) => {
                let dest_path = cache_dir.join(file_name);
                fs::copy(&path, &dest_path)?;
            }
            None => {}
        }
    }

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

    // Read in the target compiler from the .toml, otherwise use the default (clang). CLI flag overrides
    let compiler_backend = CompilerBackend::from_string(args.backend.as_str())?;
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
