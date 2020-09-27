#![allow(non_upper_case_globals)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

const no_nl_128: *const __m128i = b"aaaabbbbccccdddd".as_ptr() as *const __m128i;
const two_nl_128: *const __m128i = b"aaa\nbbbbc\nccdddd".as_ptr() as *const __m128i;
const end_nl_128: *const __m128i = b"aaaabbbbccccddd\n".as_ptr() as *const __m128i;

#[test]
fn mask_zeros() {
    let mut m = unsafe {
        let chunk = _mm_loadu_si128(no_nl_128);
        let t = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'\n' as i8));
        _mm_movemask_epi8(t)
    };
    assert_eq!(m, 0);
}
#[test]
fn mask_two() {
    let mut m = unsafe {
        let chunk = _mm_loadu_si128(two_nl_128);
        let t = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'\n' as i8));
        _mm_movemask_epi8(t)
    };
    //              ddddccxc bbbbxaaa
    assert_eq!(m, 0b00000010_00001000);
}
#[test]
fn mask_end() {
    let mut m = unsafe {
        let chunk = _mm_loadu_si128(end_nl_128);
        let t = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'\n' as i8));
        _mm_movemask_epi8(t)
    };
    assert_eq!(m, 1 << 15);
}
