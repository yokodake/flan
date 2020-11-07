use std::fs;
use std::io;
use std::io::{BufRead, Write};

use crate::cfg;
use crate::infer::Env;
use crate::sourcemap::SrcFile;
use crate::syntax::{Term, TermK, Terms};
use crate::utils::RelativeSeek;

/// write the terms to `to`.
/// `start` is the current (starting) position in the Reader `from`
pub fn write_terms<R: RelativeSeek + BufRead>(
    terms: &Terms,
    from: &mut R,
    to: &mut impl Write,
    start: usize, // position in reader (relative to sourcemap though)
    env: &Env,
) -> io::Result<usize> {
    use std::io::SeekFrom;
    let mut pos = start;
    for t in terms {
        let off = t.span.lo.as_u64() - pos as u64;
        if off > i64::MAX as u64 {
            // i64::MAX is bigger than the buffer anyways
            from.seek(SeekFrom::Current(i64::MAX))?;
            let rest = off - i64::MAX as u64;
            from.seek_relative(rest as i64)?;
        } else {
            from.seek_relative(off as i64)?;
        }
        // @TODO check how much has been written
        pos += off as usize;
        pos = write_term(t, from, to, pos, env)?;
    }
    Ok(pos)
}

/// `start` is the current (starting) position in the `from` Reader
pub fn write_term<R: RelativeSeek + BufRead>(
    term: &Term,
    from: &mut R,
    to: &mut impl Write,
    pos: usize, // position in reader (relative to sourcemap though)
    env: &Env,
) -> io::Result<usize> {
    // can we keep panics here? normally everything should be fine after typechecking
    // @TODO use write_vectored?
    match &term.node {
        TermK::Text => {
            // safe alternative?
            let mut buf = unsafe { Box::<[u8]>::new_uninit_slice(term.span.len()).assume_init() };
            from.read(&mut buf)?;
            to.write(&buf)?;
            Ok(pos + term.span.len())
        }
        TermK::Var(name) => match env.get_var(name) {
            Some(v) => {
                to.write(v.as_bytes())?;
                Ok(pos + term.span.len())
            }
            None if env.eflags().ignore_unset => Ok(pos),
            None => panic!("fatal write error: var `{}` not found", name),
        },
        TermK::Dimension { name, children } => match env.get_dimension(name) {
            Some(dim) => match children.get(dim.decision as usize) {
                Some(child) => write_terms(child, from, to, pos, env),
                None => panic!("fatal write error: OOB decision for `{}`", name),
            },
            None => panic!("fatal write error: dim `{}` not found", name),
        },
    }
}

pub fn copy_bin(flags: &cfg::Flags, file: SrcFile) -> io::Result<()> {
    if !flags.force && file.destination.exists() {
        return Ok(());
    }
    fs::copy(&file.path, &file.destination)?;
    Ok(())
}
