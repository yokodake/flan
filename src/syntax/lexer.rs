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

use core::str::Chars;

use crate::error::Handler;
use crate::sourcemap::{span, BytePos, Spanned};

/// parser error
pub struct Lexer<'a> {
    /// error handling
    pub handler: &'a mut Handler,
    src: Chars<'a>,
    /// current position in reader (index of `current`)
    pos: BytePos,
    /// next token = peek0
    next: Option<char>,
    /// current token
    current: Option<char>,
    /// number of Open dimension delimiters
    nest: usize, // @NOTE usize is probably overkill

    /// @REFACTOR
    failure: bool,
}

// static items are not allowed inside implementations
/// allowed non-alphanumerics inside IDENTIFIERs
static VAR_SYMS: [char; 16] = [
    '!', '%', '&', '\'', '*', '+', '-', '.', '/', ':', '<', '=', '>', '?', '@', '_',
];

impl<'a> Lexer<'a> {
    /// `Lexer.prev` is not valid, set to null
    pub fn new(h: &'a mut Handler, input: &'a str, offset: BytePos) -> Lexer<'a> {
        let mut l = Lexer {
            src: input.chars(),
            // current position, therefore the index of the result of getc()
            pos: offset,
            nest: 0,
            handler: h,
            current: None,
            next: None,
            failure: false,
        };
        l.current = l.src.next();
        l.next = l.src.next();
        l
    }
    /// did we encounter a failing lexing error
    pub fn failed(&self) -> bool {
        self.failure
    }
    /// get the next character without consuming it  
    /// @TODO rename/rethink the peek APIs
    fn peek0(&self) -> char {
        self.next.unwrap_or('\0')
    }
    /// bumps the src iterator, sets [`Self::current`] and [`Self::next`], increments [`Self::pos`] based on current.
    /// returns the [`Self::current`]
    fn bump(&mut self) -> Option<char> {
        self.current = self.next;
        self.next = self.src.next();
        // @FIXME don't increment more than once
        // @FIXME is this correct... if we're None it should be zero???
        self.pos += self.current.map_or(1, char::len_utf8);
        self.current.clone()
    }
    /// lexes the next token
    pub fn next_token(&mut self) -> Token {
        let mut start = self.pos;
        match self.current {
            None => return Spanned::new(EOF, start, self.pos),
            Some('\\') => match self.peek0() {
                '#' | '}' | '\\' => {
                    // we ignore the `\`, by updating `start` after eating it
                    self.bump();
                    start = self.pos;
                }
                _ => {}
            },
            // eat the '#' to avoid double `self.bump` in helper functions?
            Some('#') => match self.peek0() {
                '#' => {
                    if self.nest > 0 {
                        return self.lex_sepd(start);
                    }
                }
                '$' => return self.lex_var(start),
                c if Self::is_varstart(c) => {
                    if let Some(opend) = self.lex_opend_maybe(start) {
                        return opend;
                    }
                }
                // if None => return txt ?
                _ => {} // fallthrough
            },
            Some('}') => {
                if self.next == Some('#') {
                    return self.lex_closed(start);
                }
            }
            _ => {} // fall-through
        }
        // current isn't a meaningful lexeme start, so we can consume txt until next token
        while let Some(c) = self.bump() {
            match c {
                '#' => match self.peek0() {
                    '#' => {
                        if self.nest > 0 {
                            return self.lex_txt(start);
                        }
                    }
                    '$' => return self.lex_txt(start),
                    c if Self::is_varstart(c) => return self.lex_txt(start), // can we avoid this
                    _ => continue,
                },
                '\\' => {
                    return self.lex_txt(start);
                }
                // an escape is a meaningful token because we want 2 differnet text spans
                '}' => {
                    if self.peek0() == '#' {
                        return self.lex_txt(start);
                    }
                }
                _ => continue,
            }
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
    /// Makes a [`TokenK::Text`] from `start` to `self.pos`, i.e. all of the Text has been "consumed"
    pub fn lex_txt(&self, start: BytePos) -> Token {
        Token::new(Text, start, self.pos)
    }
    pub fn lex_var(&mut self, start: BytePos) -> Token {
        let mut ill_char = false;

        self.bump(); // eat '#'
        self.bump(); // eat '$'
        while let Some(c) = self.bump() {
            if Self::is_varsymbol(c) {
                continue;
            } else if c == '#' {
                self.bump(); // eat it
                return Token::new(Var, start, self.pos);
            } else if c.is_whitespace() {
                self.handler
                    .error("Non-terminated variable. Expected `#`, Found whitespace instead.")
                    .with_span(span(self.pos, self.pos))
                    .at_span("add `#` here")
                    .note("Variables have the following syntax: `#$variable#`")
                    .print();
                // return a wrong Var token, consumer of the TokenStream should check errors
                // @FIXME why did I do this again?
                return Token::new(Var, start, self.pos);
            } else if !ill_char {
                // if we get none-whitespace illegal characters, and the variable token is still correctly terminated
                // we can continue parsing
                self.handler
                    .error(format!("Unexpected `{}` in variable name.", c).as_ref())
                    .with_span(span(self.pos, self.pos))
                    .note(Self::identifier_note().as_ref())
                    .print();
                ill_char = true;
            }
        }
        self.handler
            .error("Non-terminated variable, expected `#`.")
            .with_span(span(start, start + 2))
            .at_span("variable starts here")
            .note("Variables have the following syntax: `#$VAR_NAME#`")
            .print();
        self.failure = true;
        // aborting here should be necessary because we're already at the end of the stream.
        // but dunno of a clean way
        assert!(false);
        Token::new(Var, start, self.pos)
    }
    pub fn lex_opend_maybe(&mut self, start: BytePos) -> Option<Token> {
        // eat opening '#'
        self.bump();
        while let Some(c) = self.current {
            if c.is_alphanumeric() || c == '_' {
                // fallthrough
            } else if c == '{' {
                self.bump(); // eat '{'
                self.nest += 1;
                return Some(Token::new(Opend, start, self.pos));
            } else {
                return None;
            }
            self.bump();
        }
        None
    }
    pub fn lex_closed(&mut self, start: BytePos) -> Token {
        // Just prevent underflow. The parser will catch the error.
        // should be asserts?
        self.bump(); // eat '}'
        self.bump(); // eat '#'
        self.nest = std::cmp::max(self.nest, 1) - 1;
        Token::new(Closed, start, self.pos)
    }
    pub fn lex_sepd(&mut self, start: BytePos) -> Token {
        self.bump(); // eat the '#'
        self.bump(); // eat the '#'
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
        Token::new(EOF, BytePos::from(0), BytePos::from(0))
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
