// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

// TODO: Use all imports

#![deny(warnings)]
use clap::Parser;
use csalt::{Args, Commands, compile_project, update_csalt};
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

    match &args.command {
        Commands::Init { dir } => {
            if let Err(e) = fs_utils::init_project(dir) {
                eprintln!("[ERROR]\n{}", e);
                std::process::exit(1);
            }
        }
        Commands::New { name, dir } => {
            if let Err(e) = fs_utils::new_project(name, &dir) {
                eprintln!("[ERROR]\n{}", e);
                std::process::exit(1);
            }
        }
        Commands::Update => {
            if let Err(e) = update_csalt() {
                eprintln!("[ERROR]\n{}", e);
                std::process::exit(1);
            }
        }
        Commands::Compile(salt_args) => {
            if let Err(e) = compile_project(salt_args) {
                eprintln!("[ERROR]\n{}", e);
                std::process::exit(1);
            }
        }
    }
}
