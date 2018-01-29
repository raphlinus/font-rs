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
use font_rs::raster::*;
use font_rs::geom::Point;

fn draw_shape(r: &mut Raster, s: f32) {
    r.draw_line(&Point{x:s*10.0, y:s*10.5}, &Point{x: s*20.0, y: s*150.0});
    r.draw_line(&Point{x:s*20.0, y:s*150.0}, &Point{x: s*50.0, y: s*139.0});
    r.draw_quad(&Point{x:s*50.0, y:s*139.0}, &Point{x: s*100.0, y: s*60.0}, &Point{x: s*10.0, y: s*10.5});
}

#[bench]
fn empty200(b: &mut Bencher) {
    b.iter(|| {
        let w = 200;
        let h = 200;
        let r = Raster::new(w, h);
        r.get_bitmap()
    })
}

#[bench]
fn render200(b: &mut Bencher) {
    b.iter(|| {
        let w = 200;
        let h = 200;
        let mut r = Raster::new(w, h);
        draw_shape(&mut r, 1.0);
        r.get_bitmap()
    })
}

#[bench]
fn prep200(b: &mut Bencher) {
    b.iter(|| {
        let w = 200;
        let h = 200;
        let mut r = Raster::new(w, h);
        draw_shape(&mut r, 1.0);
    })
}

#[bench]
fn prep400(b: &mut Bencher) {
    b.iter(|| {
        let w = 400;
        let h = 400;
        let mut r = Raster::new(w, h);
        draw_shape(&mut r, 2.0);
    })
}

#[bench]
fn render400(b: &mut Bencher) {
    b.iter(|| {
        let w = 400;
        let h = 400;
        let mut r = Raster::new(w, h);
        draw_shape(&mut r, 2.0);
        r.get_bitmap()
    })
}

#[bench]
fn empty400(b: &mut Bencher) {
    b.iter(|| {
        let w = 400;
        let h = 400;
        let r = Raster::new(w, h);
        r.get_bitmap()
    })
}

#[bench]
fn alloc400(b: &mut Bencher) {
    b.iter(|| vec![0.0; 400 * 400 + 1])
}

#[bench]
fn alloc200(b: &mut Bencher) {
    b.iter(|| vec![0.0; 200 * 200 + 1])
}
