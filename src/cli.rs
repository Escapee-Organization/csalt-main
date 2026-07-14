// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use clap::{ArgAction, Parser};
use std::path::PathBuf;

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
    New(NewArgs),

    /// Experimental: Update the csalt binary to the latest version
    #[cfg(feature = "experimental")]
    #[command(name = "update")]
    Update,

    /// Build using a compiler
    #[command(name = "compile")]
    Compile(CompileArgs),

    /// Build using a build system
    #[command(name = "build")]
    Build(BuildArgs),

    /// Emit and generate the project
    #[command(name = "emit")]
    Emit(EmitArgs),

    /// Clean the cache directory (.csalt/)
    #[command(name = "clean")]
    Clean,
}

#[derive(Parser, Debug)]
pub struct NewArgs {
    /// Project name
    pub name: String,

    /// Project directory
    #[arg(short = 'd', long = "dir")]
    pub dir: Option<String>,

    /// Full project initialization, including creating a git repository
    #[arg(short = 'f', long = "full")]
    pub full: bool,

    /// Stealth mode, suppresses output messages
    #[arg(long = "stealth")]
    pub stealth: bool,

    /// Initializes git after creation
    #[arg(long = "git", short = 'g')]
    pub init_git: bool,
}

#[derive(Parser, Debug)]
pub struct CompileArgs {
    /// Choose the host compiler driver backend, such as clang, gcc, zig, etc
    #[arg(short = 'b', long = "backend")]
    pub backend: Option<String>,

    /// Choose the mode of transpilation, such as default, in-place, and clean
    #[arg(short = 'm', long = "mode")]
    pub mode: Option<String>,

    /// Run the compiled binary after transpilation
    #[arg(short = 'r', long = "run", conflicts_with = "backend_flags")]
    pub run: bool,

    #[cfg(feature = "experimental")]
    /// Debug mode, enables debug output
    #[arg(long = "debug")]
    pub debug: bool,

    /// Trailing parameters forwarded completely intact to the backend compiler layer
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, action = ArgAction::Append)]
    pub backend_flags: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct BuildArgs {
    /// Choose the host build system backend, such as cmake, zig, etc.
    #[arg(short = 'b', long = "backend")]
    pub backend: Option<String>,

    /// Trailing parameters forwarded completely intact to the backend compiler layer
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, action = ArgAction::Append)]
    pub backend_flags: Vec<String>,
}

// NOTE: Come up with better names for these commands later
#[derive(Parser, Debug)]
pub struct EmitArgs {
    /// Changes where the operation will be performed
    #[arg(long = "path")]
    pub path: Option<PathBuf>,

    /// Whether to emit a build file in `fresh` mode
    #[arg(long = "build-file")]
    pub build_file: bool,
}
