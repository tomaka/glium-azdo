# Approaching zero-driver overhead with glium

These examples are ports of the examples from [the famous "Approaching zero-driver overhead" talk](http://gdcvault.com/play/1020791/) at GDC 2014.

[Here are the original codes](https://github.com/nvMcJohn/apitest).

## Running the examples

Go to each individual directory and `cargo run`. Note that this is very recent OpenGL, so it may not work on your system.

## Issues

 - Glium doesn't allow creating buffers in write-only mode.
 - Glium handles synchronization but locks the whole buffer instead of parts of it.

 - Bindless and sparse buffers & textures are not supported.
