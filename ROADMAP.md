# C-Salt Project Roadmap

This document outlines the development trajectory for C-Salt. As an evolving build driver, this roadmap is subject to adjustments based on community feedback, toolchain updates, and architectural studying from the original creator. So far, the roadmap is facing massive reductions in scope.

---

## Summer 2026: The Core MVP (Accomplished)

The initial summer milestone focused on establishing a working systems-level build driver from scratch, managing isolated workspace environments, and providing Cargo-like configuration.

- [x] **Configuration**: Implemented full `Salt.toml` and `Salt.lock` package and multi-unit parsing boundaries via `serde`, `toml`, and `serde_json`.
- [x] **Separation of Concerns (`csalt emit`)**: Established an isolated cache workspace (`.csalt/`) to keep source directories clean.
- [x] **Easy Workspace Creation (`csalt new`)**: Built automated template creation with `--git`, `--full`, and `--stealth` functionality.
- [x] **Entire New Simple Build System (`csalt compile`)**: Fully supported linear compilation units for `bin` (executables), `lib` (static archives), and `dyn` (shared dynamic objects).
- [x] **Dual-Mode Build System Communication (`csalt build`)**: 
  - **Fresh Mode**: Translates linear unit dependencies into functional `CMakeLists.txt` scripts for CMake 3.15.
  - **Managed Mode**: Detects pre-existing manual `CMakeLists.txt` structures and safely defers execution downstream.
- [x] "Actually finished this lol, I am not meant to look at a screen for almost 11 hours, am I? Woah my head's spinning" - BurningHot687, 2026/7/10 23:31

---

## Phase 1: Hardening & Completion

With the baseline compiler driver operational, Phase 1 shifts toward refactoring codebase constraints, introducing local caching, and beginning automated file generation.

### 1. Codebase Hardening & Refactoring
- [x] **Idiomatic Error Propagation**: Replace early-prototype `.unwrap()` statements with robust, idiomatic Rust `Result` types and unified system errors.
- [ ] **Cross-Platform Native Extension Support**: Verify and complete the deferred Windows MSVC/`cl.exe` (`.obj` / `.lib`) command generation paths.
- [ ] **Heap Allocations**: Reduce the use of heap allocations in favor of stack allocations.
- [x] **Semantic Versioning for Build Systems**: Implement semantic versioning to allow for choosing the exact build system version you need to use. This also allows C-Salt to properly generate the right build file for your specific `semver` choice.
  - Does not support ranges yet.

### 2. Workspace Optimization
- [ ] **Incremental Compilation Cache**: Track SHA256 hashes inside `.csalt/` to skip rebuilding, copying, generating, or transpiling unchanged source units, maximizing performance. Currently using `depfile` for the first approach.
- [ ] **Test Symlinks**: Figure out whether symlinks should be implemented, and if so, how. Symlinks are notorious with `dyn`, however they should be considered for every file a `dyn` doesn't touch for performance.
- [x] **Already Compiled Assets**: Add support for already-compiled assets within `Salt.toml` to allow linking of libraries and object files, etc.
  - Does not support object files yet.
- [x] **Compiler-Searching**: Allow for `Salt.toml` to use the compiler as an `Option<>` so C-Salt can imitate the compiler searching behavior of tools such as CMake.

### 3. Syntax Generation Framework
- [ ] **Tree-Sitter C Parser Integration**: Introduce `tree-sitter` and `tree-sitter-c` to safely inspect translation units without crashing on complex macros.

---

## Phase 2: Potential Improvements

**IF** C-Salt manages to be **robust** and **dependable**, then sneaking in high-level language expressiveness and integrated safety contracts is easier.

### 1. Isolated Formal Verification Contracts
- [ ] Explore a declarative binding approach to attach formal verification (and more) directly to named C targets. This would be written as comments and then passed to FragmaC or another solver like Z3.

### 2. Automated Foreign Function Interfaces (`csalt bind`)
- [ ] Extract structured C type metadata using tree-sitter to automatically generate binding interfaces for modern target languages.
- [ ] Emit compiled outputs directly into a unified `build/bindings/` directory.
- [ ] **Target Languages Considered**: Rust, Python, Go.
- [ ] Create as separate crate to avoid bloating `csalt` and allow much wider usage.

### 3.Automated Header File Creation
- [ ] **Automated Header Sync**: Figure out `tree-sitter-c` in order to call `csalt reflect` on a C file to generate a header file. Create as a separate crate to avoid bloating the main `csalt` binary.

### 4. Ease of Documentation
- [ ] **Easy Configuration Generation**: Generate config files for documentation tools to read in or respect the user's already written one.
- [ ] **Intermediate Representation**: Represent all documentation comments with an IR, oblivious to style differences between tools.
- [ ] **Repetition of Mirror Directory**: Reuse the mirror directory concept to emit whichever doc comment style the tool the user is using expects, independent of which style their codebase uses. Then call the tool upon that mirror directory.
- [ ] **Linting and Refactoring Ease**: Make switching comment styles or finding incorrect comments much easier.
- [ ] **Skeleton Generation**: Add a way to point to a function and get a terminal output which outputs a skeleton doc comment to speed up documentation process.
