# C-Salt

## Table of Contents

- [Overview](#overview)
- [Project Scope](#project-scope)
- [Licensing](#licensing)

## Overview

C-Salt is a build helper and simple transpiler for both managing existing C projects and creating new ones. Built with Rust, this project's philosophy is **"Drop your files and it just works"**.

### Usage of AI
The Zed Auto-complete functionality alongside the Gemini Flash model commonly seen in "Google Search" was used to generate a large portion of this project's codebase. This is a known issue and **must be addressed to ensure the project remains maintainable and up-to-date**.

However, the main idea of the project was created via a human's idea bouncing off the AI model. All decisions were made by the human author, with the AI model providing assistance and guidance.

## Project Scope (Summer MVP)

* ### Transpilation of simple keywords
  * Allows for more expressive and simpler syntax for C in certain cases. For this summer, a few basic keywords will be added to a C tree-sitter (e.g. `vari x: int;` -> `int x;`).
* ### Handling of existing header and CMake files and generating new files
  * Distinguish between pure C-Salt projects and any taint coming from another build system. In the former case, build up an interal representation to then generate any build system the user desires, although intially CMake or shell scripts only. In the case of the user using their own CMake or other build systems, find the master script and insert a cleanly denoted block of imports. Also, `.c` and `.csal` files should both be able to auto-generate.
* ### Provide a simple CLI for managing C-Salt projects
  * Allows the user to easily create, build, and manage C-Salt projects from the command line. Utilizes a blend between clang and cargo-like CLI.

## Licensing
This project is licensed under the MPL 2.0 License. See the [LICENSE](LICENSE) file for details.
