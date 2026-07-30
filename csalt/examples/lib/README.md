# `lib` Example

This example demonstrates:
1. The C-Salt project template `lib`, which can be created via `csalt new <name> --template lib`
2. C-Salt's ability to include paths via `include`

## Prerequisites

To run this example, first make sure you have C-Salt pre-requisites. To run `csalt build`, you need `cmake` installed as well.

## Start

1. Set your current working directory to `lib` and run `csalt compile` or `csalt build` (see [here to see how to add a build system like `cmake`](../bin/README.md)). Feel free to poke around the workspace and see what happened.
2. Try changing `lib` to `dyn`, which changes the compilation from a static library (such as `.a`) to a dynamic library (such as `.so`).
