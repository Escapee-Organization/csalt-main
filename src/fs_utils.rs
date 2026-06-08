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
            "Workspace directory not found",
        )));
    }

    // Check if the directory's structure is valid
    if !path.join("Salt.toml").is_file() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "'Salt.toml' not found",
        )));
    }
    if !path.join("Salt.lock").is_file() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "'Salt.lock' not found",
        )));
    }
    if !path.join(".csalt").is_dir() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "'.csalt/' hidden directory not found",
        )));
    }

    // Check if there is no manual Makefile or CMakeLists.txt, etc
    if path.join("Makefile").is_file() || path.join("CMakeLists.txt").is_file() {
        // Edit the Salt.lock file
        println!("[warning]\n Manual Makefile or CMakeLists.txt found, updating 'Salt.lock'");
    }

    Ok(())
}
