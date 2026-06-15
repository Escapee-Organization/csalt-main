// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use std::error::Error;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn verify_workspace(workspace: &str) -> Result<(), Box<dyn Error>> {
    // Check if the workspace directory exists
    // Then, check if the directory's structure is valid (Salt.toml, Salt.lock, .csalt/, etc)
    // If there is no manual Makefile or CMakeLists.txt, etc, update the Salt.lock file
    // Finally, return Ok(true)
    let path = Path::new(workspace);
    if !path.is_dir() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "[ERROR]\nWorkspace directory not found",
        )));
    }

    // Check if the directory's structure is valid
    if !path.join("Salt.toml").is_file() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "[ERROR]\n'Salt.toml' not found",
        )));
    }
    if !path.join("Salt.lock").is_file() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "[ERROR]\n'Salt.lock' not found",
        )));
    }
    if !path.join(".csalt").is_dir() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "[ERROR]\n'.csalt/' hidden directory not found",
        )));
    }

    // Check if there is no manual Makefile or CMakeLists.txt, etc
    if path.join("Makefile").is_file() || path.join("CMakeLists.txt").is_file() {
        // Edit the Salt.lock file
        println!("[warning]\nManual Makefile or CMakeLists.txt found, updating 'Salt.lock'");
    }

    Ok(())
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
    // Rewrite the Salt.toml and Salt.lock files with Serde
    let toml_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
authors = [""]
main = "main.c"

[build]
target = ""
edition = "2026"

[dependencies]

[settings]
"#,
        dir.to_str().unwrap()
    );

    let mut toml_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(Path::new(dir).join("Salt.toml"))?;
    let mut lock_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(Path::new(dir).join("Salt.lock"))?;
    writeln!(toml_file, "{}", toml_content)?;
    writeln!(lock_file, "")?;

    let path = Path::new(dir);
    fs::create_dir_all(path.join("src"))?;
    fs::create_dir_all(path.join("out"))?;
    fs::create_dir_all(path.join(".csalt"))?;

    // First check if there are any files within the src directory
    // If empty, write in a hello world with a return 0 and import stdio.h
    if fs::read_dir(path.join("src"))?.next().is_none() {
        let mut main_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(Path::new(dir).join("src").join("main.c"))?;
        writeln!(main_file, "#include <stdio.h>")?;
        writeln!(main_file, "")?;
        writeln!(main_file, "int main() {{")?;
        writeln!(main_file, "    printf(\"Hello, World!\\n\");")?;
        writeln!(main_file, "    return 0;")?;
        writeln!(main_file, "}}")?;
    }

    Ok(())
}
