# `bin` Example

This example demonstrates:
1. C-Salt's capability at defining and building a C project via `Salt.toml`
2. The difference between `csalt compile` and `csalt build`
3. The default new C-Salt project, run with `csalt new <name>` or `csalt new <name> --template=bin`

## Prerequisites

To run this example, first make sure you have C-Salt pre-requisites. To run `csalt build`, you need `cmake` installed as well.

## Start

1. Set your current working directory to `default-new-project` and run `csalt compile`. Feel free to poke around the workspace and see what happens or what `csalt compile` did.
    - *Hint*: Check the `.csalt/` cache directory.
2. Run `csalt clean` to clean up the workspace.
3. Run `csalt build` to build the workspace using `cmake`. Again, feel free to poke around the workspace after this. Note the difference between `csalt compile` and `csalt build`, between stdout and the final output, especially in the cache directory.
