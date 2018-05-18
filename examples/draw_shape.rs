// Copyright 2016 Google Inc. All rights reserved.
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

//! A simple test program for exercising the rasterizer.

extern crate font_rs;

use std::io::{stdout, Write};

use font_rs::geom::Point;
use font_rs::raster::Raster;

fn draw_shape(r: &mut Raster, s: f32) {
    r.draw_line(
        &Point {
            x: s * 10.0,
            y: s * 10.5,
        },
        &Point {
            x: s * 20.0,
            y: s * 150.0,
        },
    );
    r.draw_line(
        &Point {
            x: s * 20.0,
            y: s * 150.0,
        },
        &Point {
            x: s * 50.0,
            y: s * 139.0,
        },
    );
    r.draw_quad(
        &Point {
            x: s * 50.0,
            y: s * 139.0,
        },
        &Point {
            x: s * 100.0,
            y: s * 60.0,
        },
        &Point {
            x: s * 10.0,
            y: s * 10.5,
        },
    );
}

fn main() {
    let w = 400;
    let h = 400;
    let mut r = Raster::new(w, h);
    draw_shape(&mut r, 4.0);
    let mut o = stdout();
    let _ = o.write(format!("P5\n{} {}\n255\n", w, h).as_bytes());
    let _ = o.write(&r.get_bitmap());
}
