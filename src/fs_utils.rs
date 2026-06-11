#[warn(unused_imports)] // TODO: Make sure nothing is unused
use std::error::Error;
use std::fs;
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

pub fn new_project(name: &str, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Make new directory, move into it, and create all elements
    let path = Path::new(dir).join(name);
    fs::create_dir_all(&path)?;

    init_project(&path.to_str().unwrap())?;

    println!("[info]\nNew project '{}' created successfully", name);
    Ok(())
}

pub fn init_project(dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir)?;
    let toml_content = format!(
        r#"
[project]
name = "{}"
version = "0.1.0"
edition = "11"

[build]
target = "debug"
"#,
        dir
    );
    fs::write(dir.to_owned() + "/Salt.toml", toml_content)?;
    fs::write(dir.to_owned() + "/Salt.lock", "")?;

    let path = Path::new(dir);
    fs::create_dir(path.join("src"))?;
    fs::create_dir(path.join(".csalt"))?;
    // fs::write(path.join("src").to_owned(), "main.c")?;

    println!("[info]\nProject directory initialized successfully");
    Ok(())
}
