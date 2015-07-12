# Approaching zero-driver overhead with glium

These examples are ports of the examples from [the famous "Approaching zero-driver overhead" talk](http://gdcvault.com/play/1020791/) at GDC 2014.

[Here are the original codes](https://github.com/nvMcJohn/apitest).

## Running the examples

Go to each individual directory and `cargo run`. Note that this is very recent OpenGL, so it may not work on your system.

## Issues

 - Glium doesn't allow creating buffers in write-only mode.

 - The original `dynamic_streaming` example does something very inefficient in that it uses one draw call for each of the 16000 triangle. Doing the same thing with glium results in 16000 sync fences being created and managed, and leads to an horribly slow result. The example in this repository doesn't do this and submits everything at once instead, which gives the same result but can't be compared performance-wise to the original code.

 - When using multiple segments of the same vertex buffer, glium creates several VAOs instead of using glDrawElementsBaseVertex for example.

 - The `untextured-objects` example crashes when using too many object. The original example uses a multidraw-indirect buffer in RAM, while the example in this repo uses persistent mapping.

 - Sparse buffers & textures are not supported.

## Performances

This is no noticable performance difference between the `untextured-objects` code in this repo and the `GLBufferStorage-NoSDP` solution of the official `apitest`.
