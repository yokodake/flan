//! The Lexer module
//!
//! There are 4 meaningful tokens, anything else is considered text:
//! - `#{` variants opening delimiter
//! - `##` variants element separator
//! - `}#` veriants closing delimiter
//! - `#$IDENTIFIER#` variables where IDENTIFIER is made of alphanumeric characters or `!%&'*+-./:<=>?@_`
//!
//! There are two escapes (`\#` and `\\`), separators (`##`) need not to be escaped *outside* of variants.
//! @NOTE escape newlines inside of variants?

#![allow(dead_code)]
use core::str::Chars;

use crate::codemap::{span, Pos, Spanned};
use crate::error::Handler;
use crate::utils::*;

/// a `Lexer` is wrapper around a Buffered Reader
/// a stream of tokens is just like an iterator, so calling `next()` should yield the next token from the source.
pub struct Lexer<'a> {
    src: Chars<'a>,
    /// current position in the reader, helps for Spanned<>
    pos: Pos,
    /// number of Open Variant Delimiters
    nest: usize, // @NOTE usize is probably overkill
    handler: Box<Handler>,
}

// static items are not allowed inside implementations
static VAR_SYMS: [char; 16] = [
    '!', '%', '&', '\'', '*', '+', '-', '.', '/', ':', '<', '=', '>', '?', '@', '_',
];

impl<'a> Lexer<'a> {
    fn new(input: &'a str, h: Box<Handler>) -> Lexer<'a> {
        Lexer {
            src: input.chars(),
            /// current position, therefore the index of the result of getc()
            pos: Pos::from(0),
            nest: 0,
            handler: h,
        }
    }
    /// get the next character without consuming it
    fn peek1(&self) -> char {
        self.peek(0)
    }
    /// get the nth character without consuming any
    fn peek(&self, n: usize) -> char {
        self.src.clone().nth(n).unwrap_or('\0') // EOF
    }
    /// consumes and
    fn getc(&mut self) -> Option<char> {
        self.src.next().sequence(|_| self.pos += 1)
    }
    /// returns the next token
    pub fn next_token(&mut self) -> Token {
        let start = self.pos;
        while let Some(c) = self.getc() {
            match c {
                '\\' => {
                    self.getc();
                }
                '#' => match self.peek1() {
                    '{' => return self.lex_openv(start),
                    '$' => return self.lex_var(start),
                    '#' => {
                        if self.nest > 0 {
                            return self.lex_separator(start);
                        } else {
                            // separators have no meaning outside of variants, therefore we can skip them.
                            self.getc();
                            // @TODO if peek1 + peek2 is a meaningful token emit warning for not escaping current token.
                        }
                    }
                    _ => continue,
                },
                '}' => {
                    if self.peek1() == '#' {
                        return self.lex_closev(start);
                    } else {
                        continue;
                    }
                }
                _ => continue,
            };
        }
        Spanned::new(EOF, self.pos, self.pos + 1)
    }

    pub fn is_varsymbol(c: char) -> bool {
        c.is_alphanumeric() || VAR_SYMS.contains(&c)
    }

    pub fn lex_var(&mut self, start: Pos) -> Token {
        let mut err = false;
        self.getc(); // eat the '$'
        while let Some(c) = self.getc() {
            if Self::is_varsymbol(c) {
                continue;
            } else if c == '#' {
                return Token::new(Var, start, self.pos);
            } else if c.is_whitespace() {
                self.handler
                    .error("Non-terminated variable. Expected `#`, Found whitespace instead.")
                    .with_span(span(start, self.pos))
                    .note("Variables have the following syntax: #$variable#")
                    .print();
                self.handler.abort();
            } else if !err {
                // if we get none whitespace illegal characters, and the variable token is still correctly terminated
                // we can recover, maybe
                self.handler
                    .error(format!("Unexpected `{}` in variable name.", c).as_ref())
                    .with_span(span(start, self.pos))
                    .note(Self::identifier_note().as_ref())
                    .print();
                err = true;
            }
        }
        self.handler
            .error("Non-terminated variable, expected `#`.")
            .with_span(span(start, self.pos))
            .note("Variables have the following syntax: #$variable#")
            .print();
        self.handler.abort();
    }
    pub fn lex_openv(&mut self, start: Pos) -> Token {
        self.getc(); // eat the '{'
        self.nest += 1;
        Token::new(Openv, start, self.pos)
    }
    pub fn lex_closev(&mut self, start: Pos) -> Token {
        self.getc(); // eat the '#'

        // prevent underflow. The parser will catch the error.
        self.nest = std::cmp::max(self.nest - 1, 0);
        Token::new(Openv, start, self.pos)
    }
    pub fn lex_separator(&mut self, start: Pos) -> Token {
        self.getc(); // eat the '#'
        Token::new(Sepv, start, self.pos)
    }

    fn identifier_note() -> String {
        // 'a'','' ' for every element minus ", " for last element
        let mut verbose_varsym = String::with_capacity((VAR_SYMS.len() * 5) - 2);
        verbose_varsym.push('`');
        verbose_varsym.push(VAR_SYMS[0]);
        for x in VAR_SYMS[1..].iter() {
            verbose_varsym.push_str("`, `");
            verbose_varsym.push(*x);
        }
        verbose_varsym.push('`');
        format!(
            "Legal characters for variable identifiers are alphanumeric chars or one of {}.",
            verbose_varsym
        )
    }
}

#[allow(dead_code)]
pub const OPENV: &str = "#{";
#[allow(dead_code)]
pub const CLOSEV: &str = "}#";
#[allow(dead_code)]
pub const SEPV: &str = "##";
#[allow(dead_code)]
pub const PREVAR: &str = "#$";
#[allow(dead_code)]
pub const ENDVAR: char = '#';
// e.g. #$Key# = #{ $HOME ## foo_bar ## }#

pub type Token = Spanned<TokenK>;

impl Token {
    pub fn is_eof(&self) -> bool {
        self.node == EOF
    }
    pub fn is(&self, k: TokenK) -> bool {
        self.node == k
    }
    pub fn kind(&self) -> TokenK {
        self.node
    }
}
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum TokenK {
    Text,
    /// `#$identifier#`
    Var,
    /// `#{`
    Openv,
    /// `}#`
    Closev,
    /// `##`
    Sepv,
    EOF,
}
pub use TokenK::*;
