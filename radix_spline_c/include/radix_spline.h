#ifndef RADIX_SPLINE_H
#define RADIX_SPLINE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RadixSplineU32 RadixSplineU32;
typedef struct RadixSplineU64 RadixSplineU64;

typedef struct {
  size_t start;
  size_t stop;
} RadixSplineSearchBound;

RadixSplineU32 *radix_spline_u32_build(const uint32_t *keys,
                                       size_t key_count,
                                       uint64_t radix_bits,
                                       uint32_t max_error);
RadixSplineSearchBound radix_spline_u32_find(const RadixSplineU32 *spline,
                                             uint32_t key);
size_t radix_spline_u32_size(const RadixSplineU32 *spline);
void radix_spline_u32_destroy(RadixSplineU32 *spline);

RadixSplineU64 *radix_spline_u64_build(const uint64_t *keys,
                                       size_t key_count,
                                       uint64_t radix_bits,
                                       uint64_t max_error);
RadixSplineSearchBound radix_spline_u64_find(const RadixSplineU64 *spline,
                                             uint64_t key);
size_t radix_spline_u64_size(const RadixSplineU64 *spline);
void radix_spline_u64_destroy(RadixSplineU64 *spline);

#ifdef __cplusplus
}
#endif

#endif
