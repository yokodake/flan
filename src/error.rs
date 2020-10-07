//! Code for emitting and handling errors
//!
//! @DESIGN The goal is that if an error occurs we continue parsing the rest of the files
//! but I'm stil not sure whether copying should continue, stop or a rollback should occur.
use std::sync::Arc;

use crate::{
    debug,
    sourcemap::{Span, SrcFile, SrcMap},
};

#[derive(Clone, PartialEq, PartialOrd, Eq, Debug, Hash)]
pub struct Error {
    level: Level,
    msg: String,
    span: Span,
    extra: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Debug, Hash)]
pub enum Level {
    Fatal,
    Error,
    Warning,
    Note,
}
impl Level {
    pub fn is_fatal(&self) -> bool {
        match self {
            Level::Fatal => true,
            _ => false,
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
            }
        )
    }
}

impl Error {
    pub fn is_fatal(&self) -> bool {
        self.level.is_fatal()
    }
    /// a general error without any specific location
    pub fn error_general(msg: String) -> Self {
        Error {
            level: Level::Error,
            msg: msg,
            span: Span::MEMPTY,
            extra: Vec::new(),
        }
    }
    pub fn error(span: Span, msg: String) -> Self {
        Error {
            level: Level::Error,
            msg: msg,
            span: span,
            extra: Vec::new(),
        }
    }
    pub fn warn_general(msg: String) -> Self {
        Error {
            level: Level::Warning,
            msg: msg,
            span: Span::MEMPTY,
            extra: Vec::new(),
        }
    }
    pub fn warn(span: Span, msg: String) -> Self {
        Error {
            level: Level::Warning,
            msg: msg,
            span: span,
            extra: Vec::new(),
        }
    }
    pub fn note_general(msg: String) -> Self {
        Error {
            level: Level::Note,
            msg: msg,
            span: Span::MEMPTY,
            extra: Vec::new(),
        }
    }
    pub fn note(span: Span, msg: String) -> Self {
        Error {
            level: Level::Note,
            msg: msg,
            span: span,
            extra: Vec::new(),
        }
    }
    pub fn fatal_unexpected() -> Self {
        Error {
            level: Level::Fatal,
            msg: String::from("Unexpected fatal error."),
            span: Span::MEMPTY,
            extra: Vec::new(),
        }
    }
    pub fn fatal_span(msg: String, span: Span) -> Self {
        Error {
            level: Level::Fatal,
            msg,
            span,
            extra: Vec::new(),
        }
    }
    pub fn fatal(msg: String) -> Self {
        Error {
            level: Level::Fatal,
            msg: msg,
            span: Span::MEMPTY,
            extra: Vec::new(),
        }
    }
    /// add extra messages
    pub fn add_msg(&mut self, msg: String) -> &mut Self {
        self.extra.push(msg);
        self
    }
    pub fn levelu8(&self) -> u8 {
        match self.level {
            Level::Fatal => 1,
            Level::Error => 2,
            Level::Warning => 3,
            Level::Note => 4,
        }
    }
    pub fn render(&self, src: Option<SrcFile>) -> String {
        // I don't think this is a problem with String
        #![allow(unused_must_use)]
        use std::fmt::Write;

        let mut buf = format!("{}: {}\n", self.level, self.msg);
        let mut alignment = 0;

        debug!("err.span: {}", self.span);
        if src.is_some() {
            let src = src.unwrap();
            let line_ = src.lookup_line(self.span.lo);
            assert!(line_.is_some());
            let (num, line) = line_.map(|(num, line)| (num.to_string(), line)).unwrap();
            alignment = num.len() + 1;
            writeln!(buf, "  {}:{}", src.path.display(), self.span);
            writeln!(buf, "{}", Self::align_left("|", alignment));
            writeln!(buf, "{} | {}", num, line);
            // @TODO highlight span on line
            writeln!(buf, "{}", Self::align_left("|", alignment));
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

/// flags related to error reporting
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Debug, Hash)]
pub struct ErrorFlags {
    /// 0 = prints nothing, 1 = fatal errors only, 2 = also errors,
    /// 3 = also warnings, 4 = also notes/suggestions
    pub report_level: u8,
    /// treat warnings as errors (fail before copying)
    pub warn_as_error: bool,
    /// do not print extra notes & suggestions
    pub no_extra: bool,
}
impl Default for ErrorFlags {
    fn default() -> Self {
        ErrorFlags {
            report_level: 5,
            warn_as_error: false,
            no_extra: false,
        }
    }
}

#[derive(Debug)]
/// an error handler
pub struct Handler {
    pub flags: ErrorFlags,
    pub err_count: usize,
    /// errors than haven't been printed yet, these should be emitted
    /// if we abort (e.g. with a fatal error)
    pub delayed_err: Vec<Error>,
    pub sources: Arc<SrcMap>, // @TODO
}

impl Handler {
    pub fn new(flags: ErrorFlags, sources: Arc<SrcMap>) -> Self {
        Handler {
            flags,
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
    pub fn abort_now(&mut self) -> ! {
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
            std::process::exit(1)
        }
    }
    /// prints all the delayed errors
    pub fn print_all(&mut self) {
        while let Some(e) = self.delayed_err.pop() {
            Self::print_explicit(&self.flags, &self.sources, e);
        }
    }
    /// delay error reporting for later
    pub fn delay(&mut self, err: Error) {
        self.err_count += 1;
        self.delayed_err.push(err);
    }
    pub fn print(&mut self, err: Error) {
        self.err_count += 1;
        Self::print_explicit(&self.flags, &self.sources, err)
    }
    /// exists in order to avoid code duplication between `print` and `print_all` due to
    /// mutable borrow conflicts of `self`, despite borrowing two different fields
    /// ```ignore
    /// for e in self.delay_err.iter() { // immutable borrow
    ///   self.print(e) // mutable borrow
    /// }
    /// ```
    fn print_explicit(flags: &ErrorFlags, sources: &SrcMap, err: Error) {
        // @FIXME better error formatting with source files
        if flags.report_level >= err.levelu8() {
            println!("{}", err.render(sources.lookup_source(err.span.lo)));
        }
    }
    pub fn error<'a>(&'a mut self, msg: &str) -> ErrorBuilder<'a> {
        ErrorBuilder {
            handler: self,
            level: Level::Error,
            messages: vec![String::from(msg)],
            span: None,
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
    pub handler: &'a mut Handler,
    pub level: Level,
    /// `messages[0] = Error::message`, the rest are extras
    pub messages: Vec<String>,
    pub span: Option<Span>,
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
    /// consumes the builder and prints an error
    pub fn print(mut self) {
        let e = self.mk_error();
        self.handler.print(e)
    }
    /// consumes the builder and delays error reporting in the handler
    pub fn delay(mut self) {
        let e = self.mk_error();
        self.handler.delay(e)
    }
    /// consumes the builder, delay error reporting, and return a reference to it
    pub fn create(mut self) -> Error {
        self.mk_error()
    }

    fn add_extra(&mut self, msg: String) {
        if self.messages.is_empty() {
            self.messages.push(String::from(""));
        }
        self.messages.push(msg);
    }
    fn mk_error(&mut self) -> Error {
        let mut messages = Vec::new();
        std::mem::swap(&mut messages, &mut self.messages);
        let m = match messages.len() {
            0 => String::from(""),
            1 => messages.pop().unwrap(),
            _ => messages.swap_remove(0),
        };
        Error {
            level: self.level,
            msg: m,
            extra: messages,
            span: self.span.unwrap_or(Span::MEMPTY),
        }
    }
}
