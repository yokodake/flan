//! The syntax module
//!
//! There are 4 meaningful tokens, anything else is considered text:
//! - `#{` variants opening delimiter
//! - `##` variants element separator
//! - `}#` veriants closing delimiter
//! - `#$IDENTIFIER#` variables where IDENTIFIER is made of alphanumeric characters or `!%&'*+-./:<=>?@_`
//!
//! There are two escapes (`\#` and `\\`), separators (`##`) need not to be escaped *outside* of variants.

#![allow(dead_code)]
use core::str::Chars;

use crate::codemap::{span, Pos, Spanned};
use crate::error;

/// a `Lexer` is wrapper around a Buffered Reader
/// a stream of tokens is just like an iterator, so calling `next()` should yield the next token from the source.
pub struct Lexer<'a> {
    src: Chars<'a>,
    /// current position in the reader, helps for Spanned<>
    pos: Pos,
    /// number of Open Variant Delimiters
    nesting: usize, // @NOTE usize is probably overkill
}

/// static items are not allowed inside implementations
static var_syms: &str = "!%&'*+-./:<=>?@_";

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            src: input.chars(),
            /// current position, therefore the index of the result of getc()
            pos: Pos::from(0),
            nesting: 0,
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
        self.src.next()
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
                        if self.nesting > 0 {
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
        c.is_alphanumeric() || var_syms.contains(c)
    }

    pub fn lex_var(&mut self, start: Pos) -> Token {
        self.getc(); // eat the '$'
        while let Some(c) = self.getc() {
            if Self::is_varsymbol(c) {
                continue;
            } else if c == '#' {
                return Token::new(Var, start, self.pos);
            } else {
                todo!("lex_var: error");
            }
        }
        // @TODO error reporting
        todo!("lex_var: error")
    }
    pub fn lex_openv(&mut self, start: Pos) -> Token {
        self.getc(); // eat the '{'
        self.nesting += 1;
        Token::new(Openv, start, self.pos)
    }
    pub fn lex_closev(&mut self, start: Pos) -> Token {
        self.getc(); // eat the '#'

        self.nesting -= 1; // @FIXME check whether we're not negative
        Token::new(Openv, start, self.pos)
    }
    pub fn lex_separator(&mut self, start: Pos) -> Token {
        self.getc(); // eat the '#'
        Token::new(Sepv, start, self.pos)
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
pub type Name = String;
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum TokenK {
    Text,
    /// `#$identifier`
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
