// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

// TODO: Use all imports

use clap::Parser;
use csalt::{Args, Commands, CompileArgs, compile_project, run};
use dirs::home_dir;
use std::io;
use std::path::PathBuf;
pub mod fs_utils;

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

    if let Err(e) = run(&input) {
        eprintln!("[ERROR]\n{}", e);
        std::process::exit(1);
    }
}
