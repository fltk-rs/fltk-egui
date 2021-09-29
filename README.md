# fltk-egui

An FLTK backend for Egui using a GlutWindow. The code is largely based on https://github.com/ArjunNair/egui_sdl2_gl modified for fltk-rs.

To run the examples, just run:
```
$ cargo run --example demo_windows
$ cargo run --example triangle
$ cargo run --example basic
$ cargo run --example embedded
```

A demo app can be found here:
https://github.com/fltk-rs/demos/tree/master/egui-demo

- [calculator2](examples/embedded.rs)
- ![alt_test](screenshots/egui.jpg)

## Usage
Add to your Cargo.toml:
```toml
[dependencies]
fltk = { version = "1.2.4", features = ["enable-glwindow"] }
fltk-egui = "0.2"
```

## Todo
- Properly handle resizing the GlutWindow: ✅
- Support egui_demo_lib crate directly: ✅
- Clipboard support (via optional features): ✅
