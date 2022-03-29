use std::fs;
use std::io;
use std::io::{BufRead, Write};

use crate::cfg;
use crate::infer::Env;
use crate::sourcemap::SrcFile;
use crate::syntax::{Term, TermK, Terms};

/// write multiple terms to the output.  
/// This will modify the ReadCtx to start span of each term.
#[inline]
pub fn write_terms<'a, R, W>(from: &mut ReadCtx<'a, R>, to: &mut WriteCtx<'a, W>, env: &Env, terms: &Terms) 
    -> io::Result<()> 
where R : BufRead, W: Write {
    for t in terms {
        let off = t.span.lo.as_usize() - from.pos;
        from.consume(off);
        // @TODO check how much has been written?
        write_term(from, to, env, t)?;
        // @TODO maybe it would be better to set `from.pos` to `t.span.hi` after the call
    }
    Ok(())
}

/// writes one term.  
/// this won't mutate [`ReadCtx::pos`] if not needed.  
/// @TODO maybe for consistency and better usage, we could set `from.pos` to `term.span.hi`
pub fn write_term<'a, R, W>(from: &mut ReadCtx<'a, R>, to: &mut WriteCtx<'a, W>, env: &Env, term: &Term) 
    -> io::Result<()> 
where R: BufRead, W: Write {
    // can we keep panics here? normally everything should be fine after typechecking
    // @TODO use write_vectored?
    match &term.node {
        TermK::Text => { pipe(from, to, term.span.len()) }
        TermK::Var(name) => match env.get_var(name) {
            Some(v) => {
                to.write(v.as_bytes())?;
                Ok(())
            }
            None if env.eflags().ignore_unset => Ok(()), // @FIXME verify if correct
            None => panic!("fatal write error: var `{}` not found", name),
        },
        TermK::Dimension { name, children } => match env.get_dimension(name) {
            Some(dim) => match children.get(dim.decision as usize) {
                Some(child) => write_terms(from, to, env, child),
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

/// a wrapper around [`BufRead`].  
/// To avoid copies we use [`BufRead::fill_buf`] but this means we have to keep
/// track ourselves of the position in the source file.  
/// @REFACTOR currently (to avoid copying) we have to peek/consum separately, 
/// it'd be nice to have a single read function that does both.
pub struct ReadCtx<'a, R : BufRead> {
    /// the buffered reader
    inner: &'a mut R,
    /// position in the inner reader
    pub(self) pos: usize,
}
impl<'a, R : BufRead> ReadCtx<'a, R> {
    #[inline]
    pub fn new(inner: &'a mut R, pos: impl Into<usize>) -> Self {
        ReadCtx { inner, pos: pos.into() }
    }
    #[inline]
    pub(self) fn peek_buf(&mut self, len: usize) -> io::Result<&[u8]> {
        let buf = self.inner.fill_buf()?;
        if buf.len() <= len {
            Ok(buf)
        } else {
            Ok(&buf[..len])
        }
    }
    #[inline]
    pub(self) fn consume(&mut self, len:usize) {
        self.inner.consume(len);
        self.pos += len;
    }
}

/// a wrapper around [`Write`].  
/// for future use
pub struct WriteCtx<'a, W : Write> {
    inner: &'a mut W, 
}
impl<'a, W : Write> WriteCtx<'a, W> {
    #[inline]
    pub fn new(inner: &'a mut W) -> Self {
        WriteCtx { inner }
    }
    #[inline]
    pub(self) fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
}

/// pipe `len` bytes from `from` to `to`
pub(self) fn pipe<'a, R, W>(from: &mut ReadCtx<'a, R>, to: &mut WriteCtx<'a, W>, len: usize) -> io::Result<()>
where R : BufRead, W : Write {
        let end = from.pos + len;
        while from.pos < end {
            let len = end - from.pos;
            let buf = from.peek_buf(len)?;
            let read = buf.len(); // satisfy the BBC
            // break if nothing read
            if read == 0 { dbg!("breaking"); break }
            debug_assert!(read <= len); // bytes read should always be <= to bytes requested
            to.write(buf)?;
            from.consume(read);
        };
        debug_assert!(from.pos == end, "read {} bytes too much", from.pos.abs_diff(end));
        Ok(())
}