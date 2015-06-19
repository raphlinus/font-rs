# font-rs

This is a font renderer written (mostly) in pure, safe Rust. There is an optional
SIMD module for cumulative sum, currently written in C++ SSE3 intrinsics.

The current state of the code is quite rough. It's not known to compile with
stable Rust 1.0 (the original version was written well before 1.0 stabilized),
it doesn't handle composite glyphs, the code isn't organized with Cargo, and
it's basically not ready for prime time. However, it ran well enough (at least
at one time) to run benchmarks, and those benchmarks suggest extremely promising
performance compared with Freetype and the Go port of FreeType.

## Authors

The main author is Raph Levien.

## Contributions

We gladly accept contributions via GitHub pull requests, as long as the author
has signed the Google Contributor License. Please see CONTRIBUTIONS.md for
more details.

### Disclaimer

This is not an official Google product (experimental or otherwise), it
is just code that happens to be owned by Google.
