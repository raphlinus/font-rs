// Copyright 2015 Google Inc. All rights reserved.
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

//! Geometry primitive data structures and manipulations

use std::fmt::{Formatter, Result, Debug};

#[derive(Copy, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    // could be more generic, use conversion trait
    pub fn new(x: i16, y: i16) -> Point { Point{ x: x as f32, y: y as f32 } }
}

impl Debug for Point {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// maybe should be a static method of Point?
pub fn lerp(t: f32, p0: &Point, p1: &Point) -> Point {
    Point { x: p0.x + t * (p1.x - p0.x), y: p0.y + t * (p1.y - p0.y) }
}

pub struct Affine {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Affine {
    pub fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Affine {
        Affine{a: a, b: b, c: c, d: d, e: e, f: f}
    }
}

pub fn affine_pt(z: &Affine, p: &Point) -> Point {
    Point{x: z.a * p.x + z.c * p.y + z.e, y: z.b * p.x + z.d * p.y + z.f}
}
