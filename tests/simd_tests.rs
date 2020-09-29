#![allow(non_upper_case_globals)]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

const no_nl_128: *const __m128i = b"aaaabbbbccccdddd".as_ptr() as *const __m128i;
const two_nl_128: *const __m128i = b"aaa\nbbbbc\nccdddd".as_ptr() as *const __m128i;
const end_nl_128: *const __m128i = b"aaaabbbbccccddd\n".as_ptr() as *const __m128i;

unsafe fn nl_mask(ptr: *const __m128i) -> i32 {
    let chunk = _mm_loadu_si128(ptr);
    let t = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'\n' as i8));
    _mm_movemask_epi8(t)
}

#[test]
fn mask_zeros() {
    let m = unsafe { nl_mask(no_nl_128) };
    assert_eq!(m, 0);
}
#[test]
fn mask_two() {
    let m = unsafe { nl_mask(two_nl_128) };
    //              ddddccxc bbbbxaaa
    assert_eq!(m, 0b00000010_00001000);
}
#[test]
fn mask_four() {
    let m = unsafe { nl_mask(b"aa\nab\nbb\ncccd\ndd".as_ptr() as *const __m128i) };
    //              ddxd cccx bbxb axaa
    assert_eq!(m, 0b0010_0001_0010_0100)
}
#[test]
fn mask_end() {
    let m = unsafe { nl_mask(end_nl_128) };
    assert_eq!(m, 1 << 15);
}
#[test]
fn mask_end_avx() {
    use std::mem::transmute;
    let m = unsafe {
        let c = _mm256_loadu_si256(b"aaaabbbbccccddddaaaabbbbccccddd\n".as_ptr() as *const __m256i);
        let t = _mm256_cmpeq_epi8(c, _mm256_set1_epi8(b'\n' as i8));
        _mm256_movemask_epi8(t)
    };
    assert_eq!(m, unsafe { transmute(0x8000_0000 as u32) });
}

#[test]
fn zeros_no() {
    let m = unsafe { nl_mask(no_nl_128) } as u32 | 0xFFFF0000;
    assert_eq!(m.trailing_zeros(), 16);
}
#[test]
fn zeros_two() {
    let m = unsafe { nl_mask(two_nl_128) } as u32 | 0xFFFF0000;
    assert_eq!(m.trailing_zeros(), 3);
}
#[test]
fn zeros_end() {
    let m = unsafe { nl_mask(end_nl_128) } as u32 | 0xFFFF0000;
    assert_eq!(m.trailing_zeros(), 15);
}
#[test]
fn mm_avx() {
    let m = unsafe {
        let c = _mm256_loadu_si256(b"aaaabbbbccccddddaaaabbbbccccdddd".as_ptr() as *const __m256i);
        let t = _mm256_cmpgt_epi8(c, _mm256_set1_epi8(0));
        _mm256_movemask_epi8(t)
    };
    assert_eq!(m, -1);
}
