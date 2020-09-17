[**This project has nothing to do with QAnon and I oppose QAnon completely.**](https://github.com/mfeq/mfeq/blob/master/doc/QAnon.md)

[![Build Status](https://img.shields.io/github/workflow/status/mfeq/Qglif/Rust?label=Rust&logo=Rust)](https://github.com/mfeq/Qglif/actions?workflow=Rust)

# Qglif

Very early glyph editor for the Modular Font Editor Q project.

![screenshot](https://raw.githubusercontent.com/mfeq/Qglif/master/doc/screenshot.png)

## Overview

Qglif mixes three technologies: Skia, a very powerful path rasterizer and manipulation library; Dear ImGui, an immediate mode user interface toolkit; and Rust, a modern high-performance systems language.

I wrote it after, hopefully, learning from the mistakes made by George Williams in FontForge, after being a user of FontForge for six years and a co-maintainer for one and a half years.

Qglif is the premier program of the Modular Font Editor Q project. This project aims to create a full font editor by making many small programs that will all work together, fulfilling the Unix adage that each program should have one task and do that task well. Qglif aims to do the task of drawing and editing glyphs well.

To make this as easy as possible to build, and cross-platform without hassle, the icon is compiled right into the binary via the Rust `include_str!` macro.

## Building

### Mac users

Apple charges a fee to "notarize" applications and without this "notarization" Qglif will not run correctly, or in some cases, at all. So, for the foreseeable future, you must _build Qglif from source on OS X_. This is not as hard as it sounds! :-)

* Download and install the [Vulkan SDK](https://vulkan.lunarg.com/).

### For everyone

* Download and install [`rustup`](https://rustup.rs/), selecting the `nightly` toolchain.
* Pull this repository, and finally
* Run the below command to get started.

### Errors?

If you previously pulled the repository and get errors related to `glifparser`, `mfeq-ipc`, or another local unstable dependency, try running `cargo update` to force Cargo to pull the latest versions from GitHub.

## Contributing

I typically build and run Qglif like this:

```
RUSTFLAGS=-Awarnings RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- Q_.glif
```

I welcome all contributions! Please open an issue first so we can discuss before you make big changes so no effort is wasted.

Please format the codebase with `cargo fmt` before opening a pull request.

### More debug output

It is possible to get even more debug output out of Qglif for figuring out where problems lie. To ask Qglif to dump the parsed .glif file on runtime, pass `DEBUG_DUMP_GLYPH=Y`. To see every single `winit` event (warning: this will flood your stdout) pass `DEBUG_EVENTS=Y`.

### Goals

Contributions which do not work on at least GNU/Linux and Windows will be rejected; we want to be able to build Qglif on as many platforms as possible. Both Skia and Dear ImGui are cross-platform; we use Vulkan and not OpenGL so we are future-proof even on OS X.

## License

Copyright 2020 Fredrick Brennan & MFEQ Authors

Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this software or any of the provided source code files except in compliance
with the License.  You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied.  See the License for the
specific language governing permissions and limitations under the License.

**By contributing you release your contribution under the terms of the license.**
