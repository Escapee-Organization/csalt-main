// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::config::{self, SaltLock, SaltToml};
use crate::helpers::LOCK_VERSION;
use crate::helpers::verify_command;
use dirs::home_dir;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn ensure_cache_dir() -> anyhow::Result<PathBuf> {
    let home = home_dir().ok_or(anyhow::anyhow!("Home directory not found"))?;
    let cache_dir = home.join(".csalt");
    fs::create_dir_all(&cache_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create cache directory: {}", e))?;
    Ok(cache_dir)
}

pub fn verify_workspace(base_dir: &Path) -> anyhow::Result<()> {
    let manifest_path = base_dir.join("Salt.toml");
    if !manifest_path.exists() {
        anyhow::bail!("Invalid C-Salt project workspace (missing Salt.toml)");
    }
    Ok(())
}

pub fn clean_cache_dir(
    base_dir: Option<PathBuf>,
    build_dir: Option<PathBuf>,
) -> anyhow::Result<()> {
    let base_directory = base_dir
        .unwrap_or(std::env::current_dir()?)
        .canonicalize()?;
    verify_workspace(&base_directory)?;
    let cache_dir = base_directory.join(".csalt");
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
    }
    fs::create_dir_all(cache_dir).map_err(io::Error::other)?;

    let build_dir = base_directory.join(build_dir.unwrap_or_else(|| PathBuf::from("build")));
    fs::create_dir_all(&build_dir).map_err(io::Error::other)?;
    Ok(())
}

/// Walks through the directory and filters out excluded directories and files, and files with unsupported extensions. Excluded directories only apply to the current directory level.
///
/// ### Examples
/// ```
/// use csalt::fs_utils::walk_and_filter_dirs;
/// use tempfile::tempdir;
///
/// let binding = tempdir().unwrap();
/// let base_dir = binding.path();
/// std::fs::write(&base_dir.join("test.txt"), "Hello, World!").unwrap();
/// std::fs::create_dir(&base_dir.join("excluded")).unwrap();
/// std::fs::write(&base_dir.join("excluded").join("test.md"), " ").unwrap();
/// std::fs::write(&base_dir.join("excluded").join("test.txt"), " ").unwrap();
/// std::fs::write(&base_dir.join("special.txt"), " ").unwrap();
///
/// let files = walk_and_filter_dirs(&base_dir, &["excluded"], &["special.txt"], &["md"]).unwrap();
/// assert_eq!(files.len(), 1);
/// ```
pub fn walk_and_filter_dirs(
    base_dir: &Path,
    excluded_dirs: &[&str],
    excluded_files: &[&str],
    extension_filter: &[&str],
) -> anyhow::Result<Vec<PathBuf>> {
    let mut discovered_files = Vec::new();
    let mut stack = vec![base_dir.to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        for entry in fs::read_dir(&current_dir)? {
            let entry = entry?;
            let path = entry.path();
            let is_dir = path.is_dir();
            let file_name_os = path.file_name().unwrap_or_default();

            if current_dir == base_dir {
                if excluded_dirs.iter().any(|dir| file_name_os == *dir) && is_dir {
                    continue;
                }

                if excluded_files.iter().any(|file| file_name_os == *file) {
                    continue;
                }
            }

            if !is_dir {
                if !extension_filter.is_empty()
                    && extension_filter
                        .iter()
                        .any(|ext| path.extension().and_then(|e| e.to_str()).unwrap_or("") == *ext)
                {
                    continue;
                }

                discovered_files.push(path);
                continue;
            }

            stack.push(path);
        }
    }

    Ok(discovered_files)
}

/// Copies project files to the cache directory, excluding `Salt.lock`, `Salt.toml`, and others
/// TODO: Consider using `Salt.lock` to exclude unnecessary file copying and cache cleaning
pub fn copy_project_files(
    base_dir: &Path,
    cache_dir: &Path,
    build_dir: &Path,
) -> anyhow::Result<()> {
    clean_cache_dir(Some(base_dir.to_path_buf()), Some(build_dir.to_path_buf()))?;
    let excluded_dirs = [".csalt", ".git", "build"];
    let excluded_files = ["Salt.toml", "Salt.lock", ".gitignore"];

    let filtered_files = walk_and_filter_dirs(base_dir, &excluded_dirs, &excluded_files, &[])?;

    for file in filtered_files {
        let relative_path = file
            .strip_prefix(base_dir)
            .map_err(|_| anyhow::anyhow!("Failed to strip prefix from {:?}", file))?;
        let target_path = cache_dir.join(relative_path);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&file, &target_path)?;
    }

    Ok(())
}

/// Initializes the Salt.toml file in the project directory.
fn init_salt_toml(project_name: &str, dir: &Path) -> anyhow::Result<()> {
    let toml_content = SaltToml {
        package: config::PackageSection {
            name: project_name.to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["".to_string()],
            description: "".to_string(),
        },
        build: config::BuildSection {
            build_sys: None,
            build_sys_ver: None,
            build_dir: Some(PathBuf::from("build/")),
            edition: config::CEditions::C11,
            compiler: Some(config::CompilerBackend::Clang),
        },
        unit: vec![config::UnitVector {
            name: project_name.to_string(),
            kind: config::UnitKinds::Bin,
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
        toml_content.validate(dir)?;
        println!("Salt.toml already exists, skipping creation.");
    }

    Ok(())
}

/// Initializes all directories for the project.
///
/// Creates only `src/`, `include/`, `build/`, and `.csalt/` directories by default
/// with optional `tests/` and `vendor/` directories and a `README.md` file.
fn init_all_directories(full: bool, project_name: &str, dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join("include"))?;
    fs::create_dir_all(dir.join("build"))?;
    fs::create_dir_all(dir.join(".csalt"))?;
    if full {
        fs::create_dir_all(dir.join("tests"))?;
        fs::create_dir_all(dir.join("vendor"))?;
        if let Ok(false) = fs::exists(dir.join("README.md")) {
            let mut read_me = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(dir.join("README.md"))?;
            writeln!(read_me, "# {}\n", project_name)?;
        }
    }

    Ok(())
}

/// Writes the `main.c` file to the `src/` directory **if** it doesn't exist.
///
/// `main.c`:
/// ```c
/// #include <stdio.h>
///
/// int main() {
///     printf("Hello, World!\n");
///     return 0;
/// }
/// ```
fn init_and_write_main_c(dir: &Path) -> anyhow::Result<()> {
    if fs::read_dir(dir.join("src"))?.next().is_none() {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join("src").join("main.c"))
        {
            Ok(mut main_file) => {
                writeln!(main_file, "#include <stdio.h>")?;
                writeln!(main_file)?;
                writeln!(main_file, "int main() {{")?;
                writeln!(main_file, "    printf(\"Hello, World!\\n\");")?;
                writeln!(main_file, "    return 0;")?;
                writeln!(main_file, "}}")?;
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => {
                anyhow::bail!("Failed to write main.c: {}", e);
            }
        }
    }

    Ok(())
}

/// Returns the lock file, initializing it if it doesn't exist or is empty.
pub fn load_or_init_lock(current_toml: &SaltToml) -> anyhow::Result<SaltLock> {
    let lock_path = Path::new("Salt.lock");
    let perfect_salt_lock = SaltLock {
        lock_version: LOCK_VERSION.to_string(),
        manifest: current_toml.clone(),
    };

    if !lock_path.is_file() {
        return Ok(perfect_salt_lock);
    }

    let contents = fs::read_to_string(lock_path)?;
    if contents.trim().is_empty() {
        return Ok(perfect_salt_lock);
    }

    let lock =
        serde_json::from_str::<SaltLock>(&contents).unwrap_or_else(|_| perfect_salt_lock.clone());
    if lock.manifest != *current_toml {
        return Ok(perfect_salt_lock);
    }
    Ok(lock)
}

/// Creates a new project in the current directory or the specified directory.
///
/// # Arguments
///
/// * `name` - The name of the project.
/// * `dir` - The directory to create the project in.
/// * `full` - Whether to create a full project (see [`init_all_directories`])
/// * `stealth` - Whether to add configuration files to `.gitignore`.
/// * `init_git` - Whether to initialize Git.
pub fn new_project(
    name: &str,
    dir: Option<&str>,
    full: bool,
    stealth: bool,
    init_git: bool,
) -> anyhow::Result<()> {
    let path = Path::new(&dir.unwrap_or(".")).join(name);
    fs::create_dir_all(&path)?;
    init_project(&path, full, stealth, init_git)?;

    Ok(())
}

/// Initializes a project in the specified directory.
///
/// # Arguments
///
/// * `dir` - The directory to initialize the project in.
/// * `full` - Whether to create a full project (see [`init_all_directories`])
/// * `stealth` - Whether to add configuration files to `.gitignore`.
/// * `init_git` - Whether to initialize Git.
pub fn init_project(dir: &Path, full: bool, stealth: bool, init_git: bool) -> anyhow::Result<()> {
    fs::create_dir_all(dir)?;

    let project_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    init_salt_toml(project_name, dir)?;

    if !dir.join("Salt.lock").exists() {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .truncate(true)
            .open(dir.join("Salt.lock"))
        {
            Ok(mut lock_file) => writeln!(lock_file)?,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                println!("Salt.lock already exists: {}", e);
            }
            Err(e) => {
                anyhow::bail!("Failed to create Salt.lock: {}", e);
            }
        }
    }

    init_all_directories(full, project_name, dir)?;

    init_and_write_main_c(dir)?;

    let gitignore_path = dir.join(".gitignore");
    if fs::exists(&gitignore_path).is_err() {
        let mut gitignore = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&gitignore_path)?;
        writeln!(gitignore, "build/")?;
        writeln!(gitignore, ".csalt/")?;

        if stealth {
            writeln!(gitignore, "Salt.toml")?;
            writeln!(gitignore, "Salt.lock")?;
        }
    }

    if init_git {
        verify_command("git")?;
        Command::new("git")
            .current_dir(dir)
            .args(["init", "--initial-branch=main"])
            .status()
            .ok();
    }

    Ok(())
}
