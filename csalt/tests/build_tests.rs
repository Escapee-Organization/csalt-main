// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use csalt::{build_managed_project, build_manual_project};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

// TODO: Move to use nice function in fs_utils, `walk_and_filter_dirs()` later
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    fs::create_dir_all(&dst)?;

    let mut stack = vec![(src.as_ref().to_path_buf(), dst.as_ref().to_path_buf())];
    while let Some((src, dst)) = stack.pop() {
        fs::create_dir_all(&dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let file_name = entry.file_name();
            if ty.is_dir() {
                stack.push((entry.path(), dst.join(file_name)));
            } else {
                fs::copy(entry.path(), dst.join(file_name))?;
            }
        }
    }
    Ok(())
}

// FIXME: The following test below this comment appears to fail on my windows laptop. Interestingly, it only fails after successful compilation, seeming to call `clang` at the last second after cleaning and failing the whole test since it doesn't exist here.

#[test]
fn test_all_example_projects_csalt_compile() -> anyhow::Result<()> {
    let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");

    for entry in fs::read_dir(examples_dir)? {
        let entry = entry?;
        let source_path = entry.path();

        if source_path.is_file() {
            continue;
        }

        // NOTE: Find an easy way to modify any field of `SaltToml` without so much boilerplate
        if source_path.as_os_str() == "zlib-ver" {
            continue;
        }

        let temp_dir = tempdir()?;
        let test_root = temp_dir.path();

        copy_dir_all(entry.path(), test_root)?;

        build_manual_project(
            &None,
            &Some(PathBuf::from(test_root)),
            &None,
            false,
            &None,
            true,
            &Vec::new(),
        )?;

        assert!(
            test_root.join(".csalt").exists(),
            "cache directory was not created!"
        );
        assert!(
            test_root.join("build").exists(),
            "build directory was not created!"
        );
    }

    Ok(())
}

#[test]
fn test_default_project_cmake_generation() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let test_root = temp_dir.path();
    let cache_dir = test_root.join(".csalt");

    let example_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("bin");
    if example_src.exists() {
        copy_dir_all(&example_src, test_root)?;
    } else {
        anyhow::bail!("example source directory does not exist");
    }

    let toml_content = r#"[package]
name = "bin"
version = "0.1.0"
authors = [""]
description = ""

[build]
build_sys = "cmake"
build_sys_ver = "3.15"
compiler = "clang"
edition = "c11"

[[unit]]
name = "bin"
kind = "bin"
src = ["src/"]"#;

    fs::write(test_root.join("Salt.toml"), toml_content)?;

    build_managed_project(
        &None,
        &Some(PathBuf::from(test_root)),
        &None,
        &Vec::new(),
        &None,
        true,
    )?;

    let expected_cmake_path = cache_dir.join("CMakeLists.txt");
    assert!(
        expected_cmake_path.exists(),
        "CMakeLists.txt was not generated inside the cache directory!"
    );

    Ok(())
}

#[test]
fn test_zlib_ver_example_project_cmake_generation() -> anyhow::Result<()> {
    let temp_dir = tempdir()?;
    let test_root = temp_dir.path();
    let cache_dir = test_root.join(".csalt");

    let example_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("zlib-ver");
    if example_src.exists() {
        copy_dir_all(&example_src, test_root)?;
    } else {
        anyhow::bail!("example source directory does not exist");
    }

    build_managed_project(
        &None,
        &Some(PathBuf::from(test_root)),
        &None,
        &Vec::new(),
        &None,
        true,
    )?;

    let expected_cmake_path = cache_dir.join("CMakeLists.txt");
    assert!(
        expected_cmake_path.exists(),
        "CMakeLists.txt was not generated inside the cache directory!"
    );

    Ok(())
}
