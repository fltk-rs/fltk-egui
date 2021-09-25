# fltk-egui

This is a proof of concept for an FLTK backend for Egui using a GlutWindow. The code is largely based on https://github.com/ArjunNair/egui_sdl2_gl and https://github.com/not-fl3/egui-miniquad (to a lesser extent); modified for fltk-rs.

To run the examples, just run:
```
$ cargo run --example demo_windows
$ cargo run --example triangle
$ cargo run --example basic
$ cargo run --example embedded
```

A demo app can be found here:
https://github.com/fltk-rs/demos/tree/master/egui-demo

## Todo
- Properly handle resizing the GlutWindow: ✅
- Support egui_demo_lib crate directly: ✅
- Clipboard support (via optional features): ✅
