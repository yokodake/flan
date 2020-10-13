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
    pub fn levelu8(&self) -> u8 {
        match self.level {
            Level::Fatal => 1,
            Level::Error => 2,
            Level::Warning => 3,
            Level::Note => 4,
        }
    }
    pub fn render(&self, src: Option<SrcFile>) -> String {
        // @SAFETY: write does not fail on Strings
        #![allow(unused_must_use)]
        use std::fmt::Write;

        let mut buf = format!("{}: {}\n", self.level, self.msg);
        let mut alignment = 3;

        debug!("err.span: {}", self.span);
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
    /// don't process files, only parse and typecheck them  
    /// @NOTE instead of calling `infer::collect` we could just typecheck
    ///       and emit all variables/dimensions present in the environment
    pub dry_run: bool,
}
impl Default for ErrorFlags {
    fn default() -> Self {
        ErrorFlags {
            report_level: 5,
            warn_as_error: false,
            no_extra: false,
            dry_run: false,
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
    /// ```rs
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
            at_span: None,
        }
    }
    pub fn note<'a>(&'a mut self, msg: &str) -> ErrorBuilder<'a> {
        ErrorBuilder {
            handler: self,
            level: Level::Note,
            messages: vec![String::from(msg)],
            span: None,
            at_span: None,
        }
    }
    pub fn warn<'a>(&'a mut self, msg: &str) -> ErrorBuilder<'a> {
        ErrorBuilder {
            handler: self,
            level: Level::Note,
            messages: vec![String::from(msg)],
            span: None,
            at_span: None,
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
    // FIXME consume the builder
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
            span: self.span.unwrap_or(Span::NIL),
            // @FIXME
            at_span: self.at_span.clone().unwrap_or(String::from("")),
        }
    }

    pub fn is_error(&self) -> bool {
        match self.level {
            Level::Fatal | Level::Error => true,
            _ => false,
        }
    }
}
