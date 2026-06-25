// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

#![deny(warnings)]
use clap::Parser;
#[cfg(feature = "experimental")]
use csalt::update_csalt;
use csalt::{Args, Commands, build_manual_project};
pub mod config;
pub mod fs_utils;

fn main() {
    if let Err(e) = fs_utils::ensure_cache_dir() {
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
            println!("[info]\nProject directory initialized successfully");
        }
        Commands::New { name, dir } => {
            if let Err(e) = fs_utils::new_project(name, &dir) {
                eprintln!("[ERROR]\n{}", e);
                std::process::exit(1);
            }
            println!("[info]\nNew project '{}' created successfully", name);
        }
        #[cfg(feature = "experimental")]
        Commands::Update => {
            if let Err(e) = update_csalt() {
                eprintln!("[ERROR]\n{}", e);
                std::process::exit(1);
            }
        }
        Commands::Compile(salt_args) => {
            if let Err(e) = build_manual_project(salt_args) {
                eprintln!("[ERROR]\n{}", e);
                std::process::exit(1);
            }
        }
        #[cfg(feature = "experimental")]
        Commands::Build(_salt_args) => {}
    }
}
