// Copyright 2018 Google Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(test)]
extern crate font_rs;
extern crate test;

use font_rs::font;
use test::Bencher;

static FONT_DATA: &'static [u8] =
    include_bytes!("../fonts/notomono-hinted/NotoMono-Regular.ttf");

fn glyphbench(b: &mut Bencher, size: u32) {
    let font = font::parse(&FONT_DATA).unwrap();
    b.iter(|| font.render_glyph(200, size));
}

#[bench]
fn glyph400(b: &mut Bencher) {
    glyphbench(b, 400)
}

#[bench]
fn glyph100(b: &mut Bencher) {
    glyphbench(b, 100)
}

#[bench]
fn glyph040(b: &mut Bencher) {
    glyphbench(b, 40)
}

#[bench]
fn glyph020(b: &mut Bencher) {
    glyphbench(b, 20)
}

#[bench]
fn glyph010(b: &mut Bencher) {
    glyphbench(b, 10)
}
