# Roadmap

The following is the roadmap for C-Salt, divided into phases and other sections. Note this is currently a work in progress and is subject to change, both the project **and** this roadmap itself.

## Summer Goals (first deadline July 19th)
- [x] Generate new directories in C repository structure
- [x] Refactor codebase to be decently maintainable
- [ ] Add auto-generation of header files via tree-sitter
- [ ] Generate build files upon `csalt build`
- [ ] Act as build system for `csalt compile` with incremental compilation and ability to support lib, dyn, and bin.
- [ ] Create Foreign Function Interface binding feature as a separate crate which C-Salt uses to aide developers.

## Phase 1 of C-Salt (Second half of 2026)

* ### Transpilation of simple keywords
  * Allows for more expressive and simpler syntax for C in certain cases. For this summer, a few basic keywords will be added to a C tree-sitter (e.g. `vari x: int;` -> `int x;`).
* ### Handling of existing header and CMake files and generating new files
  * Distinguish between fresh C-Salt projects and managed C-Salt projects. In the former case, build up an interal representation to then generate any build system the user desires, although intially CMake or shell scripts only. In the case of the user using their own CMake or other build systems or manual header files, find the master script and provide a cleanly denoted block of imports. `.c` and `.csal` files will both be able to auto-generate these.
* ### Provide a simple CLI for managing C-Salt projects
  * Allows the user to easily create, build, and manage C-Salt projects from the command line. Utilizes a blend between clang and cargo-like CLI.
* ### Cache results for incremental builds
  * Cache results to avoid rebuilding unchanged files, improving build performance in fresh projects.

The following will be considered for the time between Phase 1 and Phase 2 of C-Salt:
* ### Auto-generation of Foreign Function Interfaces (FFIs)
  * Automatically generate FFIs for other languages needing to interface with C, reducing manual effort and potential errors. These should go into the `build/bindings/` directory.
  * Languages considered are Rust, Zig, Python, and Go for now. Please leave an issue if you need support for a different language.
* ### New syntactic additions
  * C may be able to get syntax which will enable it to perform modern safety alongside becoming more readable and clean thanks to C-Salt's nature as a transpiler. However, this is not a priority for the time being, and will not be for a long time. Unfortunately, the syntactic roadmap does not have a clear file  detailing each addition considered.
  * Syntax considered for the future are: `__defer`, `__match`, etc.

## Phase 2 of C-Salt (2027?)

* ### Formal verification
  * Perform formal verification on C-Salt code to ensure mathematical safety. To keep it simple for now, we are considering a "binding" approach where a block of requirements is "linked" to a specific named function or block of code.
  * &nbsp;
    ```c
    __bind_to_fnct my_function {
        __require(...);
        __ensure(...);
    }

    int my_function() {
        
    }
    ```
