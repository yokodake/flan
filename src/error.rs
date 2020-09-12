//! Code for emitting and handling errors
//!
//! @DESIGN The goal is that if an error occurs we continue parsing the rest of the files
//! but I'm stil not sure whether copying should continue, stop or a rollback should occur.

use std::io;

use crate::codemap::Span;

#[derive(Clone, PartialEq, PartialOrd, Eq, Debug, Hash)]
pub struct Error {
    level: Level,
    msg: String,
    span: Span,
    extra: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Debug, Hash)]
pub enum Level {
    Fatal(Option<io::ErrorKind>),
    Error,
    Warning,
    Note,
}

impl Error {
    pub fn is_fatal(&self) -> bool {
        match self.level {
            Level::Fatal(_) => true,
            _ => false,
        }
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
            level: Level::Fatal(None),
            msg: String::from("Unexpected fatal error"),
            span: Span::MEMPTY,
            extra: Vec::new(),
        }
    }
    pub fn fatal(err: io::ErrorKind) -> Self {
        Error {
            level: Level::Fatal(Some(err)),
            msg: String::from(""),
            span: Span::MEMPTY,
            extra: Vec::new(),
        }
    }
    pub fn fatal_msg(msg: String, err: io::ErrorKind) -> Self {
        Error {
            level: Level::Fatal(Some(err)),
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
            Level::Fatal(_) => 1,
            Level::Error => 2,
            Level::Warning => 3,
            Level::Note => 4,
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let i = match self.level {
            Level::Fatal(_) => "FATAL ERROR!",
            Level::Error => "error:",
            Level::Warning => "warning:",
            Level::Note => "note:",
        };
        writeln!(f, "{} {}", i, self.msg)?;
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
    report_level: u8,
    /// treat warnings as errors (fail before copying)
    warn_as_error: bool,
    /// do not print extra notes & suggestions
    no_extra: bool,
}

#[derive(Debug)]
/// an error handler
pub struct Handler {
    flags: ErrorFlags,
    printed_err: Vec<Error>,
    /// errors than haven't been printed yet, these should be emitted
    /// if we abort (e.g. with a fatal error)
    delayed_err: Vec<Error>,
    // map : srcFileMap, // @TODO
}

impl Handler {
    pub fn abort(&mut self) -> ! {
        self.print_all();
        if cfg!(windows) {
            std::process::exit(0x100)
        } else {
            std::process::exit(1)
        }
    }
    /// prints all the delayed errors
    pub fn print_all(&mut self) {
        // FIXME pop instead
        for e in self.delayed_err.iter() {
            Self::print_explicit(&self.flags, &mut self.printed_err, e.clone());
        }
        self.delayed_err.clear()
    }
    /// delay error reporting for later
    pub fn delay(&mut self, err: Error) {
        self.delayed_err.push(err);
    }
    pub fn print(&mut self, err: Error) {
        Self::print_explicit(&self.flags, &mut self.printed_err, err)
    }
    /// exists in order to avoid code duplication between `print` and `print_all` due to
    /// borrow issues, despite needing technically needing two different fields
    /// ```
    /// for e in self.delay_err.iter() { // immutable borrow
    ///   self.print(e) // mutable borrow
    /// ```
    fn print_explicit(flags: &ErrorFlags, printed: &mut Vec<Error>, err: Error) {
        // @FIXME better error formatting with source files
        if flags.report_level >= err.levelu8() {
            eprintln!("{}", err);
        }
        printed.push(err);
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

pub struct ErrorBuilder<'a> {
    handler: &'a mut Handler,
    level: Level,
    /// messages[0] = `Error::message`, the rest are extras
    messages: Vec<String>,
    span: Option<Span>,
}

impl<'a> ErrorBuilder<'a> {
    pub fn note(mut self, msg: &str) -> Self {
        self.add_extra(format!("note: {}", msg));
        self
    }
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
    pub fn print(self) {
        self.handler.print(self.mk_error())
    }
    /// consumes the builder and delays error reporting in the handler
    pub fn delay(self) {
        self.handler.delay(self.mk_error())
    }

    fn add_extra(&mut self, msg: String) {
        if self.messages.is_empty() {
            self.messages.push(String::from(""));
        }
        self.messages.push(msg);
    }
    fn mk_error(&self) -> Error {
        let (m, exs) = match self.messages.split_first() {
            Some((h, tl)) => (h.clone(), tl.to_vec()),
            None => (String::from(""), Vec::new()),
        };
        Error {
            level: self.level,
            msg: m,
            extra: exs,
            span: self.span.unwrap_or(Span::MEMPTY),
        }
    }
}
