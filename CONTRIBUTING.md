# Contributing to C-Salt

Thank you for your interest in helping build C-Salt! We welcome all contributions to make this project better now that we have transitioned into a public repository. The current aim is to provide a modern Rust-like wrapper for C compilers and bring the developer's focus back to logic instead of build systems or manual linking.

## Table of Contents
- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Standards](#development-standards)
- [Where to Start](#where-to-start)

## Code of Conduct
By participating in this project, you agree to abide by our behavioral expectations. Please be respectful and welcoming to all community members.

## How Can I Contribute?
On all issues, if you are willing to contribute, drop a comment letting us know so we can assign you to the issue and move onto another feature or bug of higher priority.

### Reporting Bugs
- Check the Issues tab to ensure the bug hasn't already been reported.
- Open a new issue using our bug template.
- Include clear steps to reproduce the bug and your environment details.

### Requesting Features
- Open an issue describing the feature you want.
- Explain why this feature would be useful to most users.

### Moving Codebase Issues
- Find a comment within the codebase which begins with either `NOTE:`, `TODO:`, or `FIXME:`.
- Check the Issues tab to see if the issue has already been moved.
- If the issue has not been moved, create a new issue using our bug, feature request, or todo template.

### Submitting Code (Pull Requests)
1. Fork the repository and clone it locally.
2. Ensure you have the latest Rust toolchain (`cargo --version`).
3. Build the repo locally to ensure everything works:
   ```bash
   cargo build
   ```
4. Create a new branch for your changes (e.g. `feat/new-feature` or `fix/some-bug`).
5. Make your changes and write tests if necessary.
6. Push to your fork and submit a Pull Request (PR) against our `main` branch.

## Development Standards
- **Git Commits**: Write commit messages in the imperative mood and use [**conventional commits**](https://www.conventionalcommits.org/en/v1.0.0/#summary) (e.g., "fix: resolve bug in `copy_project_files()`" instead of "Fixed bug").
- **Code Style**: We follow standard formatting tools. Run `cargo fmt` before committing.
- **Code Quality**: Write code which will be maintainable in the future. It's good practice to run `cargo clippy` and listen to those suggestions.
- **AI Usage, etc.**: Be transparent about the way you're contributing to C-Salt. It will be used to correctly assess potential issues or misunderstandings within your contribution, and is something the Escapee Foundation will adhere to internally as well.

## Where to Start
If you are unsure of where to start contributing, here are some things to check:
- Look for any issues which aren't resolved or being handled by someone.
- Find a specific edge case within C-Salt (e.g. specific architecture differences or unsupported compiler commands or flags)
- Find an issue with the label "good first issue"
