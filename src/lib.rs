// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization
pub mod engine;
pub mod fs_utils;

pub fn run(workspace: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs_utils::verify_workspace(workspace)?;

    // TODO: run the engine
    Ok(())
}
