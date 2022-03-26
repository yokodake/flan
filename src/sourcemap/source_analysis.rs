use crate::sourcemap::BytePos;

pub unsafe fn anal_src_sse2(src: &str, offset: BytePos, lines: &mut Vec<BytePos>) {
    // see: https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_span/analyze_source_file.rs.html
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    const CHUNK_SIZE: usize = 16;
    let src_bytes = src.as_bytes();
    let chunk_count = src.len() / CHUNK_SIZE;

    for chunk_index in 0..chunk_count {
        let ptr = src_bytes.as_ptr() as *const __m128i;
        // loadu because we don't know if aligned to 16bytes
        // @TODO align before?
        let chunk = _mm_loadu_si128(ptr.offset(chunk_index as isize));

        let lines_test = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'\n' as i8));
        let lines_mask = _mm_movemask_epi8(lines_test);

        if lines_mask != 0 {
            // set the 16 irrelevant msb to '1'
            let mut lines_mask = 0xFFFF0000 | lines_mask as u32;
            // + 1 because we want the BytePosition of the newline start, not the '\n' before
            let offset = offset + BytePos::from(chunk_index * CHUNK_SIZE + 1);

            loop {
                let i = lines_mask.trailing_zeros();
                if i >= CHUNK_SIZE as u32 {
                    // end of chunk
                    break;
                }

                lines.push(BytePos::from(i) + offset);
                lines_mask &= (!1) << i;
            }
            // done with this chunk
            continue;
        } else {
            //  no newlines, nothing to do.
            continue;
        }
    }
    // non aligned bytes on tail
    let tail_start = chunk_count * CHUNK_SIZE;
    if tail_start < src.len() {
        anal_src_slow(
            &src[tail_start..],
            src.len() - tail_start,
            BytePos::from(tail_start) + offset,
            lines,
        );
    }
}

pub unsafe fn anal_src_avx2(src: &str, offset: BytePos, lines: &mut Vec<BytePos>) {
    // see: https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_span/analyze_source_file.rs.html
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    const CHUNK_SIZE: usize = 32;
    let src_bytes = src.as_bytes();
    // @FIXME
    let chunk_count = src.len() / CHUNK_SIZE;

    for chunk_index in 0..chunk_count {
        let ptr = src_bytes.as_ptr() as *const __m256i;
        // loadu because we don't know if aligned to 16bytes
        // @TODO align before?
        let chunk = _mm256_loadu_si256(ptr.offset(chunk_index as isize));

        let lines_test = _mm256_cmpeq_epi8(chunk, _mm256_set1_epi8(b'\n' as i8));
        let lines_mask = _mm256_movemask_epi8(lines_test);

        if lines_mask != 0 {
            // set the 16 irrelevant msb to '1'
            let mut lines_mask: u32 = std::mem::transmute(lines_mask);
            // + 1 because we want the BytePosition of the newline start, not the '\n' before
            let offset = offset + BytePos::from(chunk_index * CHUNK_SIZE + 1);

            loop {
                let i = lines_mask.trailing_zeros();
                if i >= CHUNK_SIZE as u32 {
                    // end of chunk
                    break;
                }

                lines.push(BytePos::from(i) + offset);
                lines_mask &= (!1) << i;
            }
            // done with this chunk
            continue;
        } else {
            //  no newlines, nothing to do.
            continue;
        }
    }
    // non aligned bytes on tail
    let tail_start = chunk_count * CHUNK_SIZE;
    if tail_start < src.len() {
        anal_src_slow(
            &src[tail_start..],
            src.len() - tail_start,
            BytePos::from(tail_start) + offset,
            lines,
        );
    }
}

pub fn anal_src_slow(src: &str, len: usize, offset: BytePos, lines: &mut Vec<BytePos>) {
    let src_bytes = src.as_bytes();
    for i in 0..len {
        let b = unsafe { *src_bytes.get_unchecked(i) };
        if b == b'\n' {
            // + 1 because we want the BytePosition of the newline start, not the '\n' before
            lines.push(BytePos::from(i) + offset + 1);
        }
    }
}
