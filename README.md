# C-Salt

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Features](#features)
- [AI Usage Disclosure](#ai-usage-disclosure)
- [Roadmap](#roadmap)
- [Contributing and Licensing](#contributing-and-licensing)

## Overview

C-Salt is a Cargo-inspired build orchestrator for both managing existing C projects and creating new ones. Built with Rust, this project's philosophy is **"Drop your files and it just works"**.

## Quick Start

### Prerequisites

- **Cargo**: Required for building C-Salt itself.
- **C Compiler**: Required for compiling C code.
- **Git**: Required for cloning the repository.

### Installation
```bash
# Clone the repository from source (use any Git client, e.g. Git Bash, GitKraken, GitHub Desktop)
git clone https://github.com/Escapee-Organization/csalt-main.git
cd csalt-main

# Install the CLI via Cargo
cargo install --path .

# Run the CLI
csalt --help
```

## Features

- **C Repository Generation**: Just like Cargo, C-Salt can generate new C repositories with a simple command, and provides standards and flexibility for your environment.
- **Simple Build Configuration**: Uses a simple, human-readable build configuration file (`Salt.toml`) to define build targets and dependencies. Allows for an easy start, with a simple way to opt-out of using it for more complex build systems.
- **Simple Build System**: Unless you decide to pass trailing var-args to `compile` or `build` commands, C-Salt will automatically handle things for you. `csalt compile` uses C-Salt as a build system, while `csalt build` generates the build files for the build system you are using.
- **Out of the Way**: C-Salt respects if you already have build files and will **not** use `Salt.toml` to generate new build files, instead trusting your existing ones.

## AI Usage Disclosure
The main concepts were developed by the author. A large portion of the current codebase was generated using the Zed Auto-complete functionality and Gemini Flash. 

* **Current Status:** Refactoring is underway to clean up AI-generated sections, optimize maintainability, and ensure long-term stability.

## Roadmap
See the [ROADMAP](ROADMAP.md) file for details.

## Contributing & Licensing

If you would like to contribute, please see the [CONTRIBUTING](CONTRIBUTING.md) file for details.

This project is licensed under the MPL 2.0 License. See the [LICENSE](LICENSE) file for details.
