//! The Parsing module
//!
//! Aside from variable names, all text parsed is represented as a span to avoid
//! redundant memory usage (especially if they're big files).
//! The syntax:
//! ```bnf
//! Terms := Term*
//! Term  := Text
//!        | #$IDENTIFIER#
//!        | `#VARID{` Terms (`##` Terms)* `}#`
//!
//! VARID := alphanumeric+
//! IDENTIFIER := (alphanumeric | [!%&'*+-./:<=>?@_])*
//! ```
//! Variant identifiers (VARID) should be named for now.
//!
//! A whole lot of ascii symbols are accepted in identifiers, probably too much, but we can and I figured it might
//! be interresting to have variables names of paths to contain slashes for example.
// #![allow(dead_code)]
use std::collections::VecDeque;
use std::rc::Rc;

use crate::codemap::{Span, Spanned};
use crate::error::{Error, Handler};
use crate::syntax::errors::PError;
use crate::syntax::lexer::{Lexer, Token, TokenK};

/// type of a parsed expression
type Parsed<T> = Result<T, PError>;

pub struct Parser<'a> {
    // @FIXME remove mut
    handler: &'a mut Handler<PError>,
    current_token: Token,
    tokens: TokenStream,
    src: String,
}
impl Parser<'_> {
    pub fn new<'a>(input: String, h: &'a mut Handler<PError>, ts: TokenStream) -> Parser<'a> {
        Parser {
            handler: h,
            current_token: Token::default(),
            tokens: ts,
            src: input,
        }
    }

    pub fn parse_terms(&mut self) -> Parsed<Terms> {
        let mut terms = Vec::new();
        loop {
            match self.current_token.kind() {
                TokenK::Text | TokenK::Var => terms.append(&mut self.parse_alt()?),
                TokenK::Openv => terms.push(self.parse_sum()?),
                TokenK::EOF => return Ok(terms),
                k => {
                    self.handler
                        .error(
                            format!(
                                "Unexpected {}.",
                                match k {
                                    TokenK::Closev => "closing delimiter",
                                    TokenK::Sepv => "Variant branch separator",
                                    _ => unreachable!(),
                                }
                            )
                            .as_ref(),
                        )
                        .with_span(self.current_token.span)
                        .with_kind(PError::UnexpectedToken)
                        .delay();
                    return Err(PError::UnexpectedToken);
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
        while self.current_token.is_not_sum() {
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
    pub fn get_sum_name(&self) -> Option<Name> {
        let lo = self.current_token.span.lo_as_usize();
        let hi = self.current_token.span.hi_as_usize();
        if self.current_token.span.len() > 2 {
            // @SAFETY span is guaranteed to be valid by lexer
            Some(unsafe { self.src.get_unchecked(lo + 1..hi - 1) }.into())
        } else {
            None
        }
    }
    pub fn parse_sum(&mut self) -> Parsed<Term> {
        let start = self.current_token.span;
        let name = self.get_sum_name();
        self.next_token(); // eat Openv
        let mut cs = Vec::new();
        loop {
            let c = self.parse_terms()?;
            match self.current_token.kind() {
                TokenK::Closev => return Ok(Term::sum(name, cs, start + self.current_token.span)),
                TokenK::Sepv => {
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

type Name = String;
type Terms = Vec<Term>;
type Term = Spanned<TermK>;
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
    pub fn sum(n: Option<String>, cs: Vec<Terms>, s: Span) -> Term {
        Term {
            node: TermK::Sum {
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
    Sum {
        name: Option<String>,
        children: Vec<Terms>,
    },
}

type TokenStream = VecDeque<Token>;

pub fn source_to_stream(h: &mut Handler<PError>, src: &str) -> TokenStream {
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

pub fn string_to_parser<'a>(h: &'a mut Handler<PError>, str: String) -> Parser<'a> {
    let ts = source_to_stream(h, str.as_ref());
    Parser::new(str, h, ts)
}
