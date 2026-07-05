// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

#![deny(warnings)]
use clap::Parser;
use csalt::cli::{Args, Commands};
#[cfg(feature = "experimental")]
use csalt::update_csalt;
use csalt::{build_manual_project, fs_utils};

fn main() {
    if let Err(e) = fs_utils::ensure_cache_dir() {
        eprintln!("[ERROR]\n{}", e);
        std::process::exit(1);
    }

    let args = Args::parse();

    match &args.command {
        Commands::Init { dir } => {
            if let Err(e) = fs_utils::init_project(dir, false, false, false) {
                eprintln!("[ERROR]\n{}", e);
                std::process::exit(1);
            }
            println!("[info]\nProject directory initialized successfully");
        }
        Commands::New(new_args) => {
            if let Err(e) = fs_utils::new_project(new_args) {
                eprintln!("[ERROR]\n{}", e);
                std::process::exit(1);
            }
            println!(
                "[info]\nNew project '{}' created successfully",
                new_args.name
            );
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
