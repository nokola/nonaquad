
# nvg-miniquad
Vector anti-aliased drawing for Android, WASM, Desktop in Rust.

This library started as a port of [NanoVG](https://github.com/sunli829/nvg/tree/master/nvg-gl) for [miniquad](https://github.com/not-fl3/miniquad). Use this library if you want to draw graphics for a quick experiment (game, paint app, etc.) or if you want to build other libraries on top (e.g. UI library.)

## Goals
* small and fast executables for mobile, desktop and web. 
* safety
* high-quality drawing: (anti-aliasing in shaders, squircles, gradients, fast blur)
* ease-of-use
* minimal dependencies

## Supported platforms

* Windows, OpenGl 3
* Linux, OpenGl 3
* macOS, OpenGL 3
* WASM, WebGl1 - tested on ios safari, ff, chrome
* Android, GLES3

## Not supported, but desirable platforms

* Android, GLES2 - work in progress.
* Metal. Highly desirable.
* Raspberry Pi - work in progress

# Example

```rust
self.nvg_context.begin_path();
self.nvg_context.rect((100.0, 100.0, 300.0, 300.0));
self.nvg_context.fill_paint(nvg::Gradient::Linear {
    start: (100, 100).into(),
    end: (400, 400).into(),
    start_color: nvg::Color::rgb_i(0xAA, 0x6C, 0x39),
    end_color: nvg::Color::rgb_i(0x88, 0x2D, 0x60),
});
self.nvg_context.fill().unwrap();

let origin = (150.0, 140.0);
self.nvg_context.begin_path();
self.nvg_context.circle(origin, 64.0);
self.nvg_context.move_to(origin);
self.nvg_context.line_to((origin.0 + 300.0, origin.1 - 50.0));
self.nvg_context.stroke_paint(nvg::Color::rgba(1.0, 1.0, 0.0, 1.0));
self.nvg_context.stroke_width(3.0);
self.nvg_context.stroke().unwrap();

self.nvg_context.end_frame().unwrap();
```

# Screenshots
Screenshots produced from above example.

## Windows
![Windows](https://user-images.githubusercontent.com/6869225/77838091-43983a80-7125-11ea-8581-83e51083e4f2.png)

## Web
![WebGL](https://user-images.githubusercontent.com/6869225/77838092-4e52cf80-7125-11ea-842f-665abfad59a9.png)
WASM size before size stripping 640KB - ought to be enough for everyone. 

## Android
APK size: 134KB

## iOS
(not yet ready)

# Building

## Linux

```bash
# ubuntu system dependencies
apt install libx11-dev libxi-dev libgl1-mesa-dev

cargo run --example drawaa
```

## Windows 

```bash
# both MSVC and GNU target is supported:
rustup target add x86_64-pc-windows-msvc
# or
rustup target add x86_64-pc-windows-gnu 

cargo run --example drawaa
```

## WASM
```bash
rustup target add wasm32-unknown-unknown
cargo build --example drawaa --target wasm32-unknown-unknown && copy ".\target\wasm32-unknown-unknown\debug\examples\drawaa.wasm" ".\examples\drawaa.wasm" /y

cd examples
npm i -g simplehttpserver
simplehttpserver
```
Then open `http://localhost:8000`

## Android

Recommended way to build for android is using Docker.   
miniquad use slightly modifed version of `cargo-apk`

**Note:** on Windows if you see git error during `cargo apk build --example drawaa`, update your .git folder to be not read-only. See related [Docker issue #6016](https://github.com/docker/for-win/issues/6016)

```
docker run --rm -v $(pwd)":/root/src" -w /root/src notfl3/cargo-apk cargo apk build --example drawaa
docker run -it -v %cd%":/root/src" -w /root/src notfl3/cargo-apk bash
```

APK file will be in `target/android-artifacts/(debug|release)/apk`

With feature "log-impl" enabled all log calls will be forwarded to adb console.
No code modifications for Android required, everything just works.

## iOS
Not supported yet

# Roadmap
The goal of nvg-miniquad is to have a stable, high-quality vector library on mobile, web, and desktop from the same source code.

I will use it as a building block for a general purpose cross-platform app framework.

## Features
- [x] anti-aliased lines, circles, rect, rounded rect (signed distance field), curves
- [x] polygons - convex and concave
- [x] gradients (standard)
- [x] AA text
- [x] image and textures
- [x] clipping
- [ ] high-quality fast drop shadows (blur)
- [ ] gradients (dithered)
- [ ] squircles

# Contributing
See TODO-s in source code or anything else goes

# License
MIT or APACHE at your convenience
