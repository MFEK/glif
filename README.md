[**This project has nothing to do with QAnon and I oppose QAnon completely.**](https://github.com/mfeq/mfeq/blob/master/doc/QAnon.md)

# Qglif

Very early glyph editor for the Modular Font Editor Q project.

![screenshot](https://raw.githubusercontent.com/mfeq/Qglif/master/doc/screenshot.png)

## Overview

Qglif mixes three technologies: Skia, a very powerful path rasterizer and manipulation library; Dear ImGui, an immediate mode OpenGL user interface toolkit; and Rust, a modern high-performance systems language.

I wrote it after, hopefully, learning from the mistakes made by George Williams in FontForge, after being a user of FontForge for six years and a co-maintainer for one and a half years.

Qglif is the premier program of the Modular Font Editor Q project. This project aims to create a full font editor by making many small programs that will all work together, fulfilling the Unix adage that each program should have one task and do that task well. Qglif aims to do the task of drawing and editing glyphs well.

### Technical details

Two OpenGL contexts are made: one for Skia and one for Dear ImGui. Skia is redrawn only when necessary, while Dear ImGui is redrawn upon every frame. At the start of the program, `glifparser.rs` parses the input `.glif` file and creates a Skia path. Keyboard events are handled by `glutin`, which triggers the appropriate actions on both the Skia canvas and Dear ImGui's UI.

## Contributing

I typically build and run Qglif like this:

```
RUSTFLAGS=-Awarnings DEBUG=y RUST_BACKTRACE=1 cargo run -- Q_.glif
```

I welcome all contributions! Please open an issue first so we can discuss before you make big changes so no effort is wasted.

Please format the codebase with `cargo fmt` before opening a pull request.

### Goals

Contributions which do not work on at least GNU/Linux and Windows will be rejected; we want to be able to build Qglif on as many platforms as possible. Both Skia and Dear ImGui are cross-platform and should work everywhere OpenGL works.
