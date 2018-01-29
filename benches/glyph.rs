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
extern crate test;
extern crate font_rs;

use test::Bencher;
use font_rs::font;

use std::fs::File;
use std::io::Read;

fn glyphbench(b: &mut Bencher, size: u32) {
    let filename = "misc/wt024.ttf";
    let mut file = File::open(filename).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let font = font::parse(&data).unwrap();
    b.iter(|| font.render_glyph(6000, size));
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
