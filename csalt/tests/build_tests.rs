use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

use csalt::{build_managed_project, build_manual_project};

// TODO: If you're going to use the `examples/` as the source for many tests, why not just loop over 'em? Unable to see exactly what went wrong without it clogging the whole thing perhaps? Figure it out, copy-pasting sucks

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
fn test_default_project_self_build_system() -> anyhow::Result<()> {
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

    build_manual_project(
        &None,
        &Some(PathBuf::from(test_root)),
        &None,
        false,
        &None,
        true,
        &Vec::new(),
    )?;

    assert!(cache_dir.exists(), "cache directory was not created!");

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

#[test]
fn test_zlib_ver_self_build_system() -> anyhow::Result<()> {
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

    build_manual_project(
        &None,
        &Some(PathBuf::from(test_root)),
        &None,
        false,
        &None,
        true,
        &Vec::new(),
    )?;

    assert!(cache_dir.exists(), "cache directory was not created!");

    Ok(())
}
