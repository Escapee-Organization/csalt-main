# C-Salt Project Roadmap

This document outlines the development trajectory for C-Salt. As an evolving build driver, this roadmap is subject to adjustments based on community feedback, toolchain updates, and architectural studying from the original creator.

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

## 🍁 Phase 1: Hardening & Completion (H2 2026)

With the baseline compiler driver operational, Phase 1 shifts toward refactoring codebase constraints, introducing local caching, and beginning automated file generation.

### 1. Codebase Hardening & Refactoring
- [ ] **Idiomatic Error Propagation**: Replace early-prototype `.unwrap()` statements with robust, idiomatic Rust `Result` types and unified system errors.
- [ ] **Cross-Platform Native Extension Support**: Verify and complete the deferred Windows MSVC/`cl.exe` (`.obj` / `.lib`) command generation paths.
- [ ] **Heap Allocations**: Avoid idiomatic Rust in cases where hidden heap allocations are possible. This mostly applies to closures.

### 2. Workspace Optimization
- [ ] **Incremental Compilation Cache**: Track SHA256 hashes inside `Salt.lock` to skip rebuilding, copying, generating, or transpiling unchanged source units, maximizing performance.
- [ ] **Automated Header Sync**: Build out the automated generation of header files during the `csalt emit` phase.
- [ ] **Test Symlinks**: Figure out whether symlinks should be implemented, and if so, how. Symlinks are notorious with `dyn`, however they should be considered for every file a `dyn` doesn't touch for performance.

### 3. Syntax Generation Framework
- [ ] **Tree-Sitter C Parser Integration**: Introduce `tree-sitter` and `tree-sitter-c` to safely inspect translation units without crashing on complex macros.

---

## ❄️ Phase 1.5: Interoperability Extensions (Late 2026 / Early 2027)

These features represent the transition from a pure build manager to an expressive compilation orchestrator.

### 1. Automated Foreign Function Interfaces (`csalt bind`)
- [ ] Extract structured C type metadata using tree-sitter to automatically generate binding interfaces for modern target languages.
- [ ] Emit compiled outputs directly into a unified `build/bindings/` directory.
- [ ] **Target Languages Considered**: Rust, Python, Go.
- [ ] Create as separate crate to avoid bloating `csalt-main` and allow much wider usage.

### 2. Basic Syntax Transpilation
- [ ] Introduce transparent keyword lowering for cleaner variable allocation syntax (e.g., lowering `vari x: int;` down to standard `int x;`).

---

## 🚀 Phase 2: Advanced Language Horizons (2027+)

**IF** C-Salt manages to be **robust** and **dependable**, then sneaking in high-level language expressiveness and integrated safety contracts is easy.

### 1. Modern Semantic Additions
- [ ] Investigate compile-time macro code generation for modern language ergonomics like safe scope cleanup (`__defer`) and structural switching (`__match`).

### 2. Isolated Formal Verification Contracts
- [ ] Explore a declarative binding approach to attach formal verification (and more) directly to named C targets. This would transpile to FragmaC comments or be passed directly into a solver.
- ```c
  __bind_to_fnct my_function {
      __require(...);
      __ensure(...);
  }

  int my_function() {
      // Verified code block
  }
