# font-rs

This is a font renderer written (mostly) in pure, safe Rust. There is an optional
SIMD module for cumulative sum, currently written in C SSE3 intrinsics.

The current state of the code is quite rough. The code isn't well organized,
and it's basically not ready for prime time. However, it runs well enough to
run benchmarks, and those benchmarks suggest extremely promising performance
compared with Freetype and freetype-go (the loose port of Freetype to Go).

The rasterizer is basically very similar in design to
[libart](https://people.gnome.org/~mathieu/libart/internals.html), except that
vectors are drawn immediately into the buffer, rather than sorted and stored
in intermediate form, and that the buffer for rasterization is a dense array
rather than a sparse data structure. The main motivation for the latter is to
avoid branch misprediction and to better exploit data parallelism, both valid
trends in optimization since libart was originally written.

It's worth comparing the algorithm with that in
[Anti-Grain Geometry](http://projects.tuxee.net/cl-vectors/section-the-cl-aa-algorithm).
The original libart algorithm was also inspiration for the current antialiased
renderer in Freetype. All these renderers share many common features,
particularly computation of exact subpixel areas and an integration step
to determine winding number (and convert to pixel value), but differ in details
such as data structures to represent the vectors and the buffer.

The parsing of TrueType glyph data is done in pull-parser style, as iterators
over the lower-level data. This technique basically avoids allocating any
memory for representation of points and quadratic Beziers.

## Authors

The main author is Raph Levien.

## Contributions

We gladly accept contributions via GitHub pull requests, as long as the author
has signed the Google Contributor License. Please see CONTRIBUTIONS.md for
more details.

### Disclaimer

This is not an official Google product (experimental or otherwise), it
is just code that happens to be owned by Google.
