# C-Salt

## Table of Contents

- [Overview](#overview)
- [Project Scope](#project-scope)
- [Licensing](#licensing)

## Overview

A build helper and simple transpiler for both managing existing C projects and creating new ones. Built with Rust, this project's philosophy is **"Drop your files and it just works"**.

## Project Scope (Summer MVP)

* ### Transpilation of simple keywords
  * Allows for more expressive and simpler syntax for C in certain cases. For this summer, a few basic keywords will be added to a C tree-sitter (e.g. `vari x: int;` -> `int x;`)
* ### Handling of existing header and CMake files and generating new files
  * Distinguish between pure C-Salt projects and any taint coming from another build system. In the former case, build up an interal representation to then generate any build system the user desires, although intially CMake or shell scripts only. In the case of the user using their own CMake or other build systems, find the master script and insert a cleanly denoted block of imports. Also, `.c` and `.csal` files should both be able to auto-generate.

## Licensing
This project is licensed under the MPL 2.0 License. See the [LICENSE](LICENSE) file for details.
