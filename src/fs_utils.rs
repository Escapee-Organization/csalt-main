// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::cli::NewArgs;
use crate::config::{
    BuildSection, BuildSystems, CompilerBackend, PackageSection, SaltToml, UnitKinds, UnitVector,
};
use crate::verify_command;
use dirs::home_dir;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn ensure_cache_dir() -> Result<PathBuf, Error> {
    let home = home_dir().ok_or(Error::new(
        ErrorKind::NotFound,
        "[ERROR]\nhome directory not found",
    ))?;
    let cache_dir = home.join(".csalt");
    std::fs::create_dir_all(&cache_dir).map_err(|e| Error::new(ErrorKind::Other, e))?;
    Ok(cache_dir)
}

pub fn verify_workspace(base_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = base_dir.join("Salt.toml");
    if !manifest_path.exists() {
        return Err("Fatal: Not a valid C-Salt project workspace (missing Salt.toml)".into());
    }
    Ok(())
}

// TODO: Consider using `Salt.lock` to exclude unnecessary file copying
pub fn copy_project_files(
    base_dir: &Path,
    cache_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let excluded_dirs = [".csalt", ".git", "build"];
    let excluded_files = ["Salt.toml", "Salt.lock", ".gitignore"];

    let mut stack = vec![base_dir.to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        for entry in fs::read_dir(&current_dir)? {
            let entry = entry?;

            let is_dir = entry.file_type()?.is_dir();
            let file_name = entry.file_name();
            if let Some(name) = file_name.to_str() {
                if current_dir == base_dir {
                    if is_dir && excluded_dirs.contains(&name) {
                        continue;
                    }

                    if !is_dir && excluded_files.contains(&name) {
                        continue;
                    }
                }
            }
            let path = entry.path();
            let relative_path = path
                .as_path()
                .strip_prefix(&base_dir)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            let target_path = cache_dir.join(relative_path);

            if is_dir {
                fs::create_dir_all(&target_path)?;
                stack.push(path);
            } else {
                fs::copy(&path, &target_path)?;
            }
        }
    }

    Ok(())
}

pub fn new_project(args: &NewArgs) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(&args.dir.as_deref().unwrap_or(".")).join(&args.name);
    fs::create_dir_all(&path)?;
    init_project(&path, args.full, args.stealth, args.init_git)?;

    Ok(())
}

pub fn init_project(
    dir: &Path,
    full: bool,
    stealth: bool,
    init_git: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;

    let project_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    let toml_content = SaltToml {
        package: PackageSection {
            name: project_name.to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["".to_string()],
            description: "".to_string(),
        },
        build: BuildSection {
            build_system: BuildSystems::CMake3_15,
            build_dir: Some(PathBuf::from("build/")),
            edition: "2011".to_string(),
            compiler: CompilerBackend::Clang,
        },
        unit: vec![UnitVector {
            name: project_name.to_string(),
            kind: UnitKinds::Bin,
            main: PathBuf::from("src/main.c"),
            src: vec![PathBuf::from("src/")],
            include: Some(vec![PathBuf::from("include/")]),
            deps: None,
            compiler_flags: None,
            linker_flags: None,
        }],
    };

    if !dir.join("Salt.toml").exists() {
        fs::write(
            dir.join("Salt.toml"),
            toml::to_string_pretty(&toml_content)?,
        )?;
    } else {
        toml_content.validate()?;
        println!("Salt.toml already exists, skipping creation.");
    }

    if !dir.join("Salt.lock").exists() {
        match OpenOptions::new()
            .write(true)
            .create(true)
            .open(dir.join("Salt.lock"))
        {
            Ok(mut lock_file) => writeln!(lock_file, "")?,
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                println!("Salt.lock already exists: {}", e);
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }

    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join("include"))?;
    fs::create_dir_all(dir.join("build"))?;
    fs::create_dir_all(dir.join(".csalt"))?;
    if full {
        fs::create_dir_all(dir.join("tests"))?;
        fs::create_dir_all(dir.join("vendor"))?;
        match fs::exists(dir.join("README.md")) {
            Ok(false) => {
                let mut read_me = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(dir.join("README.md"))?;
                writeln!(
                    read_me,
                    "# {}\n",
                    dir.file_name().and_then(|n| n.to_str()).unwrap_or("")
                )?;
            }
            _ => {}
        }
    }

    if fs::read_dir(dir.join("src"))?.next().is_none() {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join("src").join("main.c"))
        {
            Ok(mut main_file) => {
                writeln!(main_file, "#include <stdio.h>")?;
                writeln!(main_file, "")?;
                writeln!(main_file, "int main() {{")?;
                writeln!(main_file, "    printf(\"Hello, World!\\n\");")?;
                writeln!(main_file, "    return 0;")?;
                writeln!(main_file, "}}")?;
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                println!("main.c already exists");
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }

    match fs::exists(dir.join(".gitignore")) {
        Ok(false) => {
            let mut gitignore = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(dir.join(".gitignore"))?;
            writeln!(gitignore, "build/")?;
            writeln!(gitignore, ".csalt/")?;
            if stealth {
                writeln!(gitignore, "Salt.toml")?;
                writeln!(gitignore, "Salt.lock")?;
            }
        }
        _ => {}
    }

    if init_git {
        verify_command("git")?;
        Command::new("git")
            .current_dir(&dir)
            .args(["init", "--initial-branch=main"])
            .status()
            .ok();
    }

    Ok(())
}
