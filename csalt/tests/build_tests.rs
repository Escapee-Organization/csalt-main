use std::fs;
use std::path::{Path, PathBuf};

use csalt::build_managed_project;
use csalt::fs_utils::init_project;

#[test]
fn cmake_test() {
    let test_root = Path::new("salt-test/cmake");
    let cache_dir = test_root.join(".csalt");
    let build_dir = test_root.join("build");

    let _ = fs::remove_dir_all(test_root);
    fs::create_dir_all(&cache_dir).unwrap();
    fs::create_dir_all(&build_dir).unwrap();

    init_project(test_root, false, false, false).unwrap();

    // FIXME: Make it easier to change the TOML content without a raw string or manual editing of *everything*
    let toml_content = r#"
        [package]
        name = "cmake"
        version = "1.0.0"
        authors = [""]
        description = ""

        [build]
        build_sys = "cmake"
        build_sys_ver = "3.15"
        edition = "c11"

        [[unit]]
        kind = "bin"
        name = "test"
        src = ["src/"]
        "#;

    std::fs::write(test_root.join("Salt.toml"), toml_content).unwrap();

    let result = build_managed_project(&None, &Some(PathBuf::from(test_root)), &None, &Vec::new());

    assert!(
        result.is_ok(),
        "build_managed_project failed with error: {:?}",
        result.err()
    );

    let expected_cmake_path = cache_dir.join("CMakeLists.txt");
    assert!(
        expected_cmake_path.exists(),
        "CMakeLists.txt was not generated inside the cache directory!"
    );

    let _ = fs::remove_dir_all(test_root);
}
