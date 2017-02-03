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

// SSE3 instrinsics for cumulative sum and conversion to pixels

#include <stdint.h>
#include <tmmintrin.h>

#ifdef _MSC_VER
  #define M128CAST _mm_castsi128_ps
#else
  #define M128CAST __m128
#endif

void accumulate_sse(const float *in, uint8_t *out, uint32_t n) {
  __m128 offset = _mm_setzero_ps();
  __m128i mask = _mm_set1_epi32(0x0c080400);
  __m128 sign_mask = _mm_set1_ps(-0.f);
  for (int i = 0; i < n; i += 4) {
    __m128 x = _mm_load_ps(&in[i]);
    x = _mm_add_ps(x, _mm_castsi128_ps(_mm_slli_si128(_mm_castps_si128(x), 4)));
    x = _mm_add_ps(x, _mm_shuffle_ps(_mm_setzero_ps(), x, 0x40));
    x = _mm_add_ps(x, offset);
    __m128 y = _mm_andnot_ps(sign_mask, x);  // fabs(x)
    y = _mm_min_ps(y, _mm_set1_ps(1.0f));
    y = _mm_mul_ps(y, _mm_set1_ps(255.0f));
    __m128i z = _mm_cvtps_epi32(y);
    z = _mm_shuffle_epi8(z, mask);
    _mm_store_ss((float *)&out[i], (M128CAST)(z));
    offset = _mm_shuffle_ps(x, x, _MM_SHUFFLE(3, 3, 3, 3));
  }
}
