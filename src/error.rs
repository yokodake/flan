//! Code for emitting and handling errors
//!
//! @DESIGN The goal is that if an error occurs we continue parsing the rest of the files
//! but I'm stil not sure whether copying should continue, stop or a rollback should occur.
use std::sync::Arc;

pub use crate::cfg::ErrorFlags;
use crate::sourcemap::{Span, SrcFile, SrcMap};

#[macro_export]
macro_rules! emit_error {
    ($($arg:tt)*) => ({
        $crate::error::Error::_emit($crate::error::Level::Error, format_args_nl!($($arg)*));
    })
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Debug, Hash)]
pub struct Error {
    level: Level,
    msg: String,
    /// error location
    span: Span,
    /// extra error messages
    extra: Vec<String>,
    /// message right under the error location
    at_span: String,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Debug, Hash)]
pub enum Level {
    Fatal,
    Error,
    Warning,
    Note,
    More,
}
impl Level {
    pub fn is_fatal(&self) -> bool {
        match self {
            Level::Fatal => true,
            _ => false,
        }
    }
    pub fn as_u8(&self) -> u8 {
        match self {
            Level::Fatal => 1,
            Level::Error => 2,
            Level::Warning => 3,
            Level::Note => 4,
            Level::More => 5,
        }
    }
}
impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Level::Fatal => "FATAL ERROR",
                Level::Error => "error",
                Level::Warning => "warning",
                Level::Note => "note",
                Level::More => "",
            }
        )
    }
}

impl Error {
    pub fn _emit(level: Level, args: std::fmt::Arguments) {
        use std::io::{self, Write};
        #[allow(unused_must_use)] {
            io::stderr().write(Self::with_msg(level, std::fmt::format(args))
                              .render(None)
                              .as_ref()
                              );
        }
    }
    pub fn is_fatal(&self) -> bool {
        self.level.is_fatal()
    }
    /// a general error without any specific location
    pub fn error_general(msg: String) -> Self {
        Self::with_msg(Level::Error, msg)
    }
    pub fn error(span: Span, msg: String) -> Self {
        Self::with_msg_span(Level::Error, msg, span)
    }
    pub fn warn_general(msg: String) -> Self {
        Self::with_msg(Level::Warning, msg)
    }
    pub fn warn(span: Span, msg: String) -> Self {
        Self::with_msg_span(Level::Warning, msg, span)
    }
    pub fn note_general(msg: String) -> Self {
        Self::with_msg(Level::Note, msg)
    }
    pub fn note(span: Span, msg: String) -> Self {
        Self::with_msg_span(Level::Note, msg, span)
    }
    pub fn fatal_unexpected() -> Self {
        Self::with_msg(Level::Fatal, String::from("Unexpected fatal error."))
    }
    pub fn fatal_span(msg: String, span: Span) -> Self {
        Self::with_msg_span(Level::Fatal, msg, span)
    }
    pub fn fatal(msg: String) -> Self {
        Self::with_msg(Level::Fatal, msg)
    }
    fn with_msg(level: Level, msg: String) -> Self {
        Self::with_msg_span(level, msg, Span::NIL)
    }
    fn with_msg_span(level: Level, msg: String, span: Span) -> Self {
        Error {
            level,
            msg,
            span,
            extra: Vec::new(),
            at_span: String::from(""),
        }
    }
    /// add extra messages
    pub fn add_msg(&mut self, msg: String) -> &mut Self {
        self.extra.push(msg);
        self
    }
    pub fn render(&self, src: Option<SrcFile>) -> String {
        // @SAFETY: write does not fail on Strings
        #![allow(unused_must_use)]
        use std::fmt::Write;

        let mut buf = format!("{}: {}\n", self.level, self.msg);
        let mut alignment = 3;

        if src.is_some() {
            write!(buf, "in {}", src.as_ref().unwrap().path.display());
            if !self.span.is_nil() {
                let src = src.unwrap();
                let line_ = src.lookup_line(self.span.lo);
                assert!(line_.is_some());
                let (lnum, line, lspan) = line_
                    .map(|loc| ((loc.index + 1).to_string(), loc.line, loc.span))
                    .unwrap();
                let rel_span = self.span.correct(lspan.lo);

                alignment = lnum.len() + 1;
                writeln!(buf, ":{}:{}", lnum, rel_span.lo + 1);

                writeln!(buf, "{}", Self::align_left("|", alignment));

                writeln!(buf, "{} | {}", lnum, line);

                // highlight span
                write!(buf, "{} ", Self::align_left("|", alignment));
                write!(buf, "{}", Self::align_left("", rel_span.lo.as_usize()));
                write!(buf, "{}", "^".repeat(rel_span.len()));
                writeln!(buf, " {}", self.at_span);

                writeln!(buf, "{}", Self::align_left("|", alignment));
            } else {
                writeln!(buf, "");
            }
        }
        for m in self.extra.iter() {
            writeln!(buf, "{} {}", Self::align_left("*", alignment), m);
        }
        buf
    }
    fn align_left(txt: &str, size: usize) -> String {
        let mut buf = String::with_capacity(size + txt.len());
        buf.push_str(" ".repeat(size).as_ref());
        buf.push_str(txt);
        buf
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.span != Span::MEMPTY {
            writeln!(f, " > filename:line_n:offset {}", self.span)?
        }
        if self.is_fatal() {
            return writeln!(f, "aborting...");
        }
        for m in self.extra.iter() {
            writeln!(f, "   {}", m)?
        }
        Ok(())
    }
}

#[derive(Debug)]
/// an error handler
pub struct Handler {
    pub eflags: ErrorFlags,
    pub err_count: usize,
    /// errors than haven't been printed yet, these should be emitted
    /// if we abort (e.g. with a fatal error)
    pub delayed_err: Vec<Error>,
    pub sources: Arc<SrcMap>,
}

impl Handler {
    pub fn new(eflags: ErrorFlags, sources: Arc<SrcMap>) -> Self {
        Handler {
            eflags,
            err_count: 0,
            delayed_err: Vec::new(),
            sources,
        }
    }
    /// prints delayed errors and [`Self::abort_now`]
    pub fn abort(&mut self) -> ! {
        self.print_all();
        self.abort_now()
    }
    /// aborts without printing delayed errors
    pub fn abort_now(&self) -> ! {
        if self.err_count > 1 {
            eprintln!("Aborting due to previous errors.");
        } else if self.err_count == 1 {
            eprintln!("Aborting due to previous error.");
        } else {
            eprintln!("Aborting.");
        }
        if cfg!(windows) {
            std::process::exit(0x100)
        } else {
            std::process::exit(-1)
        }
    }
    pub fn abort_if_err(&self) {
        if self.err_count > 0 {
            self.abort_now();
        }
    }
    /// prints all the delayed errors
    pub fn print_all(&mut self) {
        while let Some(e) = self.delayed_err.pop() {
            Self::eprint_explicit(&self.eflags, &self.sources, e);
        }
    }
    /// delay error reporting for later
    pub fn delay(&mut self, err: Error) {
        if err.level.as_u8() < Level::Warning.as_u8() {
            self.err_count += 1;
        }
        self.delayed_err.push(err);
    }
    pub fn print(&mut self, err: Error) {
        if err.level.as_u8() < Level::Warning.as_u8() {
            self.err_count += 1;
        }
        Self::eprint_explicit(&self.eflags, &self.sources, err)
    }
    /// exists in order to avoid code duplication between `print` and `print_all` due to
    /// mutable borrow conflicts of `self`, despite borrowing two different fields
    /// ```rs
    /// for e in self.delay_err.iter() { // immutable borrow
    ///   self.print(e) // mutable borrow
    /// }
    /// ```
    fn eprint_explicit(eflags: &ErrorFlags, sources: &SrcMap, err: Error) {
        if eflags.report_level >= err.level.as_u8() {
            eprintln!("{}", err.render(sources.lookup_source(err.span.lo)));
        }
    }
    pub fn error<'a>(&'a mut self, msg: &str) -> ErrorBuilder<'a> {
        let no_extra = self.eflags.no_extra;
        ErrorBuilder {
            handler: self,
            level: Level::Error,
            messages: vec![String::from(msg)],
            span: None,
            at_span: None,
            no_extra,
        }
    }
    pub fn note<'a>(&'a mut self, msg: &str) -> ErrorBuilder<'a> {
        let no_extra = self.eflags.no_extra;
        ErrorBuilder {
            handler: self,
            level: Level::Note,
            messages: vec![String::from(msg)],
            span: None,
            at_span: None,
            no_extra,
        }
    }
    pub fn warn<'a>(&'a mut self, msg: &str) -> ErrorBuilder<'a> {
        let no_extra = self.eflags.no_extra;
        let level = if self.eflags.warn_as_error {
            Level::Error
        } else {
            Level::Warning
        };
        ErrorBuilder {
            handler: self,
            level,
            messages: vec![String::from(msg)],
            span: None,
            at_span: None,
            no_extra,
        }
    }
}

/// similar to [`std::str::pattern::Pattern`]
pub trait Pattern<E> {
    fn found(&self, e: &E) -> bool;
}
impl<E, F> Pattern<E> for F
where
    F: Fn(&E) -> bool,
{
    fn found(&self, e: &E) -> bool {
        self(e)
    }
}
impl Pattern<Level> for Level {
    fn found(&self, e: &Level) -> bool {
        self == e
    }
}

pub struct ErrorBuilder<'a> {
    handler: &'a mut Handler,
    level: Level,
    /// `messages[0] = Error::message`, the rest are extras
    messages: Vec<String>,
    span: Option<Span>,
    at_span: Option<String>,
    no_extra: bool,
}

impl<'a> ErrorBuilder<'a> {
    /// adds an extra message as note
    pub fn note(mut self, msg: &str) -> Self {
        self.add_extra(format!("note: {}", msg));
        self
    }
    /// adds an extra message as suggestion
    pub fn suggest(mut self, msg: &str) -> Self {
        self.add_extra(format!("suggestion: {}", msg));
        self
    }
    /// should we refine or enlarge the span if they're different?
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
    /// adds a message under the error location
    pub fn at_span(mut self, msg: &str) -> Self {
        self.at_span = Some(String::from(msg));
        self
    }
    /// consumes the builder and prints an error
    pub fn print(self) {
        let (e, h) = self.create();
        h.print(e)
    }
    /// consumes the builder and delays error reporting in the handler
    pub fn delay(self) {
        let (e, h) = self.create();
        h.delay(e)
    }

    fn add_extra(&mut self, msg: String) {
        if self.no_extra {
            return;
        }
        if self.messages.is_empty() {
            self.messages.push(String::from(""));
        }
        self.messages.push(msg);
    }
    /// consume the builder to generate and error and return a Handler ref
    fn create(mut self) -> (Error, &'a mut Handler) {
        let m = match self.messages.len() {
            0 => String::from(""),
            1 => self.messages.pop().unwrap(),
            _ => self.messages.swap_remove(0),
        };

        (
            Error {
                level: self.level,
                msg: m,
                extra: self.messages,
                span: self.span.unwrap_or(Span::NIL),
                at_span: self.at_span.unwrap_or(String::from("")),
            },
            self.handler,
        )
    }

    pub fn is_error(&self) -> bool {
        match self.level {
            Level::Fatal | Level::Error => true,
            _ => false,
        }
    }
}
