
use core::intrinsics::*;

/// Abstraction for floating point intrinsics in no_std mode
pub trait Float32 {
    fn abs(self) -> f32;
    fn floor(self) -> f32;
    fn ceil(self) -> f32;
    fn sqrt(self) -> f32;
}

impl Float32 for f32 {
    fn abs(self) -> f32 {
        unsafe { fabsf32(self) }
    }

    fn floor(self) -> f32 {
        unsafe { floorf32(self) }
    }

    fn ceil(self) -> f32 {
        unsafe { ceilf32(self) }
    }

    fn sqrt(self) -> f32 {
        unsafe { sqrtf32(self) }
    }
}
