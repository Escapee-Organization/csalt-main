// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use clap::{ArgAction, Parser, Subcommand};
use std::path::PathBuf;

pub mod engine;
pub mod fs_utils;

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

    #[command(name = "build")]
    Compile(CompileArgs),
}

#[derive(Parser, Debug)]
pub struct CompileArgs {
    /// The main input source file or target directory
    #[arg(required = true, default_value = ".")]
    input: String,

    /// Explicitly set the output binary file destination
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Choose the host compiler driver backend [possible values: clang, gcc, zig]
    #[arg(short = 'b', long = "backend")]
    backend: String,

    /// Trailing parameters forwarded completely intact to the backend compiler layer
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, action = ArgAction::Append)]
    backend_flags: Vec<String>,
}

pub fn run(workspace: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs_utils::verify_workspace(workspace)?;

    // TODO: run the engine
    Ok(())
}

pub fn compile_project(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
