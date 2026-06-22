// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use std::fs;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;

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
        dir.file_name().and_then(|n| n.to_str()).unwrap_or("")
    );

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(Path::new(dir).join("Salt.toml"))
    {
        Ok(mut toml_file) => writeln!(toml_file, "{}", toml_content)?,
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            println!("Salt.toml already exists: {}", e);
        }
        Err(e) => {
            return Err(Box::new(e));
        }
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
