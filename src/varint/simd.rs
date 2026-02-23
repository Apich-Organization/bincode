#![allow(unsafe_code)]
#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Results of a SIMD varint scan.
pub struct VarintScan {
    /// The number of consecutive single-byte varints found.
    pub single_byte_count: usize,
}

/// Scans a buffer for consecutive single-byte varints (0..=250).
/// This returns the number of bytes that can be safely read as individual varints.
#[inline(always)]
pub fn scan_single_byte_varints(bytes: &[u8]) -> VarintScan {
    let mut count = 0;

    #[cfg(target_arch = "x86_64")]
    {
        if bytes.len() >= 16 {
            unsafe {
                while count + 16 <= bytes.len() {
                    let chunk = _mm_loadu_si128(bytes.as_ptr().add(count) as *const __m128i);
                    // _mm_cmpgt_epi8 treats bytes as signed, so 0..127 are positive, 128..255 are negative.
                    // This makes it a bit tricky.
                    // Instead, let's use the unsigned comparison if available, or a trick.

                    // Trick: x <= 250 (unsigned) is equivalent to:
                    // (x ^ 0x80) <= (250 ^ 0x80) (signed)
                    // 250 ^ 0x80 = 250 - 128 = 122
                    let shifted = _mm_xor_si128(chunk, _mm_set1_epi8(-128));
                    let mask = _mm_cmpgt_epi8(shifted, _mm_set1_epi8(122));
                    let movemask = _mm_movemask_epi8(mask);

                    if movemask != 0 {
                        // Found a byte > 250.
                        count += movemask.trailing_zeros() as usize;
                        return VarintScan {
                            single_byte_count: count,
                        };
                    }
                    count += 16;
                }
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if bytes.len() >= 16 {
            unsafe {
                let limit = vdupq_n_u8(250);
                while count + 16 <= bytes.len() {
                    let chunk = vld1q_u8(bytes.as_ptr().add(count));
                    let mask = vcgtq_u8(chunk, limit);

                    if vmaxvq_u8(mask) != 0 {
                        // Found a byte > 250.
                        // Fallback loop will find the exact count.
                        break;
                    }
                    count += 16;
                }
            }
        }
    }

    // Fallback for remaining or non-x86
    while count < bytes.len() && bytes[count] <= 250 {
        count += 1;
    }

    VarintScan {
        single_byte_count: count,
    }
}
