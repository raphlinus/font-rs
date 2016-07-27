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

//! A simple renderer for TrueType fonts

//extern crate test;
//use self::test::Bencher;

use std::collections::HashMap;
use std::fmt::{Formatter, Display};
//use std::hash::{Hash};
use std::io::{Read, Write};
use std::result::Result;
use std::fs::File;
//use std::path::Path;

use std::time::SystemTime;

use geom::{Point, lerp, Affine, affine_pt};
use raster::Raster;

mod geom;
mod raster;

#[derive(PartialEq, Eq, Hash)]
struct Tag(u32);

impl Tag {
  fn from_str(s: &str) -> Tag {
    Tag(get_u32(s.as_bytes(), 0).unwrap())
  }
}

impl Display for Tag {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    let &Tag(tag) = self;
    let buf = vec![((tag >> 24) & 0xff) as u8,
        ((tag >> 16) & 0xff) as u8,
        ((tag >> 8) & 0xff) as u8,
        (tag & 0xff) as u8];
    f.write_str(&String::from_utf8(buf).unwrap())
  }
}

fn get_u16(data: &[u8], off: usize) -> Option<u16> {
  if off + 1 > data.len() {
    None
  } else {
    Some(((data[off] as u16) << 8) | data[off + 1] as u16)
  }
}

fn get_i16(data: &[u8], off: usize) -> Option<i16> {
  get_u16(data, off).map(|x| x as i16)
}

fn get_u32(data: &[u8], off: usize) -> Option<u32> {
  if off + 3 > data.len() {
    None
  } else {
    Some(((data[off] as u32) << 24) | ((data[off + 1] as u32) << 16) |
      ((data[off + 2] as u32) << 8) | data[off + 3] as u32)
  }
}

// TODO: be consistent, use newtype or one-field struct everywhere
struct Head<'a>(&'a [u8]);

impl<'a> Head<'a> {
  fn index_to_loc_format(&'a self) -> i16 {
    let &Head(data) = self;
    get_i16(data, 50).unwrap()
  }

  fn units_per_em(&'a self) -> u16 {
    let &Head(data) = self;
    get_u16(data, 18).unwrap()
  }
}

struct Maxp<'a> {
  data: &'a [u8]
}

impl<'a> Maxp<'a> {
  fn num_glyphs(&'a self) -> u16 {
    get_u16(self.data, 4).unwrap()
  }
}

struct Loca<'a>(&'a [u8]);

impl<'a> Loca<'a> {
  fn get_off(&'a self, glyph_ix: u16, fmt: i16) -> Option<u32> {
    let &Loca(data) = self;
    if fmt != 0 {
      get_u32(data, glyph_ix as usize * 4)
    } else {
      get_u16(data, glyph_ix as usize * 2).map(|raw| raw as u32 * 2)
    }
  }
}

enum Glyph<'a> {
  EmptyGlyph,
  SimpleGlyph(SimpleGlyph<'a>),
  CompoundGlyph(CompoundGlyph<'a>),
}

struct SimpleGlyph<'a> {
  data: &'a [u8]
}

impl<'a> SimpleGlyph<'a> {
  fn number_of_contours(&'a self) -> i16 {
    get_i16(self.data, 0).unwrap()
  }

  fn bbox(&'a self) -> (i16, i16, i16, i16) {
    (get_i16(self.data, 2).unwrap(),
     get_i16(self.data, 4).unwrap(),
     get_i16(self.data, 6).unwrap(),
     get_i16(self.data, 8).unwrap(),
    )
  }

  fn points(&'a self) -> GlyphPoints<'a> {
    let data = self.data;
    let n_contours = self.number_of_contours();
    let insn_len_off = 10 + 2 * n_contours as usize;
    let n_points = get_u16(data, insn_len_off - 2).unwrap() as usize + 1;
    let insn_len = get_u16(data, insn_len_off).unwrap();  // insn_len
    let flags_ix = insn_len_off + insn_len as usize + 2;
    let mut flags_size = 0;
    let mut x_size = 0;
    let mut points_remaining = n_points;
    while points_remaining > 0 {
      let flag = data[flags_ix as usize + flags_size];
      let repeat_count = if (flag & 8) == 0 {
        1
      } else {
        flags_size += 1;
        data[flags_ix as usize + flags_size] as usize + 1
      };
      flags_size += 1;
      match flag & 0x12 {
        0x02 | 0x12 => x_size += repeat_count,
        0x00 => x_size += 2 * repeat_count,
        _ => ()
      }
      points_remaining -= repeat_count;
    }
    let x_ix = flags_ix + flags_size;
    let y_ix = x_ix + x_size;
    GlyphPoints{data: data, x: 0, y: 0, points_remaining: n_points,
      last_flag:0, flag_repeats_remaining: 0,
      flags_ix: flags_ix, x_ix: x_ix, y_ix: y_ix }
  }

  fn contour_sizes(&'a self) -> ContourSizes<'a> {
    let n_contours = self.number_of_contours();
    ContourSizes{data: self.data,
      contours_remaining: n_contours as usize,
      ix: 10,
      offset: -1,
    }
  }
}

struct GlyphPoints<'a> {
  data: &'a [u8],
  x: i16,
  y: i16,
  points_remaining: usize,
  last_flag: u8,
  flag_repeats_remaining: u8,
  flags_ix: usize,
  x_ix: usize,
  y_ix: usize,
}

impl<'a> Iterator for GlyphPoints<'a> {
  type Item = (bool, i16, i16);
  fn next(&mut self) -> Option<(bool, i16, i16)> {
    if self.points_remaining == 0 {
      None
    } else {
      if self.flag_repeats_remaining == 0 {
        self.last_flag = self.data[self.flags_ix];
        if (self.last_flag & 8) == 0 {
          self.flags_ix += 1;
        } else {
          self.flag_repeats_remaining = self.data[self.flags_ix + 1];
          self.flags_ix += 2;
        }
      } else {
        self.flag_repeats_remaining -= 1;
      }
      let flag = self.last_flag;
      //println!("flag={:02x}, flags_ix={}, x_ix={}, ({}) y_ix={} ({})",
      //  flag, self.flags_ix, self.x_ix, self.data.get(self.x_ix), self.y_ix, self.data.get(self.y_ix));
      match flag & 0x12 {
        0x02 => {
          self.x -= self.data[self.x_ix] as i16;
          self.x_ix += 1;
        },
        0x00 => {
          self.x += get_i16(self.data, self.x_ix).unwrap();
          self.x_ix += 2;
        }
        0x12 => {
          self.x += self.data[self.x_ix] as i16;
          self.x_ix += 1;
        },
        _ => ()
      }
      match flag & 0x24 {
        0x04 => {
          self.y -= self.data[self.y_ix] as i16;
          self.y_ix += 1;
        },
        0x00 => {
          self.y += get_i16(self.data, self.y_ix).unwrap();
          self.y_ix += 2;
        }
        0x24 => {
          self.y += self.data[self.y_ix] as i16;
          self.y_ix += 1;
        },
        _ => ()
      }
      self.points_remaining -= 1;
      Some(((self.last_flag & 1) != 0, self.x, self.y))
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.points_remaining as usize, Some(self.points_remaining as usize))
  }
}

struct ContourSizes<'a> {
  data: &'a [u8],
  contours_remaining: usize,
  ix: usize,
  offset: i32,
}

impl<'a> Iterator for ContourSizes<'a> {
  type Item = usize;
  fn next(&mut self) -> Option<(usize)> {
    if self.contours_remaining == 0 {
      None
    } else {
      let ret = get_u16(self.data, self.ix).unwrap() as i32 - self.offset;
      self.offset += ret;
      self.ix += 2;
      self.contours_remaining -= 1;
      Some(ret as usize)
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.contours_remaining, Some(self.contours_remaining))
  }
}

struct CompoundGlyph<'a> {
  data: &'a [u8]
}

struct Font<'a> {
  version: u32,
  tables: HashMap<Tag, &'a [u8]>,
  head: Head<'a>,
  maxp: Maxp<'a>,
  loca: Option<Loca<'a>>,
  glyf: Option<&'a [u8]>,
}

impl<'a> Font<'a> {
  fn get_glyph(&'a self, glyph_ix: u16) -> Option<Glyph<'a>> {
    if glyph_ix >= self.maxp.num_glyphs() { return None }
    let fmt = self.head.index_to_loc_format();
    match self.loca {
      Some(ref loca) => match (loca.get_off(glyph_ix, fmt), loca.get_off(glyph_ix + 1, fmt), self.glyf) {
        (Some(off0), Some(off1), Some(glyf)) =>
          if off0 == off1 {
            Some(Glyph::EmptyGlyph)
          } else {
            let glyph_data = &glyf[off0 as usize .. off1 as usize];
            if get_i16(glyph_data, 0) == Some(-1) {
              Some(Glyph::CompoundGlyph(CompoundGlyph{data: glyph_data}))
            } else {
              Some(Glyph::SimpleGlyph(SimpleGlyph{data: glyph_data}))
            }
          },
        (_, _, _) => None
      },
      None => None
    }
  }
}

#[derive(Debug)]
enum PathOp {
  MoveTo(Point),
  LineTo(Point),
  QuadTo(Point, Point),
}

use PathOp::{MoveTo, LineTo, QuadTo};

struct BezPathOps<T> {
  inner: T,
  first_oncurve: Option<Point>,
  first_offcurve: Option<Point>,
  last_offcurve: Option<Point>,
  alldone: bool,
  closing: bool,
}

fn path_from_pts<T: Iterator>(inner: T) -> BezPathOps<T> {
  BezPathOps{
    inner: inner, first_oncurve: None, first_offcurve: None, last_offcurve: None,
    alldone: false, closing: false
  }
}

impl<I> Iterator for BezPathOps<I> where I: Iterator<Item=(bool, i16, i16)> {
  type Item = PathOp;
  fn next(&mut self) -> Option<PathOp> {
    loop {
      if self.closing {
        if self.alldone {
          return None
        } else {
          match (self.first_offcurve, self.last_offcurve) {
            (None, None) => {
              self.alldone = true;
              return Some(LineTo(self.first_oncurve.unwrap()))
            },
            (None, Some(last_offcurve)) => {
              self.alldone = true;
              return Some(QuadTo(last_offcurve, self.first_oncurve.unwrap()))
            },
            (Some(first_offcurve), None) => {
              self.alldone = true;
              return Some(QuadTo(first_offcurve, self.first_oncurve.unwrap()))
            },
            (Some(first_offcurve), Some(last_offcurve)) => {
              self.last_offcurve = None;
              return Some(QuadTo(last_offcurve, lerp(0.5, &last_offcurve, &first_offcurve)))
            }
          }
        }
      } else {
        match self.inner.next() {
          None => {
            self.closing = true;
          },
          Some((oncurve, x, y)) => {
            let p = Point::new(x, y);
            if self.first_oncurve.is_none() {
              if oncurve {
                self.first_oncurve = Some(p);
                return Some(MoveTo(p));
              } else {
                match self.first_offcurve {
                  None => self.first_offcurve = Some(p),
                  Some(first_offcurve) => {
                    let midp = lerp(0.5, &first_offcurve, &p);
                    self.first_oncurve = Some(midp);
                    self.last_offcurve = Some(p);
                    return Some(MoveTo(midp));
                  }
                }
              }
            } else {
              match (self.last_offcurve, oncurve) {
                (None, false) => self.last_offcurve = Some(p),
                (None, true) => return Some(LineTo(p)),
                (Some(last_offcurve), false) => {
                  self.last_offcurve = Some(p);
                  return Some(QuadTo(last_offcurve, lerp(0.5, &last_offcurve, &p)));
                },
                (Some(last_offcurve), true) => {
                  self.last_offcurve = None;
                  return Some(QuadTo(last_offcurve, p));
                }
              }
            }
          }
        }
      }
    }
  }
}

enum FontError {
  Invalid
}

fn parse<'a>(data: &'a [u8]) -> Result<Font<'a>, FontError> {
  if data.len() < 12 {
    return Err(FontError::Invalid);
  }
  let version = get_u32(data, 0).unwrap();
  let numTables = get_u16(data, 4).unwrap() as usize;
  let searchRange = get_u16(data, 6).unwrap();
  let entrySelector = get_u16(data, 8).unwrap();
  let rangeShift = get_u16(data, 10).unwrap();
  let mut tables = HashMap::new();
  for i in 0..numTables {
    let header = &data[12 + i*16 .. 12 + (i + 1) * 16];
    let tag = get_u32(header, 0).unwrap();
    let checkSum = get_u32(header, 4).unwrap();
    let offset = get_u32(header, 8).unwrap();
    let length = get_u32(header, 12).unwrap();
    let tableData = &data[offset as usize .. (offset + length) as usize];
    //println!("{}: {}", Tag(tag), tableData.len())
    tables.insert(Tag(tag), tableData);
  }
  let head = Head(*tables.get(&Tag::from_str("head")).unwrap()); // todo: don't fail
  let maxp = Maxp{data: *tables.get(&Tag::from_str("maxp")).unwrap()};
  let loca = tables.get(&Tag::from_str("loca")).map(|&data| Loca(data));
  let glyf = tables.get(&Tag::from_str("glyf")).map(|&data| data);
  let f = Font{version: version, tables: tables,
    head: head,
    maxp: maxp,
    loca: loca,
    glyf: glyf,
  };
  //println!("version = {:x}", version);
  Ok(f)
}

fn dump_glyph(g: Glyph) {
  match g {
    Glyph::EmptyGlyph => println!("empty"),
    Glyph::SimpleGlyph(s) => {
      //println!("{} contours", s.number_of_contours())
      let mut p = s.points();
      for n in s.contour_sizes() {
        for _ in 0..n {
          println!("{:?}", p.next().unwrap());
        }
        println!("z");
      }
      let mut p = s.points();
      for n in s.contour_sizes() {
        for pathop in path_from_pts(p.by_ref().take(n)) {
          println!("{:?}", pathop);
        }
      }
    },
    _ => println!("other")
  }
}

fn dump(data: Vec<u8>) {
  println!("length is {}", data.len());
  match parse(&data) {
    Ok(font) => {
      println!("numGlyphs = {}", font.maxp.num_glyphs());
      for i in 0.. font.maxp.num_glyphs() {
        println!("glyph {}", i);
        match font.get_glyph(i) {
          Some(g) => dump_glyph(g),
          None => println!("glyph {} error", i)
        }
      }
    },
    _ => ()
  }
}

fn draw_path<I: Iterator<Item=PathOp>>(r: &mut Raster, z: &Affine, path: &mut I) {
  let mut lastp = Point::new(0, 0);
  for op in path {
    match op {
      MoveTo(p) => lastp = p,
      LineTo(p) => {
        r.draw_line(&affine_pt(z, &lastp), &affine_pt(z, &p));
        lastp = p
      },
      QuadTo(p1, p2) => {
        r.draw_quad(&affine_pt(z, &lastp), &affine_pt(z, &p1), &affine_pt(z, &p2));
        lastp = p2;
      }
    }
  }
}

struct GlyphBitmap {
  width: usize,
  height: usize,
  left: i32,
  top: i32,
  data: Vec<u8>,
}

// lifetime elision will certainly be effective here
fn render_glyph<'a>(f: &'a Font<'a>, glyph_id: u16, size: u32) -> Option<GlyphBitmap> {
  let ppem = f.head.units_per_em();
  let scale = (size as f32) / (ppem as f32);
  match f.get_glyph(glyph_id) {
    Some(Glyph::SimpleGlyph(s)) => {
      let (xmin, ymin, xmax, ymax) = s.bbox();
      let l = (xmin as f32 * scale).floor() as i32;
      let t = (ymax as f32 * -scale).floor() as i32;
      let r = (xmax as f32 * scale).ceil() as i32;
      let b = (ymin as f32 * -scale).ceil() as i32;
      let w = (r - l) as usize;
      let h = (b - t) as usize;
      let mut raster = Raster::new(w, h);
      let z = Affine::new(scale, 0.0, 0.0, -scale, -l as f32, -t as f32);
      //dump_glyph(SimpleGlyph(s));
      let mut p = s.points();
      for n in s.contour_sizes() {
        //println!("n = {}", n);
        draw_path(&mut raster, &z, &mut path_from_pts(p.by_ref().take(n)));
      }
      Some(GlyphBitmap{width: w, height: h, left: l, top: t, data: raster.get_bitmap()})
    },
    _ => {
      println!("glyph {} error", glyph_id);
      None
    }
  }
}

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
              match render_glyph(&font, glyph_id, size) {
                Some(_glyph) => (),
                None => println!("failed to render {} {}", filename, glyph_id)
              }
            }
            let elapsed = start.elapsed().unwrap();
            let elapsed = elapsed.as_secs() as f64 + 1e-9 * (elapsed.subsec_nanos() as f64);
            println!("{} {}", size, elapsed * (1e6 / n_iter as f64));
          }
        } else {
          match render_glyph(&font, glyph_id, 400) {
            Some(glyph) => dump_pgm(&glyph, &out_filename),
            None => println!("failed to render {} {}", filename, glyph_id)
          }
        }
      },
      Err(_) => println!("failed to parse {}", filename)
    }
  }

}

/*
TODO: get these benchmarks to work

fn glyphbench(b: &mut Bencher, size: u32) {
  let filename = "/Users/raph/Downloads/wt024.ttf";
  let mut f = File::open(filename).unwrap();
  let mut data = Vec::new();
  match f.read_to_end(&mut data) {
    Ok(_) => match parse(&data) {
      Ok(font) =>
        b.iter(|| render_glyph(&font, 6000, size)),
      _ => ()
    },
    _ => ()
  }
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
*/
