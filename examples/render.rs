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

extern crate font_rs;

use std::fs::File;
use std::io::{Read, Write};
use std::time::SystemTime;

use font_rs::font::{parse, GlyphBitmap};

fn dump_pgm(glyph: &GlyphBitmap, out_filename: &str) {
    let mut o = File::create(&out_filename).unwrap();
    let _ = o.write(format!("P5\n{} {}\n255\n", glyph.width, glyph.height).as_bytes());
    println!("data len = {}", glyph.data.len());
    let _ = o.write(&glyph.data);
}

fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    let filename = args.next().unwrap();
    let glyph_id: u16 = args.next().unwrap().parse().unwrap();
    let out_filename = args.next().unwrap();
    let mut f = File::open(&filename).unwrap();
    let mut data = Vec::new();
    match f.read_to_end(&mut data) {
        Err(e) => println!("failed to read {}, {}", filename, e),
        Ok(_) => match parse(&data) {
            Ok(font) => {
                if out_filename == "__bench__" {
                    for size in 1..201 {
                        let start = SystemTime::now();
                        let n_iter = 1000;
                        for _ in 0..n_iter {
                            match font.render_glyph(glyph_id, size) {
                                Some(_glyph) => (),
                                None => (),
                            }
                        }
                        let elapsed = start.elapsed().unwrap();
                        let elapsed =
                            elapsed.as_secs() as f64 + 1e-9 * (elapsed.subsec_nanos() as f64);
                        println!("{} {}", size, elapsed * (1e6 / n_iter as f64));
                    }
                } else {
                    match font.render_glyph(glyph_id, 400) {
                        Some(glyph) => dump_pgm(&glyph, &out_filename),
                        None => println!("failed to render {} {}", filename, glyph_id),
                    }
                }
            }
            Err(_) => println!("failed to parse {}", filename),
        },
    }
}
