use std::fs;
use std::path::{Path, PathBuf};

use csalt::build_managed_project;
use csalt::cli::BuildArgs;
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

    let result = build_managed_project(&BuildArgs {
        backend: None,
        path: Some(PathBuf::from(test_root)),
        backend_flags: Vec::new(),
    });

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
    );

    let _ = fs::remove_dir_all(test_root);
}
