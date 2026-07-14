// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

#![deny(warnings)]
use anyhow::Context;
use clap::Parser;
use csalt::cli::{Args, Commands};
#[cfg(feature = "experimental")]
use csalt::update_csalt;
use csalt::{build_managed_project, build_manual_project, emit_project, fs_utils};

// --------------- FUNCTIONS ---------------

fn run_csalt() -> anyhow::Result<()> {
    fs_utils::ensure_cache_dir().context("Failed to ensure cache directory")?;

    let args = Args::parse();

    match &args.command {
        Commands::Init { dir } => {
            fs_utils::init_project(dir, false, false, false)?;
            println!("[info]\nProject directory initialized successfully");
        }
        Commands::New(new_args) => {
            fs_utils::new_project(new_args)?;
            println!(
                "[info]\nNew project '{}' created successfully",
                new_args.name
            );
        }
        #[cfg(feature = "experimental")]
        Commands::Update => {
            update_csalt()?;
            println!("[info]\nCsalt updated successfully");
        }
        Commands::Compile(salt_args) => {
            build_manual_project(salt_args)?;
            println!("[info]\nProject compiled successfully");
        }

        Commands::Emit(emit_args) => {
            let base_dir = match emit_args.path.as_deref() {
                Some(path) => std::path::Path::new(path).canonicalize()?,
                None => std::env::current_dir().context("Failed to get current directory")?,
            };
            let cache_dir = base_dir.join(".csalt");
            let toml = toml::from_str(&std::fs::read_to_string(base_dir.join("Salt.toml"))?)?;
            let lock = csalt::load_or_init_lock(&toml)?;
            let build_dir = base_dir.join(
                lock.manifest
                    .build
                    .build_dir
                    .as_deref()
                    .unwrap_or(std::path::Path::new("build")),
            );
            std::fs::create_dir_all(&build_dir)?;
            let build_dir = build_dir.canonicalize()?;
            emit_project(&base_dir, &cache_dir, &build_dir, emit_args.build_file)?;
            println!("[info] Project emitted successfully");
        }

        Commands::Clean { path } => {
            fs_utils::clean_cache_dir(path.clone())?;
            println!("[info]\nCache directory cleaned successfully");
        }

        Commands::Build(_build_args) => {
            build_managed_project(_build_args)?;
            println!("[info]\nProject built successfully");
        }
    }
    Ok(())
}

// ==========================================
// ================== MAIN ==================
// ==========================================

fn main() {
    if let Err(e) = run_csalt() {
        eprintln!("\x1b[1;31m[ERROR]\x1b[0m\n{}", e);
        std::process::exit(1);
    }
}
