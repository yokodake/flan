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
use crate::codemap::{Span, Spanned};
use crate::error::Handler;
use crate::syntax::errors::PError;
use crate::syntax::lexer::{Lexer, Token, TokenK};

/// type of a parsed expression
type Parsed<T> = Result<T, PError>;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    handler: Box<Handler<PError>>,
    current_token: Token,
    src: String,
}
impl Parser<'_> {
    pub fn new<'a>(input: String, mut h: Box<Handler<PError>>) -> Parser<'a> {
        Parser {
            lexer: Lexer::new(input.as_ref(), h.as_mut()),
            handler: h,
            current_token: todo!(),
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
                _ => todo!("error"),
            };
            self.next_token();
        }
    }
    pub fn parse_var(&self) -> Parsed<Term> {
        let lo = self.current_token.span.lo_as_usize();
        let hi = self.current_token.span.hi_as_usize();
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
        self.current_token = self.lexer.next_token();
        self.current_token
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
