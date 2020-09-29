//! The Lexer module
//!
//! There are 4 meaningful tokens, anything else is considered text:
//! - `#DIMID{` dimension opening delimiter where `DIMID` is made of alphanumerics and underscore `_`. Cannot start with numeric.
//! - `##` choices separator
//! - `}#` dimension closing delimiter
//! - `#$IDENTIFIER#` variables where `IDENTIFIER` is made of alphanumeric characters or `!%&'*+-./:<=>?@_`
//!
//! For now, there are two escapes (`\#` and `\\`), separators (`##`) need not to be escaped *outside* of dimensions.
//!
//! @TODO whitespace escape
//! @TODO escape first whitespace after `#..{`, before `}#` and around `##`.
//! @TODO allow newline escapes inside dimensions

#![allow(dead_code)]
use core::str::Chars;

use crate::codemap::{span, Pos, Spanned};
use crate::error::Handler;
use crate::syntax;
use crate::utils::*;

/// parser error
type PError = syntax::Error;
pub struct Lexer<'a> {
    /// error handling
    pub handler: &'a mut Handler,
    pub src: Chars<'a>,
    /// current position in the reader
    pub pos: Pos,
    /// number of Open dimension delimiters
    pub nest: usize, // @NOTE usize is probably overkill
    /// @TODO get rid of this field and use peek instead, Chars.clone() doesn't clone the underlying source
    pub prev: char,
    pub cur: Option<char>,

    failure: bool,
}

// static items are not allowed inside implementations
/// allowed non-alphanumerics inside IDENTIFIERs
static VAR_SYMS: [char; 16] = [
    '!', '%', '&', '\'', '*', '+', '-', '.', '/', ':', '<', '=', '>', '?', '@', '_',
];

impl<'a> Lexer<'a> {
    /// `Lexer.prev` is not valid, set to null
    pub fn new(input: &'a str, h: &'a mut Handler) -> Lexer<'a> {
        Lexer {
            src: input.chars(),
            // current position, therefore the index of the result of getc()
            pos: Pos::from(0 as usize),
            nest: 0,
            handler: h,
            prev: '\0',
            cur: None,
            failure: false,
        }
    }
    /// did we encounter a failing lexing error
    pub fn failed(&self) -> bool {
        self.failure
    }
    /// get the next character without consuming it
    fn peek1(&self) -> char {
        self.peek(0)
    }
    /// get the nth character without consuming any
    fn peek(&self, n: usize) -> char {
        self.src.clone().nth(n).unwrap_or('\0') // EOF
    }
    /// bumps the src iterator, sets [`Self::cur`] and [`Self::prev`], increments [`Self::pos`]
    /// `prev` is set to `\0` if cur was `None`
    fn getc(&mut self) -> Option<char> {
        self.cur = self.src.next().sequence(|_| {
            self.prev = self.cur.unwrap_or('\0');
            self.pos += 1
        });
        self.cur.clone()
    }
    /// returns the next token
    /// @REWRITE please...
    pub fn next_token(&mut self) -> Token {
        let start = self.pos;
        while let Some(c) = self.getc() {
            if self.prev == '#' && Self::is_varstart(c) {
                match self.lex_opend_maybe(start - 1) {
                    Some(t) => return t,
                    None => {}
                }
            }
            match c {
                '\\' => {
                    self.getc();
                }
                '{' => match self.prev {
                    '#' => continue, // @FIXME either fail or warn.
                    _ => continue,
                },
                '$' => match self.prev {
                    '#' => return self.lex_var(start - 1),
                    _ => continue,
                },
                '#' => match self.prev {
                    '#' => {
                        if self.nest > 0 {
                            return self.lex_sepd(start - 1);
                        } else {
                            continue;
                        }
                    }
                    '}' => return self.lex_closed(start),
                    _ => match self.peek1() {
                        '{' | '$' | '#' => return self.lex_txt(start),

                        c if Self::is_varstart(c) => return self.lex_txt(start), // might be a `#{` opening next
                        _ => continue,
                    },
                },
                '}' => match self.peek1() {
                    '#' => return self.lex_txt(start),
                    _ => continue,
                },
                _ => continue,
            };
        }
        if start != self.pos {
            self.lex_txt(start)
        } else {
            Spanned::new(EOF, start, self.pos)
        }
    }

    pub fn is_varstart(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }
    pub fn is_varsymbol(c: char) -> bool {
        c.is_alphanumeric() || VAR_SYMS.contains(&c)
    }
    pub fn lex_txt(&self, start: Pos) -> Token {
        Token::new(Text, start, self.pos)
    }
    pub fn lex_var(&mut self, start: Pos) -> Token {
        let mut err = false;
        while let Some(c) = self.getc() {
            if Self::is_varsymbol(c) {
                continue;
            } else if c == '#' {
                self.getc(); // eat it
                return Token::new(Var, start, self.pos - 1);
            } else if c.is_whitespace() {
                self.handler
                    .error("Non-terminated variable. Expected `#`, Found whitespace instead.")
                    .with_span(span(start, self.pos))
                    .note("Variables have the following syntax: #$variable#")
                    .print();
                self.failure = true;
                // return a wrong Var token, consumer of the TokenStream should check errors
                return Token::new(Var, start, self.pos);
            } else if !err {
                // if we get none-whitespace illegal characters, and the variable token is still correctly terminated
                // we can recover, maybe
                self.handler
                    // @FIXME illegal characters aren't fatal lexer errors.
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
        self.failure = true;
        // aborting here should be necessary because we're already at the end of the stream.
        // but dunno how we can abort from inside here
        Token::new(Var, start, self.pos)
    }
    /// useless?
    pub fn lex_opend(&mut self, start: Pos) -> Token {
        self.nest += 1;
        Token::new(Opend, start, self.pos)
    }
    pub fn lex_opend_maybe(&mut self, start: Pos) -> Option<Token> {
        while let Some(c) = self.getc() {
            if c.is_alphanumeric() || c == '_' {
                continue;
            } else if c == '{' {
                self.getc();
                self.nest += 1;
                return Some(Token::new(Opend, start, self.pos));
            } else {
                return None;
            }
        }
        None
    }
    pub fn lex_closed(&mut self, start: Pos) -> Token {
        // Just prevent underflow. The parser will catch the error.
        self.getc();
        self.nest = std::cmp::max(self.nest, 1) - 1;
        Token::new(Closed, start, self.pos)
    }
    pub fn lex_sepd(&mut self, start: Pos) -> Token {
        self.getc(); // eat the '#'
        Token::new(Sepd, start, self.pos)
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

/// a Spanned Token Kind
pub type Token = Spanned<TokenK>;

impl Token {
    pub fn is_eof(&self) -> bool {
        self.is(EOF)
    }
    /// is the token related to dimension or eof?
    pub fn is_dimension_or_eof(&self) -> bool {
        !(self.is(Var) || self.is(Text))
    }
    /// @NOTE copying should be cheap, or is derefing cheaper?
    pub fn is(&self, k: TokenK) -> bool {
        self.node == k
    }
    pub fn kind(&self) -> TokenK {
        self.node
    }
}
impl Default for Token {
    fn default() -> Token {
        Token::new(EOF, Pos::from(0 as u64), Pos::from(0 as u64))
    }
}

/// Kind of Token
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum TokenK {
    Text,
    /// `#$identifier#`
    Var,
    /// `#id{`
    Opend,
    /// `}#`
    Closed,
    /// `##`
    Sepd,
    EOF,
}
#[doc(hidden)]
pub use TokenK::*;
