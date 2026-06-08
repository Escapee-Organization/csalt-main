// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org.
// Copyright (c) 2026 Escapee Organization

use clap::{ArgAction, Parser};

/// csalt - A CLI tool and language that just works with C
#[derive(Parser, Debug)]
#[command(author = "BurningHot687", version, about, long_about = None, name = "csalt")]
struct Args {
    /// The main input source file or target directory
    #[arg(required = true)]
    input: String,

    /// Explicitly set the output binary file destination
    #[arg(short = 'o', long = "output")]
    output: Option<String>,

    /// Choose the host compiler driver backend [possible values: clang, gcc, zig]
    #[arg(short = 'b', long = "backend")]
    backend: String,

    /// Trailing parameters forwarded completely intact to the backend compiler layer
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, action = ArgAction::Append)]
    backend_flags: Vec<String>,
}

fn main() {
    let args = Args::parse();
    println!("Hello, world!");
}
