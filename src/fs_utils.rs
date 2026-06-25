// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::config::{BuildSection, PackageSection, SaltToml};
use dirs::home_dir;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};

pub fn ensure_cache_dir() -> Result<PathBuf, Error> {
    let home = home_dir().ok_or(Error::new(
        ErrorKind::NotFound,
        "[ERROR]\nhome directory not found",
    ))?;
    let cache_dir = home.join(".csalt");
    std::fs::create_dir_all(&cache_dir).map_err(|e| Error::new(ErrorKind::Other, e))?;
    Ok(cache_dir)
}

pub fn new_project(name: &str, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Make new directory, move into it, and create all elements
    let path = Path::new(dir).join(name);
    fs::create_dir_all(&path)?;
    init_project(&path)?;

    Ok(())
}

pub fn init_project(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;

    let project_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    // Rewrite the Salt.toml and Salt.lock files with Serde
    let toml_content = SaltToml {
        package: PackageSection {
            name: project_name.to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["".to_string()],
            description: "".to_string(),
        },
        build: BuildSection {
            main: "main.c".to_string(),
            build: "cmake3.28".to_string(),
            edition: "2011".to_string(),
            compiler: "clang".to_string(),
            settings: std::collections::BTreeMap::new(),
        },
    };

    if !dir.join("Salt.toml").exists() {
        fs::write(
            dir.join("Salt.toml"),
            toml::to_string_pretty(&toml_content)?,
        )?;
    } else {
        println!("Salt.toml already exists, skipping creation.");
    }

    match OpenOptions::new()
        .write(true)
        .create(true)
        .open(Path::new(dir).join("Salt.lock"))
    {
        Ok(mut lock_file) => writeln!(lock_file, "")?,
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            println!("Salt.lock already exists: {}", e);
        }
        Err(e) => {
            return Err(Box::new(e));
        }
    }

    let path = Path::new(dir);
    fs::create_dir_all(path.join("src"))?;
    fs::create_dir_all(path.join("include"))?;
    fs::create_dir_all(path.join("build"))?;
    fs::create_dir_all(path.join("tests"))?;
    fs::create_dir_all(path.join(".csalt"))?;

    // First check if there are any files within the src directory
    // If empty, write in a hello world with a return 0 and import stdio.h
    if fs::read_dir(path.join("src"))?.next().is_none() {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(Path::new(dir).join("src").join("main.c"))
        {
            Ok(mut main_file) => {
                writeln!(main_file, "#include <stdio.h>")?;
                writeln!(main_file, "")?;
                writeln!(main_file, "int main() {{")?;
                writeln!(main_file, "    printf(\"Hello, World!\\n\");")?;
                writeln!(main_file, "    return 0;")?;
                writeln!(main_file, "}}\n")?;
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                println!("main.c already exists");
            }
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }

    match fs::exists(Path::new(dir).join("README.md")) {
        Ok(false) => {
            let mut read_me = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(Path::new(dir).join("README.md"))?;
            writeln!(
                read_me,
                "# {}\n",
                dir.file_name().and_then(|n| n.to_str()).unwrap_or("")
            )?;
        }
        _ => {}
    }

    match fs::exists(Path::new(dir).join(".gitignore")) {
        Ok(false) => {
            let mut gitignore = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(Path::new(dir).join(".gitignore"))?;
            writeln!(gitignore, "build/")?;
            writeln!(gitignore, ".cache/")?;
            writeln!(gitignore, ".csalt/")?;
        }
        _ => {}
    }

    Ok(())
}
