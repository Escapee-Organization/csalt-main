<div align="center">
  <picture>
    <source srcset="https://raw.githubusercontent.com/Escapee-Organization/csalt-artwork/main/logos/csalt-original-official-logo.png" type="image/png">
    <img src="https://raw.githubusercontent.com/Escapee-Organization/csalt-artwork/main/logos/csalt-original-official-logo.png" alt="C-Salt CLI Logo" width="300" height="300">
  </picture>
  <h1>C-Salt</h1>
</div>


<p align="center">
  <strong>"Drop your C files and it just works."</strong><br><br> A Cargo-inspired, declarative build driver and workspace orchestrator designed to eliminate manual compilation headaches while not compromising absolute control over your repo.
</p>

<p align="center">
  <a href="https://crates.io/crates/csalt">
    <img src="https://img.shields.io/crates/v/csalt?style=for-the-badge" alt="Crates.io Version">
    <img src="https://img.shields.io/crates/l/csalt?style=for-the-badge" alt="Crates.io License">
    <img src="https://img.shields.io/crates/size/csalt?style=for-the-badge" alt="Crates.io Size">
    <img src="https://img.shields.io/crates/d/csalt?style=for-the-badge" alt="Crates.io Total Downloads">
  </a>
</p>

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Features](#features)
- [FAQ](#faq)
- [AI Usage Disclosure](#ai-usage-disclosure)
- [Roadmap](#roadmap)
- [Contributing and Licensing](#contributing-and-licensing)

## Why C-Salt?
C isn't a bad language. However, the tooling is tedious and old. C-Salt aims to solve this by providing a modern, Cargo-inspired build orchestrator. It operates on a compilation "unit", which allows for easy management of C projects at the start. However, it also doesn't force `Salt.toml` upon you if you don't want to use it, as C-Salt is a lightweight wrapper, not a replacement.

**NOTE**: we use `.csalt/` as a cache for all of its tasks so far to keep your repo clean, but in the future, you will be able to adjust that, and this architecture will prove its usefulness.

```toml
# Salt.toml
[package]
name = "example"
version = "0.1.0"
authors = [""]
description = ""

[build]
build_sys = "cmake"
build_sys_ver = "3.15"
build_dir = "build/"
edition = "c11"
compiler = "clang"

[[unit]]
name = "example"
kind = "bin"
main = "src/main.c"
src = ["src/"]
```

**NOTE**: C-Salt is a Minimum Viable Product (MVP) in this current stage. It is not ready for production use, will change heavily, has unfinished features, and is not battle-tested against every edge case, especially for cross-platform uses.

## Quick Start
There are a few ways to get started with C-Salt:

### 1. Releases
Download the latest pre-release from the [releases page](https://github.com/Escapee-Organization/csalt-main/releases).

### 2. Crates.io
If you have Cargo installed, you can install C-Salt directly from [crates.io](https://crates.io/crates/csalt):

```bash
cargo install csalt
```

### 3. Source
If you prefer to build from source, you can clone the repository and install it locally:

```bash
# Clone the repository from source (use any Git client, e.g. Git Bash, GitKraken, GitHub Desktop)
git clone https://github.com/Escapee-Organization/csalt-main.git
cd csalt-main

# Install the CLI via Cargo
cargo install --path .

# Verify installation
csalt --help
```

## Features

- **`csalt new <name>`**: Provisions a standardized C workspace complete with `src/`, `include/`, `.gitignore` instantly.
  - `--git`: Auto-initializes a Git repository.
  - `--full`: Populates placeholder directories for `vendor/` and `tests/` and `README.md`.
  - `--stealth`: **Stealth Mode.** Removes configuration files from tracking so you can secretly manage massive legacy codebases with C-Salt without disrupting your team. *Note: it is always wise to copy an existing C project into a new directory before using C-Salt.*
- **`csalt compile`**: Executes C-Salt as a build system, calling the compiler and linker inside the cache (`.csalt/`), and enforcing a strict, safe compilation order. Supports `bin` (executable), `lib` (static archives), and `dyn` (shared dynamic libraries) out of the box.
- **`csalt build`**: Automatically acts as a communicator to a build system, such as CMake 3.15. It has two modes:
  - **Fresh Mode**: Translates your linear `Salt.toml` structure seamlessly into native, readable `CMakeLists.txt` scripts for CMake 3.15, compiling the final workspace cleanly.
  - **Managed Mode**: Detects if you already have a custom, manual `CMakeLists.txt` in your root, safely steps out of the way, and passes command execution downstream to trust your existing system rules.
- **Raw Passthrough Escape Hatch**: Pass trailing variable arguments directly to your underlying backend (`csalt (compile/build) -- [args]`) to run raw commands. In other words, it's a macro for:
  - ```bash
    csalt emit
    cd .csalt/
    <command> [args]
    cd ..
    ```

## FAQ

1. **What are the kinds of `[[unit]]` I can use?**
  - There are 5 basic kinds of `[[unit]]` you can use:
  - `bin` (binary, e.g. `main` or `main.exe`)
  - `lib` (static library, e.g. `libmath.a` or `math.lib`)
  - `dyn` (dynamic library, e.g. `libmath.dll` or `libmath.so`)
  - `extlib` (pre-compiled static library, e.g. `libmath.a` or `math.lib`)
  - `extdyn` (pre-compiled dynamic library, e.g. `libmath.dll` or `libmath.so`)
  - **NOTE**: `extdyn` and `dyn` have caused the original creator many issues, especially late at night for cross-platform uses, so they may not be complete

## AI Usage Disclosure
All architectural decisions were made by the original creator (BurningHot687). A large portion of the current codebase was generated using the Zed Auto-complete functionality and Gemini Flash.

* **Current Status:** Refactoring is underway to clean up AI-generated sections, optimize maintainability, and ensure long-term stability.
* **Why AI?:** This was the original creator's first Rust project, which started on 2026/6/7, and finished the summer MVP on 2026/7/10 (albeit not testing). The creator had only written simple "Hello World" programs in C and Rust before this project, so they had to actively learn Rust and C at the same time while working on this project. They used AI to speed up development, get working code quickly, and quickly learn how to use Rust, Git, and more. They also were able to learn enough to spot the mistakes in AI-generated code, which was common due to using a weak model.

## Roadmap
See the [ROADMAP](ROADMAP.md) file for details.

## Contributing & Licensing
If you would like to contribute, please see the [CONTRIBUTING](CONTRIBUTING.md) file for details.

This project is licensed under the MPL 2.0 License. See the [LICENSE](LICENSE) file for details.
