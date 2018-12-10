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

use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::result::Result;

use geom::{affine_pt, Affine, Point};
use raster::Raster;

#[derive(PartialEq, Eq, Hash)]
struct Tag(u32);

impl Tag {
    fn from_str(s: &str) -> Tag {
        Tag(get_u32(s.as_bytes(), 0).unwrap())
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let &Tag(tag) = self;
        let buf = vec![
            ((tag >> 24) & 0xff) as u8,
            ((tag >> 16) & 0xff) as u8,
            ((tag >> 8) & 0xff) as u8,
            (tag & 0xff) as u8,
        ];
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

fn get_f2_14(data: &[u8], off: usize) -> Option<f32> {
    get_i16(data, off).map(|x| x as f32 * (1.0 / (1 << 14) as f32))
}

fn get_u32(data: &[u8], off: usize) -> Option<u32> {
    if off + 3 > data.len() {
        None
    } else {
        Some(
            ((data[off] as u32) << 24)
                | ((data[off + 1] as u32) << 16)
                | ((data[off + 2] as u32) << 8)
                | data[off + 3] as u32,
        )
    }
}

// TODO: be consistent, use newtype or one-field struct everywhere
struct Head<'a>(&'a [u8]);

impl<'a> Head<'a> {
    fn index_to_loc_format(&self) -> i16 {
        get_i16(self.0, 50).unwrap()
    }

    fn units_per_em(&self) -> u16 {
        get_u16(self.0, 18).unwrap()
    }
}

struct Maxp<'a> {
    data: &'a [u8],
}

impl<'a> Maxp<'a> {
    fn num_glyphs(&self) -> u16 {
        get_u16(self.data, 4).unwrap()
    }
}

struct Loca<'a>(&'a [u8]);

impl<'a> Loca<'a> {
    fn get_off(&self, glyph_ix: u16, fmt: i16) -> Option<u32> {
        if fmt != 0 {
            get_u32(self.0, glyph_ix as usize * 4)
        } else {
            get_u16(self.0, glyph_ix as usize * 2).map(|raw| raw as u32 * 2)
        }
    }
}

struct EncodingRecord<'a>(&'a [u8]);

impl<'a> EncodingRecord<'a> {
    fn get_platform_id(&self) -> u16 {
        get_u16(self.0, 0).unwrap()
    }

    fn get_encoding_id(&self) -> u16 {
        get_u16(self.0, 2).unwrap()
    }

    fn get_offset(&self) -> u32 {
        get_u32(self.0, 4).unwrap()
    }
}

impl<'a> Debug for EncodingRecord<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("EncodingRecord")
            .field("platformID", &self.get_platform_id())
            .field("encodingID", &self.get_encoding_id())
            .field("offset", &self.get_offset())
            .finish()
    }
}

struct Encoding<'a>(&'a [u8]);

impl<'a> Encoding<'a> {
    fn get_format(&self) -> u16 {
        get_u16(self.0, 0).unwrap()
    }

    fn get_length(&self) -> u16 {
        get_u16(self.0, 2).unwrap()
    }

    fn get_language(&self) -> u16 {
        get_u16(self.0, 4).unwrap()
    }
}

impl<'a> Debug for Encoding<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Encoding")
            .field("format", &self.get_format())
            .field("length", &self.get_length())
            .field("language", &self.get_language())
            .finish()
    }
}

struct EncodingFormat4<'a>(&'a [u8]);

impl<'a> EncodingFormat4<'a> {
    fn get_format(&self) -> u16 {
        get_u16(self.0, 0).unwrap()
    }

    fn get_length(&self) -> u16 {
        get_u16(self.0, 2).unwrap()
    }

    fn get_language(&self) -> u16 {
        get_u16(self.0, 4).unwrap()
    }

    fn get_seg_count_x_2(&self) -> u16 {
        get_u16(self.0, 6).unwrap()
    }

    fn get_seg_count(&self) -> u16 {
        self.get_seg_count_x_2() / 2
    }

    fn get_search_range(&self) -> u16 {
        get_u16(self.0, 8).unwrap()
    }

    fn get_entry_selector(&self) -> u16 {
        get_u16(self.0, 10).unwrap()
    }

    fn get_range_shift(&self) -> u16 {
        get_u16(self.0, 12).unwrap()
    }

    fn get_u16_vec(&self, start_position: u16, count: u16) -> Vec<u16> {
        let mut result = vec![];
        let mut vec_position = start_position;
        let limit = vec_position + 2 * count;
        while vec_position < limit {
            result.push(get_u16(self.0, vec_position as usize).unwrap());
            vec_position += 2;
        }
        result
    }

    fn get_i16_vec(&self, start_position: u16, count: u16) -> Vec<i16> {
        let mut result = vec![];
        let mut vec_position = start_position;
        let limit = vec_position + 2 * count;
        while vec_position < limit {
            result.push(get_i16(self.0, vec_position as usize).unwrap());
            vec_position += 2;
        }
        result
    }

    fn get_end_counts_position() -> u16 {
        14
    }

    fn get_end_counts(&self) -> Vec<u16> {
        let seg_count = self.get_seg_count();
        self.get_u16_vec(Self::get_end_counts_position(), seg_count)
    }

    fn get_start_counts_position(seg_count: u16) -> u16 {
        Self::get_end_counts_position() + 2 + 2 * seg_count
    }

    fn get_start_counts(&self) -> Vec<u16> {
        let seg_count = self.get_seg_count();
        self.get_u16_vec(Self::get_start_counts_position(seg_count), seg_count)
    }

    fn get_id_deltas_position(seg_count: u16) -> u16 {
        Self::get_start_counts_position(seg_count) + 2 * seg_count
    }

    fn get_id_deltas(&self) -> Vec<i16> {
        let seg_count = self.get_seg_count();
        self.get_i16_vec(Self::get_id_deltas_position(seg_count), seg_count)
    }

    fn get_id_range_offset_position(seg_count: u16) -> u16 {
        Self::get_id_deltas_position(seg_count) + 2 * seg_count
    }

    fn get_id_range_offsets(&self) -> Vec<u16> {
        let seg_count = self.get_seg_count();
        self.get_u16_vec(Self::get_id_range_offset_position(seg_count), seg_count)
    }

    fn extract_glyph_id(
        &self, code_point: u16, start_value: u16, seg_count: u16, seg_index: u16,
    ) -> Option<u16> {
        let data = self.0;
        let seg_index_pos = 2 * seg_index;
        let id_range_offset_pos = Self::get_id_range_offset_position(seg_count) + seg_index_pos;
        let id_range_offset_value = get_u16(data, id_range_offset_pos as usize).unwrap();
        let id_delta_pos = Self::get_id_deltas_position(seg_count) + seg_index_pos;
        let id_delta = get_i16(data, id_delta_pos as usize).unwrap();
        if id_range_offset_value == 0 {
            Some(code_point.wrapping_add(id_delta as u16))
        } else {
            let delta = (code_point - start_value) * 2;
            let pos = id_range_offset_pos.wrapping_add(delta) + id_range_offset_value;
            let glyph_array_value = get_u16(data, pos as usize).unwrap();
            if glyph_array_value == 0 {
                return None;
            }
            let glyph_index = (glyph_array_value as i16).wrapping_add(id_delta);
            Some(glyph_index as u16)
        }
    }

    pub fn lookup_glyph_id(&self, code_point: u16) -> Option<u16> {
        let end_counts_position = Self::get_end_counts_position();
        let seg_count = self.get_seg_count();
        let mut start = 0;
        let mut end = seg_count;
        while end > start {
            // Note: this is overflow-safe because seg_count < 0x8000
            let index = (start + end) / 2;
            let search = end_counts_position + index * 2;
            let end_value = get_u16(self.0, search as usize).unwrap();
            if end_value >= code_point {
                let start_pos = Self::get_start_counts_position(seg_count) + 2 * index;
                let start_value = get_u16(self.0, start_pos as usize).unwrap();
                if start_value > code_point {
                    end = index;
                } else {
                    return self.extract_glyph_id(code_point, start_value, seg_count, index);
                }
            } else {
                start = index + 1;
            }
        }
        None
    }
}

impl<'a> Debug for EncodingFormat4<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("EncodingFormat4")
            .field("format", &self.get_format())
            .field("length", &self.get_length())
            .field("language", &self.get_language())
            .field("segCountX2", &self.get_seg_count_x_2())
            .field("searchRange", &self.get_search_range())
            .field("entrySelector", &self.get_entry_selector())
            .field("rangeShift", &self.get_range_shift())
            .field("endCounts", &self.get_end_counts())
            .field("startCounts", &self.get_start_counts())
            .field("idDeltas", &self.get_id_deltas())
            .field("idRangeOffsets", &self.get_id_range_offsets())
            .finish()
    }
}

struct Cmap<'a>(&'a [u8]);

impl<'a> Cmap<'a> {
    fn get_version(&self) -> u16 {
        get_u16(self.0, 0).unwrap()
    }

    fn get_num_tables(&self) -> u16 {
        get_u16(self.0, 2).unwrap()
    }

    fn get_encoding_record(&self, index: u16) -> Option<EncodingRecord<'a>> {
        if index >= self.get_num_tables() {
            return None;
        }
        let enc_offset = (index * 8 + 4) as usize;
        let encoding_data = &self.0[enc_offset as usize..(enc_offset + 12) as usize];
        Some(EncodingRecord(encoding_data))
    }

    fn get_encoding_records(&self) -> Vec<EncodingRecord> {
        let mut encodings = vec![];
        for i in 0..self.get_num_tables() {
            encodings.push(self.get_encoding_record(i).unwrap());
        }
        encodings
    }

    fn get_encoding(&self, index: u16) -> Option<Encoding<'a>> {
        if index >= self.get_num_tables() {
            return None;
        }
        let record = self.get_encoding_record(index).unwrap();
        let subtable_len = get_u16(self.0, (record.get_offset() + 2) as usize).unwrap() as u32;
        let encoding_data =
            &self.0[record.get_offset() as usize..(record.get_offset() + subtable_len) as usize];
        Some(Encoding(encoding_data))
    }

    fn get_encoding_format_4_at(&self, index: u16) -> Option<EncodingFormat4<'a>> {
        let encoding = self.get_encoding(index);
        if encoding.is_none() || encoding.unwrap().get_format() != 4 {
            return None;
        }
        let record = self.get_encoding_record(index).unwrap();
        let subtable_len = get_u16(self.0, (record.get_offset() + 2) as usize).unwrap() as u32;
        let encoding_data =
            &self.0[record.get_offset() as usize..(record.get_offset() + subtable_len) as usize];
        Some(EncodingFormat4(encoding_data))
    }

    fn get_encodings(&self) -> Vec<Encoding> {
        let mut encodings = vec![];
        for i in 0..self.get_num_tables() {
            encodings.push(self.get_encoding(i).unwrap());
        }
        encodings
    }

    pub fn find_format_4_encoding(&self) -> Option<u16> {
        for index in 0..self.get_num_tables() {
            let encoding = self.get_encoding(index);
            if let Some(encoding) = encoding {
                if encoding.get_format() == 4 {
                    return Some(index);
                }
            }
        }
        None
    }
}

impl<'a> Debug for Cmap<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Cmap")
            .field("version", &self.get_version())
            .field("numTables", &self.get_num_tables())
            .field("encodingRecords", &self.get_encoding_records())
            .field("encodings", &self.get_encodings())
            .finish()
    }
}

fn get_bbox_raw(data: &[u8]) -> (i16, i16, i16, i16) {
    (
        get_i16(data, 2).unwrap(),
        get_i16(data, 4).unwrap(),
        get_i16(data, 6).unwrap(),
        get_i16(data, 8).unwrap(),
    )
}

enum Glyph<'a> {
    Empty,
    Simple(SimpleGlyph<'a>),
    Compound(CompoundGlyph<'a>),
}

struct SimpleGlyph<'a> {
    data: &'a [u8],
}

impl<'a> SimpleGlyph<'a> {
    fn number_of_contours(&self) -> i16 {
        get_i16(self.data, 0).unwrap()
    }

    fn bbox(&self) -> (i16, i16, i16, i16) {
        get_bbox_raw(self.data)
    }

    fn points(&self) -> GlyphPoints<'a> {
        let data = self.data;
        let n_contours = self.number_of_contours();
        let insn_len_off = 10 + 2 * n_contours as usize;
        let n_points = get_u16(data, insn_len_off - 2).unwrap() as usize + 1;
        let insn_len = get_u16(data, insn_len_off).unwrap(); // insn_len
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
                _ => (),
            }
            points_remaining -= repeat_count;
        }
        let x_ix = flags_ix + flags_size;
        let y_ix = x_ix + x_size;
        GlyphPoints {
            data: data,
            x: 0,
            y: 0,
            points_remaining: n_points,
            last_flag: 0,
            flag_repeats_remaining: 0,
            flags_ix: flags_ix,
            x_ix: x_ix,
            y_ix: y_ix,
        }
    }

    fn contour_sizes(&self) -> ContourSizes {
        let n_contours = self.number_of_contours();
        ContourSizes {
            data: self.data,
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
            // flag, self.flags_ix, self.x_ix, self.data.get(self.x_ix), self.y_ix,
            // self.data.get(self.y_ix));
            match flag & 0x12 {
                0x02 => {
                    self.x -= self.data[self.x_ix] as i16;
                    self.x_ix += 1;
                }
                0x00 => {
                    self.x += get_i16(self.data, self.x_ix).unwrap();
                    self.x_ix += 2;
                }
                0x12 => {
                    self.x += self.data[self.x_ix] as i16;
                    self.x_ix += 1;
                }
                _ => (),
            }
            match flag & 0x24 {
                0x04 => {
                    self.y -= self.data[self.y_ix] as i16;
                    self.y_ix += 1;
                }
                0x00 => {
                    self.y += get_i16(self.data, self.y_ix).unwrap();
                    self.y_ix += 2;
                }
                0x24 => {
                    self.y += self.data[self.y_ix] as i16;
                    self.y_ix += 1;
                }
                _ => (),
            }
            self.points_remaining -= 1;
            Some(((self.last_flag & 1) != 0, self.x, self.y))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.points_remaining as usize,
            Some(self.points_remaining as usize),
        )
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
    data: &'a [u8],
}

struct Components<'a> {
    data: &'a [u8],
    more: bool,
    ix: usize,
}

const ARG_1_AND_2_ARE_WORDS: u16 = 1;
const WE_HAVE_A_SCALE: u16 = 1 << 3;
const MORE_COMPONENTS: u16 = 1 << 5;
const WE_HAVE_AN_X_AND_Y_SCALE: u16 = 1 << 6;
const WE_HAVE_A_TWO_BY_TWO: u16 = 1 << 7;

impl<'a> Iterator for Components<'a> {
    type Item = (u16, Affine);
    fn next(&mut self) -> Option<(u16, Affine)> {
        if !self.more {
            return None;
        }
        let flags = get_u16(self.data, self.ix).unwrap();
        self.ix += 2;
        let glyph_index = get_u16(self.data, self.ix).unwrap();
        self.ix += 2;
        let arg1;
        let arg2;
        if (flags & ARG_1_AND_2_ARE_WORDS) != 0 {
            arg1 = get_i16(self.data, self.ix).unwrap();
            self.ix += 2;
            arg2 = get_i16(self.data, self.ix).unwrap();
            self.ix += 2;
        } else {
            arg1 = self.data[self.ix] as i16;
            self.ix += 1;
            arg2 = self.data[self.ix] as i16;
            self.ix += 1;
        }
        let mut a = 1.0;
        let mut b = 0.0;
        let mut c = 0.0;
        let mut d = 1.0;
        if (flags & WE_HAVE_A_TWO_BY_TWO) != 0 {
            a = get_f2_14(self.data, self.ix).unwrap();
            self.ix += 2;
            b = get_f2_14(self.data, self.ix).unwrap();
            self.ix += 2;
            c = get_f2_14(self.data, self.ix).unwrap();
            self.ix += 2;
            d = get_f2_14(self.data, self.ix).unwrap();
            self.ix += 2;
        } else if (flags & WE_HAVE_AN_X_AND_Y_SCALE) != 0 {
            a = get_f2_14(self.data, self.ix).unwrap();
            self.ix += 2;
            d = get_f2_14(self.data, self.ix).unwrap();
            self.ix += 2;
        } else if (flags & WE_HAVE_A_SCALE) != 0 {
            a = get_f2_14(self.data, self.ix).unwrap();
            self.ix += 2;
            d = a;
        }
        // TODO: handle non-ARGS_ARE_XY_VALUES case
        let x = arg1 as f32;
        let y = arg2 as f32;
        let z = Affine::new(a, b, c, d, x, y);
        self.more = (flags & MORE_COMPONENTS) != 0;
        Some((glyph_index, z))
    }
}

impl<'a> CompoundGlyph<'a> {
    fn bbox(&self) -> (i16, i16, i16, i16) {
        get_bbox_raw(self.data)
    }

    fn components(&self) -> Components {
        Components {
            data: self.data,
            ix: 10,
            more: true,
        }
    }
}

struct Hhea<'a>(&'a [u8]);

impl<'a> Hhea<'a> {
    fn get_ascent(&self) -> i16 {
        get_i16(self.0, 4).unwrap()
    }

    fn get_descent(&self) -> i16 {
        get_i16(self.0, 6).unwrap()
    }

    fn get_line_gap(&self) -> i16 {
        get_i16(self.0, 8).unwrap()
    }

    fn get_advance_width_max(&self) -> u16 {
        get_u16(self.0, 10).unwrap()
    }
}

pub struct Font<'a> {
    _version: u32,
    _tables: HashMap<Tag, &'a [u8]>,
    head: Head<'a>,
    maxp: Maxp<'a>,
    cmap: Option<Cmap<'a>>,
    loca: Option<Loca<'a>>,
    hhea: Option<Hhea<'a>>,
    glyf: Option<&'a [u8]>,
    encoding_index: Option<u16>,
}

struct Metrics {
    l: i32,
    t: i32,
    r: i32,
    b: i32,
}

impl Metrics {
    fn width(&self) -> usize {
        (self.r - self.l) as usize
    }

    fn height(&self) -> usize {
        (self.b - self.t) as usize
    }
}

impl<'a> Font<'a> {
    fn metrics_and_affine(
        &self, xmin: i16, ymin: i16, xmax: i16, ymax: i16, size: u32,
    ) -> (Metrics, Affine) {
        let ppem = self.head.units_per_em();
        let scale = (size as f32) / (ppem as f32);
        let l = (xmin as f32 * scale).floor() as i32;
        let t = (ymax as f32 * -scale).floor() as i32;
        let r = (xmax as f32 * scale).ceil() as i32;
        let b = (ymin as f32 * -scale).ceil() as i32;
        let metrics = Metrics {
            l: l,
            t: t,
            r: r,
            b: b,
        };
        let z = Affine::new(scale, 0.0, 0.0, -scale, -l as f32, -t as f32);
        (metrics, z)
    }

    fn render_glyph_inner(&self, raster: &mut Raster, z: &Affine, glyph: &Glyph) {
        match *glyph {
            Glyph::Simple(ref s) => {
                let mut p = s.points();
                for n in s.contour_sizes() {
                    //println!("n = {}", n);
                    //let v = path_from_pts(p.by_ref().take(n)).collect::<Vec<_>>();
                    //println!("size = {}", v.len());
                    draw_path(raster, z, &mut path_from_pts(p.by_ref().take(n)));
                }
            }
            Glyph::Compound(ref c) => {
                for (glyph_index, affine) in c.components() {
                    //println!("component {} {:?}", glyph_index, affine);
                    let concat = Affine::concat(z, &affine);
                    if let Some(component_glyph) = self.get_glyph(glyph_index) {
                        self.render_glyph_inner(raster, &concat, &component_glyph);
                    }
                }
            }
            _ => {
                println!("unhandled glyph case");
            }
        }
    }

    pub fn render_glyph(&self, glyph_id: u16, size: u32) -> Option<GlyphBitmap> {
        let glyph = self.get_glyph(glyph_id);
        match glyph {
            Some(Glyph::Simple(ref s)) => {
                let (xmin, ymin, xmax, ymax) = s.bbox();
                let (metrics, z) = self.metrics_and_affine(xmin, ymin, xmax, ymax, size);
                let mut raster = Raster::new(metrics.width(), metrics.height());
                //dump_glyph(SimpleGlyph(s));
                self.render_glyph_inner(&mut raster, &z, glyph.as_ref().unwrap());
                //None
                Some(GlyphBitmap {
                    width: metrics.width(),
                    height: metrics.height(),
                    left: metrics.l,
                    top: metrics.t,
                    data: raster.get_bitmap(),
                })
            }
            Some(Glyph::Compound(ref c)) => {
                let (xmin, ymin, xmax, ymax) = c.bbox();
                let (metrics, z) = self.metrics_and_affine(xmin, ymin, xmax, ymax, size);
                let mut raster = Raster::new(metrics.width(), metrics.height());
                self.render_glyph_inner(&mut raster, &z, glyph.as_ref().unwrap());
                Some(GlyphBitmap {
                    width: metrics.width(),
                    height: metrics.height(),
                    left: metrics.l,
                    top: metrics.t,
                    data: raster.get_bitmap(),
                })
            }
            _ => {
                println!("glyph {} error", glyph_id);
                None
            }
        }
    }

    fn get_glyph(&self, glyph_ix: u16) -> Option<Glyph> {
        if glyph_ix >= self.maxp.num_glyphs() {
            return None;
        }
        let fmt = self.head.index_to_loc_format();
        match self.loca {
            Some(ref loca) => match (
                loca.get_off(glyph_ix, fmt),
                loca.get_off(glyph_ix + 1, fmt),
                self.glyf,
            ) {
                (Some(off0), Some(off1), Some(glyf)) => if off0 == off1 {
                    Some(Glyph::Empty)
                } else {
                    let glyph_data = &glyf[off0 as usize..off1 as usize];
                    if get_i16(glyph_data, 0) == Some(-1) {
                        Some(Glyph::Compound(CompoundGlyph { data: glyph_data }))
                    } else {
                        Some(Glyph::Simple(SimpleGlyph { data: glyph_data }))
                    }
                },
                (_, _, _) => None,
            },
            None => None,
        }
    }

    pub fn lookup_glyph_id(&self, code_point: u32) -> Option<u16> {
        match self.encoding_index {
            Some(encoding_index) => {
                if code_point > u16::max_value() as u32 {
                    return None;
                }

                self.cmap
                    .as_ref()
                    .unwrap()
                    .get_encoding_format_4_at(encoding_index)
                    .unwrap()
                    .lookup_glyph_id(code_point as u16)
            }
            None => None,
        }
    }

    /// Gets the max descent of the font from the `hhea` table.
    pub fn max_descent(&self, font_size: u32) -> Option<i32> {
        self.hhea.as_ref().map(|hhea| hhea.get_descent() as i32 * font_size as i32 / self.head.units_per_em() as i32)
    }

    /// Gets the max ascent of the font from the `hhea` table.
    pub fn max_ascent(&self, font_size: u32) -> Option<i32> {
        self.hhea.as_ref().map(|hhea| hhea.get_ascent() as i32 * font_size as i32 / self.head.units_per_em() as i32)
    }

    /// Gets the line gap of the font from the `hhea` table.
    pub fn line_gap(&self, font_size: u32) -> Option<i32> {
        self.hhea.as_ref().map(|hhea| hhea.get_line_gap() as i32 * font_size as i32 / self.head.units_per_em() as i32)
    }

    /// Gets the max advance width of the font from the `hhea` table.
    pub fn max_advance_width(&self, font_size: u32) -> Option<u32> {
        self.hhea.as_ref().map(|hhea| hhea.get_advance_width_max() as u32 * font_size as u32 / self.head.units_per_em() as u32)
    }
}

#[derive(Debug)]
enum PathOp {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
}

use self::PathOp::{LineTo, MoveTo, QuadTo};

struct BezPathOps<T> {
    inner: T,
    first_oncurve: Option<Point>,
    first_offcurve: Option<Point>,
    last_offcurve: Option<Point>,
    alldone: bool,
    closing: bool,
}

fn path_from_pts<T: Iterator>(inner: T) -> BezPathOps<T> {
    BezPathOps {
        inner: inner,
        first_oncurve: None,
        first_offcurve: None,
        last_offcurve: None,
        alldone: false,
        closing: false,
    }
}

impl<I> Iterator for BezPathOps<I>
where
    I: Iterator<Item = (bool, i16, i16)>,
{
    type Item = PathOp;
    fn next(&mut self) -> Option<PathOp> {
        loop {
            if self.closing {
                if self.alldone {
                    return None;
                } else {
                    match (self.first_offcurve, self.last_offcurve) {
                        (None, None) => {
                            self.alldone = true;
                            return Some(LineTo(self.first_oncurve.unwrap()));
                        }
                        (None, Some(last_offcurve)) => {
                            self.alldone = true;
                            return Some(QuadTo(last_offcurve, self.first_oncurve.unwrap()));
                        }
                        (Some(first_offcurve), None) => {
                            self.alldone = true;
                            return Some(QuadTo(first_offcurve, self.first_oncurve.unwrap()));
                        }
                        (Some(first_offcurve), Some(last_offcurve)) => {
                            self.last_offcurve = None;
                            return Some(QuadTo(
                                last_offcurve,
                                Point::lerp(0.5, &last_offcurve, &first_offcurve),
                            ));
                        }
                    }
                }
            } else {
                match self.inner.next() {
                    None => {
                        self.closing = true;
                    }
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
                                        let midp = Point::lerp(0.5, &first_offcurve, &p);
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
                                    return Some(QuadTo(
                                        last_offcurve,
                                        Point::lerp(0.5, &last_offcurve, &p),
                                    ));
                                }
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

#[derive(Debug)]
pub enum FontError {
    Invalid,
}

pub fn parse(data: &[u8]) -> Result<Font, FontError> {
    if data.len() < 12 {
        return Err(FontError::Invalid);
    }
    let version = get_u32(data, 0).unwrap();
    let num_tables = get_u16(data, 4).unwrap() as usize;
    let _search_range = get_u16(data, 6).unwrap();
    let _entry_selector = get_u16(data, 8).unwrap();
    let _range_shift = get_u16(data, 10).unwrap();
    let mut tables = HashMap::new();
    for i in 0..num_tables {
        let header = &data[12 + i * 16..12 + (i + 1) * 16];
        let tag = get_u32(header, 0).unwrap();
        let _check_sum = get_u32(header, 4).unwrap();
        let offset = get_u32(header, 8).unwrap();
        let length = get_u32(header, 12).unwrap();
        let table_data = &data[offset as usize..(offset + length) as usize];
        //println!("{}: {}", Tag(tag), table_data.len());
        tables.insert(Tag(tag), table_data);
    }
    let head = Head(*tables.get(&Tag::from_str("head")).unwrap()); // todo: don't fail
    let maxp = Maxp {
        data: *tables.get(&Tag::from_str("maxp")).unwrap(),
    };
    let loca = tables.get(&Tag::from_str("loca")).map(|&data| Loca(data));
    let glyf = tables.get(&Tag::from_str("glyf")).map(|&data| data);
    let cmap = tables.get(&Tag::from_str("cmap")).map(|&data| Cmap(data));
    let hhea = tables.get(&Tag::from_str("hhea")).map(|&data| Hhea(data));
    let encoding_index = cmap.as_ref().and_then(|cmap| cmap.find_format_4_encoding());
    let f = Font {
        _version: version,
        _tables: tables,
        head: head,
        maxp: maxp,
        loca: loca,
        hhea: hhea,
        cmap: cmap,
        glyf: glyf,
        encoding_index: encoding_index,
    };
    //println!("version = {:x}", version);
    Ok(f)
}

/*
fn dump_glyph(g: Glyph) {
    match g {
        Glyph::Empty => println!("empty"),
        Glyph::Simple(s) => {
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
*/

/*
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
*/

fn draw_path<I: Iterator<Item = PathOp>>(r: &mut Raster, z: &Affine, path: &mut I) {
    let mut lastp = Point::new(0i16, 0i16);
    for op in path {
        match op {
            MoveTo(p) => lastp = p,
            LineTo(p) => {
                r.draw_line(&affine_pt(z, &lastp), &affine_pt(z, &p));
                lastp = p
            }
            QuadTo(p1, p2) => {
                r.draw_quad(
                    &affine_pt(z, &lastp),
                    &affine_pt(z, &p1),
                    &affine_pt(z, &p2),
                );
                lastp = p2;
            }
        }
    }
}

pub struct GlyphBitmap {
    pub width: usize,
    pub height: usize,
    pub left: i32,
    pub top: i32,
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {

    use font::parse;

    static FONT_DATA: &'static [u8] =
        include_bytes!("../fonts/notomono-hinted/NotoMono-Regular.ttf");

    #[test]
    fn test_cmap_format_4() {
        let font = parse(&FONT_DATA).unwrap();
        let cmap = font.cmap.as_ref().unwrap();
        assert!(cmap.get_encoding_record(cmap.get_num_tables()).is_none());
        assert!(cmap.get_encoding(cmap.get_num_tables()).is_none());
        assert_eq!(font.lookup_glyph_id('A' as u32).unwrap(), 36);
        assert_eq!(font.lookup_glyph_id(0x3c8).unwrap(), 405);
        assert_eq!(font.lookup_glyph_id(0xfffd).unwrap(), 589);
        assert_eq!(font.lookup_glyph_id(0x232B).is_none(), true);
        assert_eq!(font.lookup_glyph_id(0x1000232B).is_none(), true);
        // test for panics
        for i in 0..0x1ffff {
            font.lookup_glyph_id(i);
        }
    }

    static KOSUGI_MARU_ENCODING_4: &'static [u8] =
        include_bytes!("../fonts/kosugi_maru/kosugi_maru_enc_4.bin");

    static KANJI: &'static str = "\
        一九七二人入八力十下三千上口土夕大女子小山川五天中六円手文日月木水火犬王正出本右四左\
        玉生田白目石立百年休先名字早気竹糸耳虫村男町花見貝赤足車学林空金雨青草音校森刀万丸才\
        工弓内午少元今公分切友太引心戸方止毛父牛半市北古台兄冬外広母用矢交会合同回寺地多光当\
        毎池米羽考肉自色行西来何作体弟図声売形汽社角言谷走近里麦画東京夜直国姉妹岩店明歩知長\
        門昼前南点室後春星海活思科秋茶計風食首夏弱原家帰時紙書記通馬高強教理細組船週野雪魚鳥\
        黄黒場晴答絵買朝道番間雲園数新楽話遠電鳴歌算語読聞線親頭曜顔丁予化区反央平申世由氷主\
        仕他代写号去打皮皿礼両曲向州全次安守式死列羊有血住助医君坂局役投対決究豆身返表事育使\
        命味幸始実定岸所放昔板泳注波油受物具委和者取服苦重乗係品客県屋炭度待急指持拾昭相柱";

    #[test]
    fn test_glyph_lookup_format_4() {
        use font::EncodingFormat4;
        let encoding4 = EncodingFormat4(KOSUGI_MARU_ENCODING_4);
        for kanji in KANJI.chars() {
            assert!(encoding4.lookup_glyph_id(kanji as u16).is_some());
        }
        assert!(encoding4.lookup_glyph_id('\n' as u16).is_none());
    }
}
