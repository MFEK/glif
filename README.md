[**This project has nothing to do with QAnon and I oppose QAnon completely.**](https://github.com/mfeq/mfeq/blob/master/doc/QAnon.md)

# Qglif

Very early glyph editor for the Modular Font Editor Q project.

![screenshot](https://raw.githubusercontent.com/mfeq/Qglif/master/doc/screenshot.png)

## Overview

Qglif mixes three technologies: Skia, a very powerful path rasterizer and manipulation library; Dear ImGui, an immediate mode OpenGL user interface toolkit; and Rust, a modern high-performance systems language.

I wrote it after, hopefully, learning from the mistakes made by George Williams in FontForge, after being a user of FontForge for six years and a co-maintainer for one and a half years.

Qglif is the premier program of the Modular Font Editor Q project. This project aims to create a full font editor by making many small programs that will all work together, fulfilling the Unix adage that each program should have one task and do that task well. Qglif aims to do the task of drawing and editing glyphs well.

To make this as easy as possible to build, and cross-platform without hassle, the icon is compiled right into the binary via the Rust `include_str!` macro.

### Mac users

Apple charges a fee to "notarize" applications and without this "notarization" Qglif will not run correctly, or in some cases, at all. So, for the foreseeable future, you must _build Qglif from source on OS X_. This is not as hard as it sounds! :-)

* Download and install the [Vulkan SDK](https://vulkan.lunarg.com/).
* Download and install [`rustup`](https://rustup.rs/), selecting the `nightly` toolchain.
* Pull this repository, and finally
* Run the below command to get started.

## Contributing

I typically build and run Qglif like this:

```
RUSTFLAGS=-Awarnings DEBUG=y RUST_BACKTRACE=1 cargo run -- Q_.glif
```

I welcome all contributions! Please open an issue first so we can discuss before you make big changes so no effort is wasted.

Please format the codebase with `cargo fmt` before opening a pull request.

### Goals

Contributions which do not work on at least GNU/Linux and Windows will be rejected; we want to be able to build Qglif on as many platforms as possible. Both Skia and Dear ImGui are cross-platform and should work everywhere OpenGL works.

## License

Copyright 2020 Fredrick Brennan & MFEQ Authors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

**By contributing you release your contribution under the terms of the license.**
