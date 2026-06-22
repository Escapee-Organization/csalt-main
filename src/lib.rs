// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use clap::{ArgAction, Parser};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod fs_utils;
pub mod transpile;

/// csalt - A CLI tool and language that just works with C
#[derive(Parser, Debug)]
#[command(author = "BurningHot687", version, about, long_about = None, name = "csalt")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    #[command(name = "init")]
    Init {
        /// The directory to initialize the project in
        #[arg(default_value = ".")]
        dir: PathBuf,
    },

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

    #[command(name = "build")]
    Compile(CompileArgs),
}

#[derive(Parser, Debug)]
pub struct CompileArgs {
    /// The main input source file or target directory
    #[arg(default_value = ".")]
    input: String,

    /// Explicitly set the output binary file destination
    #[arg(short = 'o', long = "output", default_value = "main")]
    output: String,

    // TODO: Make this Option<String> so we can check the .toml for the backend
    /// Choose the host compiler driver backend [possible values: clang, gcc, zig]
    #[arg(short = 'b', long = "backend", default_value = "clang")]
    backend: String,

    /// Trailing parameters forwarded completely intact to the backend compiler layer
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, action = ArgAction::Append)]
    backend_flags: Vec<String>,
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

pub fn run(workspace: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs_utils::verify_workspace(workspace)?;

    // TODO: run the engine
    Ok(())
}

/*  To compile a project, we must follow specific steps:
 *  1. Check Salt.lock's cache to see what changed
 *  2. Run the header file engine
 *  3. Transpile the source code
 *  4. Link the transpiled code with the backend compiler
 *  5. Output the compiled binary to out/
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
    fs::create_dir_all(&cache_dir)?;

    if src_dir.exists() && src_dir.is_dir() {
        for entry in fs::read_dir(&src_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    let dest_path = cache_dir.join(file_name);
                    fs::copy(&path, &dest_path)?;
                }
            }
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
    } else {
        for input_target in args.input.split_whitespace() {
            let input_path = Path::new(input_target);
            if let Some(file_name) = input_path.file_name() {
                let cached_target = cache_dir.join(file_name);
                if cached_target.exists() {
                    files_to_compile.push(cached_target);
                } else {
                    println!(
                        "[warning]\nTarget source file not found in staging cache: {:?}",
                        file_name
                    );
                }
            }
        }
    }

    if files_to_compile.is_empty() {
        return Err("No files to compile".into());
    }

    // TODO: Transpile the input files
    // transpile::transpile(...)?;

    // TODO: Add in the .toml check using short-circuit evaluation
    // Read in the target compiler from the .toml, otherwise use the default (clang). CLI flag overrides
    let mut target_compiler = Command::new(&args.backend);
    let binary_name = &args.output;
    let output_executable = out_bin_dir.join(binary_name);

    for file in &files_to_compile {
        target_compiler.arg(file.to_str().unwrap());
    }

    target_compiler.arg("-o");
    target_compiler.arg(&output_executable);

    for flag in &args.backend_flags {
        target_compiler.arg(flag);
    }

    let status = target_compiler.status()?;
    if !status.success() {
        return Err("Failed to compile".into());
    }

    Ok(())
}
