// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use crate::config::{SaltLock, SaltToml};
use crate::helpers::LOCK_VERSION;
use crate::helpers::verify_command;
use dirs::home_dir;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

// ------------------- DATA -------------------

pub struct TemplateFile {
    pub virtual_path: PathBuf,
    pub content: &'static str,
}

// ----------------- FUNCTIONS -----------------

fn overwrite_salt_toml_with_project_name(
    salt_toml_path: &Path,
    salt_toml_name: &str,
) -> anyhow::Result<()> {
    let salt_toml_str = fs::read_to_string(salt_toml_path)?;
    let mut salt_toml: SaltToml = toml::from_str(salt_toml_str.as_str())?;

    salt_toml.package.name = salt_toml_name.to_string();

    let new_toml_str = toml::to_string_pretty(&salt_toml)?;
    fs::write(salt_toml_path, new_toml_str)?;

    Ok(())
}

pub fn lookup_template(template_name: &str) -> anyhow::Result<Vec<TemplateFile>> {
    match template_name {
        "bin" => Ok(vec![
            TemplateFile {
                virtual_path: PathBuf::from("src/main.c"),
                content: include_str!("../examples/bin/src/main.c"),
            },
            TemplateFile {
                virtual_path: PathBuf::from("Salt.toml"),
                content: include_str!("../examples/bin/Salt.toml"),
            },
        ]),
        invalid_str => Err(anyhow::anyhow!("could not find template '{}'", invalid_str)),
    }
}

/// **Unused directory** - Creates the cache directory if it does not exist.
pub fn ensure_cache_dir() -> anyhow::Result<PathBuf> {
    let home = home_dir().ok_or(anyhow::anyhow!("Home directory not found"))?;
    let cache_dir = home.join(".csalt");
    fs::create_dir_all(&cache_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create cache directory: {}", e))?;
    Ok(cache_dir)
}

/// Verifies that the given directory is a valid C-Salt project workspace.
/// Checks if the directory contains a `Salt.toml` file.
///
/// ### Examples
///
/// ```
/// use csalt::fs_utils::verify_workspace;
///
/// let temp_dir = tempfile::tempdir().unwrap();
/// std::fs::write(&temp_dir.path().join("Salt.toml"), "foo").unwrap();
/// let result = verify_workspace(&temp_dir.path());
///
/// assert!(result.is_ok());
/// ```
pub fn verify_workspace(base_dir: &Path) -> anyhow::Result<()> {
    let manifest_path = base_dir.join("Salt.toml");
    if !manifest_path.exists() {
        anyhow::bail!("Invalid C-Salt project workspace (missing Salt.toml)");
    }
    Ok(())
}

/// Removes all files or subdirectories within the given directory.
///
/// ### Examples
///
/// ```
/// use csalt::fs_utils::clean_this_dir;
///
/// let temp_dir = tempfile::tempdir().unwrap();
/// std::fs::write(&temp_dir.path().join("foo"), "bar").unwrap();
/// clean_this_dir(&temp_dir.path(), false).unwrap();
///
/// assert!(!temp_dir.path().join("foo").exists());
/// ```
pub fn clean_this_dir(dir: &Path, verbose_on: bool) -> anyhow::Result<()> {
    // --- VERBOSE ---
    if verbose_on {
        println!("[info] Cleaning directory: {}", dir.display());
    }

    if dir.exists() {
        fs::remove_dir_all(dir).map_err(|e| anyhow::anyhow!("Failed to clean directory: {}", e))?;

        // --- VERBOSE ---
        if verbose_on {
            println!("[info] Removing directory: {}", dir.display());
        }
    }

    fs::create_dir_all(dir).map_err(|e| anyhow::anyhow!("Failed to recreate directory: {}", e))?;

    // --- VERBOSE ---
    if verbose_on {
        println!("[info] Recreating directory: {}", dir.display());
    }

    Ok(())
}

// TODO: Remove this function and use above `clean_this_dir()` so you can clean both cache and build directories in one call, or separately.
pub fn clean_cache_dir(
    base_dir: PathBuf,
    build_dir: PathBuf,
    verbose_on: bool,
) -> anyhow::Result<()> {
    let base_directory = base_dir.canonicalize()?;
    verify_workspace(&base_directory)?;
    let cache_dir = base_directory.join(".csalt");

    clean_this_dir(&cache_dir, verbose_on)?;
    clean_this_dir(&build_dir, verbose_on)?;

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
    verbose: bool,
) -> anyhow::Result<()> {
    clean_cache_dir(base_dir.to_path_buf(), build_dir.to_path_buf(), verbose)?;
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
///
/// ### Examples
///
/// ```
/// use csalt::fs_utils::init_default_salt_toml;
///
/// let temp_dir = tempfile::tempdir().unwrap();
/// init_default_salt_toml("my_project", &temp_dir.path()).unwrap();
///
/// assert!(temp_dir.path().join("Salt.toml").exists());
/// ```
pub fn init_default_salt_toml(project_name: &str, dir: &Path) -> anyhow::Result<()> {
    let toml_content = SaltToml {
        package: crate::config::PackageSection {
            name: project_name.to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["".to_string()],
            description: "".to_string(),
        },
        build: crate::config::BuildSection {
            build_sys: None,
            build_sys_ver: None,
            build_dir: Some(PathBuf::from("build/")),
            edition: crate::config::CEditions::C11,
            compiler: Some(crate::config::CompilerBackend::Clang),
        },
        unit: vec![crate::config::UnitVector {
            name: project_name.to_string(),
            kind: crate::config::UnitKinds::Bin,
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

pub fn init_default_gitignore(dir: &Path, stealth: bool) -> anyhow::Result<()> {
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

    Ok(())
}

pub fn init_empty_salt_lock(dir: &Path) -> anyhow::Result<()> {
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

    Ok(())
}

pub fn init_git_version_control(init_git: bool, dir: &Path) -> anyhow::Result<()> {
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

/// Creates a new project in the current directory or the specified directory.
///
/// # Arguments
///
/// * `name` - The name of the project.
/// * `dir` - The directory to create the project in.
/// * `template` - Whether to follow a specific workspace layout for a specific goal
/// * `stealth` - Whether to add configuration files to `.gitignore`.
/// * `init_git` - Whether to initialize Git.
///
/// ### Examples
/// ```
/// ```
pub fn new_project(
    name: &str,
    dir: Option<&str>,
    template: &str,
    stealth: bool,
    init_git: bool,
) -> anyhow::Result<()> {
    let path = Path::new(&dir.unwrap_or(".")).join(name);
    fs::create_dir_all(&path)?;

    let template_files = lookup_template(template)?;

    for file in template_files {
        let file_path = path.join(file.virtual_path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(file_path, file.content)?;
    }

    init_empty_salt_lock(path.as_path())?;
    init_default_gitignore(path.as_path(), stealth)?;
    init_git_version_control(init_git, path.as_path())?;
    overwrite_salt_toml_with_project_name(path.join("Salt.toml").as_path(), name)?;

    Ok(())
}

/// Initializes a project in the specified directory.
///
/// # Arguments
///
/// * `dir` - The directory to initialize the project in.
/// * `stealth` - Whether to add configuration files to `.gitignore`.
/// * `init_git` - Whether to initialize Git.
pub fn init_project(dir: &Path, stealth: bool, init_git: bool) -> anyhow::Result<()> {
    fs::create_dir_all(dir)?;

    let project_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    init_default_salt_toml(project_name, dir)?;
    init_empty_salt_lock(dir)?;
    init_default_gitignore(dir, stealth)?;
    init_git_version_control(init_git, dir)?;

    Ok(())
}
