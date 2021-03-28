[![Build Status](https://img.shields.io/github/workflow/status/MFEK/glif/Rust?label=Rust&logo=Rust)](https://github.com/MFEK/glif/actions?workflow=Rust)

# glif

Very early glyph editor for the Modular Font Editor K project.

<img src="https://raw.githubusercontent.com/MFEK/glif/master/doc/screenshot.png" width="300"><img src="https://raw.githubusercontent.com/MFEK/glif/master/doc/screenshot2.png" width="300">

## Overview

MFEKglif mixes three technologies: Skia, a very powerful path rasterizer and manipulation library; Dear ImGui, an immediate mode user interface toolkit; and Rust, a modern high-performance systems language.

I wrote it after, hopefully, learning from the mistakes made by George Williams in FontForge, after being a user of FontForge for six years and a co-maintainer for one and a half years.

MFEKglif is the premier program of the Modular Font Editor K project. This project aims to create a full font editor by making many small programs that will all work together, fulfilling the Unix adage that each program should have one task and do that task well. MFEKglif aims to do the task of drawing and editing glyphs well.

To make this as easy as possible to build, and cross-platform without hassle, the icon is compiled right into the binary via the Rust `include_str!` macro.

## Running from artifacts

MFEKglif is still alpha-quality software, and a numbered release hasn't been made yet. Before 1.0 is out, though, you can test it out with the artifacts function in GitHub. Go to [«Actions»](https://github.com/MFEK/glif/actions), choose a commit, and download the artifact for your OS. Three are uploaded: MFEKglif-linux, MFEKglif-windows, and MFEKglif-macos (not notarized).

## Building

### Mac users

Apple charges a fee to "notarize" applications and without this "notarization" MFEKglif will not run correctly, or in some cases, at all. So, for the foreseeable future, you must _build MFEKglif from source on OS X_. This is not as hard as it sounds! :-)

* Download and install the [Vulkan SDK](https://vulkan.lunarg.com/).

### For everyone

* Download and install [`rustup`](https://rustup.rs/), selecting the `nightly` toolchain.
* Pull this repository, and finally
* Run the below command to get started.

### Errors?

If you previously pulled the repository and get errors related to `glifparser`, `mfeq-ipc`, or another local unstable dependency, try running `cargo update` to force Cargo to pull the latest versions from GitHub.

## Contributing

I typically build and run MFEKglif like this:

```
RUSTFLAGS=-Awarnings RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- Q_.glif
```

I welcome all contributions! Please open an issue first so we can discuss before you make big changes so no effort is wasted.

Please format the codebase with `cargo fmt` before opening a pull request.

### More debug output

It is possible to get even more debug output out of MFEKglif for figuring out where problems lie. To ask MFEKglif to dump the parsed .glif file on runtime, pass `DEBUG_DUMP_GLYPH=Y`. To see every single `winit` event (warning: this will flood your stdout) pass `DEBUG_EVENTS=Y`.

### Goals

Contributions which do not work on at least GNU/Linux and Windows will be rejected; we want to be able to build MFEKglif on as many platforms as possible. Both Skia and Dear ImGui are cross-platform; we use Vulkan and not OpenGL so we are future-proof even on OS X.

## License

Copyright 2020 Fredrick Brennan & MFEK Authors

Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this software or any of the provided source code files except in compliance
with the License.  You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied.  See the License for the
specific language governing permissions and limitations under the License.

**By contributing you release your contribution under the terms of the license.**
