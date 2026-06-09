// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

#[warn(unused_imports)] // TODO: Use all imports
use clap::{ArgAction, Parser};
use csalt::run;
use dirs::home_dir;
use std::io;
use std::path::PathBuf;

/// csalt - A CLI tool and language that just works with C
#[derive(Parser, Debug)]
#[command(author = "BurningHot687", version, about, long_about = None, name = "csalt")]
struct Args {
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

fn ensure_cache_dir() -> Result<PathBuf, io::Error> {
    let home = home_dir().ok_or(io::Error::new(
        io::ErrorKind::NotFound,
        "[ERROR]\nhome directory not found",
    ))?;
    let cache_dir = home.join(".csalt");
    std::fs::create_dir_all(&cache_dir).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(cache_dir)
}

fn main() {
    if let Err(e) = ensure_cache_dir() {
        eprintln!("[ERROR]\n{}", e);
        std::process::exit(1);
    }

    let args = Args::parse();
    println!("Hello, world!");

    let input = &args.input;
    let _output = &args.output;
    let _backend = &args.backend;
    let _backend_flags = &args.backend_flags;

    if let Err(e) = run(&input) {
        eprintln!("[ERROR]\n{}", e);
        std::process::exit(1);
    }
}
