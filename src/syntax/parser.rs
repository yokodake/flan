//! The Parsing module
//!
//! Aside from variable names, all text parsed is represented as a span to avoid
//! redundant memory usage.
//! The syntax:
//! ```bnf
//! Terms := Term*
//! Term  := Text
//!        | #$IDENTIFIER#
//!        | `#CHID{` Terms (`##` Terms)* `}#`
//!
//! DIMID := alphanumeric+
//! IDENTIFIER := (alphanumeric | [!%&'*+-./:<=>?@_])*
//! ```
//! Dimension identifiers (DIMID) should be named for now.
//!
//! A whole lot of ascii symbols are accepted in identifiers, probably too much, but we can and I figured it might
//! be interresting to have variables names of paths to contain slashes for example.
#![allow(dead_code)]
use std::collections::VecDeque;

use crate::codemap::{Span, Spanned};
use crate::error::Handler;
use crate::syntax::lexer::{Lexer, Token, TokenK};
use crate::syntax::Error;

/// type of a parsed expression
type Parsed<T> = Result<T, Error>;

pub struct Parser<'a> {
    // @FIXME remove mut
    handler: &'a mut Handler<Error>,
    current_token: Token,
    tokens: TokenStream,
    src: String,
}
impl Parser<'_> {
    pub fn new<'a>(input: String, h: &'a mut Handler<Error>, ts: TokenStream) -> Parser<'a> {
        let mut p = Parser {
            handler: h,
            current_token: Token::default(),
            tokens: ts,
            src: input,
        };
        p.next_token();
        p
    }

    pub fn parse_terms(&mut self) -> Parsed<Terms> {
        let mut terms = Vec::new();
        loop {
            match self.current_token.kind() {
                TokenK::Text | TokenK::Var => terms.append(&mut self.parse_alt()?),
                TokenK::Opend => terms.push(self.parse_dim()?),
                TokenK::EOF => return Ok(terms),
                k => {
                    self.handler
                        .error(
                            format!(
                                "Unexpected {}.",
                                match k {
                                    TokenK::Closed => "Dimension closing delimiter",
                                    TokenK::Sepd => "Dimension branch separator",
                                    _ => unreachable!(),
                                }
                            )
                            .as_ref(),
                        )
                        .with_span(self.current_token.span)
                        .with_kind(Error::UnexpectedToken)
                        .delay();
                    return Err(Error::UnexpectedToken);
                }
            };
            self.next_token();
        }
    }
    pub fn parse_var(&self) -> Parsed<Term> {
        let lo = self.current_token.span.lo_as_usize();
        let hi = self.current_token.span.hi_as_usize();
        // @SAFETY span is guaranteed to be valid by lexer
        let name = unsafe { self.src.get_unchecked(lo + 2..hi - 1) };
        Ok(Term::var(name.into(), self.current_token.span))
    }
    pub fn parse_txt(&self) -> Parsed<Term> {
        Ok(Term::text(self.current_token.span))
    }
    pub fn parse_alt(&mut self) -> Parsed<Terms> {
        let mut xs = Vec::new();
        while !self.current_token.is_dimension() {
            let x = match self.current_token.kind() {
                TokenK::Text => self.parse_txt()?,
                TokenK::Var => self.parse_var()?,
                _ => unreachable!(),
            };
            xs.push(x);
            self.next_token();
        }
        Ok(xs)
    }
    pub fn get_dim_name(&self) -> Name {
        let lo = self.current_token.span.lo_as_usize();
        let hi = self.current_token.span.hi_as_usize();
        // @TODO use get_unchecked instead?
        match self.src.get(lo + 1..hi - 1).map(String::from) {
            Some(s) => s,
            None => String::from(""),
        }
    }
    pub fn parse_dim(&mut self) -> Parsed<Term> {
        let start = self.current_token.span;
        let name = self.get_dim_name();
        self.next_token(); // eat Opend
        let mut cs = Vec::new();
        loop {
            let c = self.parse_terms()?;
            match self.current_token.kind() {
                TokenK::Closed => return Ok(Term::dim(name, cs, start + self.current_token.span)),
                TokenK::Sepd => {
                    cs.push(c);
                    self.next_token();
                }
                TokenK::EOF => todo!("error"),
                _ => unreachable!(),
            }
        }
    }

    fn next_token(&mut self) -> Token {
        self.current_token = match self.tokens.pop_front() {
            Some(t) => t,
            None => Token::new(TokenK::EOF, self.src.len().into(), self.src.len().into()),
        };
        self.current_token
    }
    #[allow(dead_code)]
    fn peek(&self, n: usize) -> Option<&Token> {
        if n == 0 {
            Some(&self.current_token)
        } else {
            self.tokens.get(n)
        }
    }
}

pub type Name = String;
pub type Terms = Vec<Term>;
pub type Term = Spanned<TermK>;
impl Term {
    pub fn text(s: Span) -> Term {
        Term {
            node: TermK::Text,
            span: s,
        }
    }
    pub fn var(n: Name, s: Span) -> Term {
        Term {
            node: TermK::Var(n),
            span: s,
        }
    }
    pub fn dim(n: String, cs: Vec<Terms>, s: Span) -> Term {
        Term {
            node: TermK::Dimension {
                name: n,
                children: cs,
            },
            span: s,
        }
    }
}
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum TermK {
    Text,
    Var(Name),
    Dimension { name: String, children: Vec<Terms> },
}

type TokenStream = VecDeque<Token>;

pub fn source_to_stream(h: &mut Handler<Error>, src: &str) -> TokenStream {
    let mut vd = VecDeque::new();
    let mut lexer = Lexer::new(src, h);
    loop {
        let t = lexer.next_token();
        vd.push_back(t);
        if t.is_eof() {
            break;
        }
    }
    vd
}

pub fn string_to_parser<'a>(h: &'a mut Handler<Error>, str: String) -> Parser<'a> {
    let ts = source_to_stream(h, str.as_ref());
    Parser::new(str, h, ts)
}

use crate::codemap::SrcFile;
use std::io;
pub fn file_to_parser<'a>(h: &'a mut Handler<Error>, src: &mut SrcFile) -> io::Result<Parser<'a>> {
    use crate::codemap::Source;
    use std::io::{Error, ErrorKind};
    // @SPEED stop cloning sources
    match &src.src {
        Source::Binary => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "binary data cannot be parsed",
            ))
        }
        Source::Src(s) => return Ok(string_to_parser(h, s.clone())),
        Source::NotLoaded => {
            let s = std::fs::read_to_string(&src.absolute_path)?;
            src.src = Source::Src(s.clone());
            return Ok(string_to_parser(h, s));
        }
        // process again?
        Source::Processed => {
            let s = std::fs::read_to_string(&src.absolute_path)?;
            return Ok(string_to_parser(h, s));
        }
    }
}
