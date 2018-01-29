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

#[cfg(feature = "sse")]
#[link(name = "accumulate")]
extern "C" {
    fn accumulate_sse(src: *const f32, dst: *mut u8, n: u32);
}

#[cfg(feature = "sse")]
pub fn accumulate(src: &[f32]) -> Vec<u8> {
    // SIMD instructions force us to align data since we iterate each 4 elements
    // So:
    // n (0) => 0
    // n (1 or 2 or 3 or 4) => 4,
    // n (5) => 8
    // and so on
    let len = src.len();
    let n = (len + 3) & !3; // align data
    let mut dst: Vec<u8> = Vec::with_capacity(n);
    unsafe {
        accumulate_sse(src.as_ptr(), dst.as_mut_ptr(), n as u32);
        dst.set_len(len); // we must return vec of the same length as src.len()
    }
    dst
}

#[cfg(not(feature = "sse"))]
pub fn accumulate(src: &[f32]) -> Vec<u8> {
    let mut acc = 0.0;
    src.iter()
        .map(|c| {
            // This would translate really well to SIMD
            acc += c;
            let y = acc.abs();
            let y = if y < 1.0 { y } else { 1.0 };
            (255.0 * y) as u8
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // The most simple and straightforward implementation of
    //  accumulate fn
    fn accumulate_simple_impl(src: &[f32]) -> Vec<u8> {
        let mut acc = 0.0;
        src.iter()
            .map(|c| {
                acc += c;
                let y = acc.abs();
                let y = if y < 1.0 { y } else { 1.0 };
                (255.0 * y) as u8
            })
            .collect()
    }
    fn test_accumulate(src: Vec<f32>) {
        assert_eq!(accumulate_simple_impl(&src), accumulate(&src));
    }

    #[test]
    fn max_255_from_1_0() {
        // 1.0 * 255.0 = 255.0 (max value)
        test_accumulate(vec![1.0]);
    }
    #[test]
    fn max_255_from_0_5() {
        // 0.5 * 2 = 1.0
        // 1.0 * 255.0 = 255.0 (max value)
        test_accumulate(vec![0.5; 2]);
    }
    #[test]
    fn max_255_from_0_25() {
        // 0.25 * 4 = 1.0
        // 1.0 * 255.0 = 255.0 (max value)
        test_accumulate(vec![0.25; 4]);
    }
    #[test]
    fn max_255_from_0_125() {
        // 0.125 * 8 = 1.0
        // 1.0 * 255.0 = 255.0 (max value)
        test_accumulate(vec![0.125; 8]);
    }
    #[test]
    fn max_255_from_0_0625() {
        // 0.0625 * 16 = 1.0
        // 1.0 * 255.0 = 255.0 (max value)
        test_accumulate(vec![0.0625; 16]);
    }
    #[test]
    fn max_255_from_0_03125() {
        // 0.03125 * 32 = 1.0
        // 1.0 * 255.0 = 255.0 (max value)
        test_accumulate(vec![0.03125; 32]);
    }
    #[test]
    fn max_255_from_0_015625() {
        // 0.015625 * 64 = 1.0
        // 1.0 * 255.0 = 255.0 (max value)
        test_accumulate(vec![0.015625; 64]);
    }
    #[test]
    fn max_255_from_0_0078125() {
        // 0.0078125 * 128 = 1.0
        // 1.0 * 255.0 = 255.0 (max value)
        test_accumulate(vec![0.0078125; 128]);
    }

    #[test]
    fn simple_0() {
        test_accumulate(vec![]);
    }
    #[test]
    fn simple_1() {
        test_accumulate(vec![0.1]);
    }
    #[test]
    fn simple_2() {
        test_accumulate(vec![0.1, 0.2]);
    }
    #[test]
    fn simple_3() {
        test_accumulate(vec![0.1, 0.2, 0.3]);
    }
    #[test]
    fn simple_4() {
        test_accumulate(vec![0.1, 0.2, 0.3, 0.4]);
    }
    #[test]
    fn simple_5() {
        test_accumulate(vec![0.1, 0.2, 0.3, 0.4, 0.5]);
    }
    #[test]
    fn simple_6() {
        test_accumulate(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6]);
    }
    #[test]
    fn simple_7() {
        test_accumulate(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7]);
    }
}
