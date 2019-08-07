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

//! A very high performance font renderer.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc, core_intrinsics))]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[macro_use]
pub mod macros;
pub mod accumulate;
pub mod font;
pub mod geom;
pub mod raster;

#[cfg(not(feature = "std"))]
mod float32;
