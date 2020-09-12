use crate::codemap::{Span, Spanned};
use crate::error::{Error, Handler};
use crate::syntax::lexer::{Lexer, Token, TokenK};

/// type of a parsed expression
type Parsed<T> = Result<T, PError>;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    handler: Box<Handler>,
    current_token: Token,
    src: String,
}
impl Parser<'_> {
    pub fn parse(&mut self) -> Parsed<Term> {
        // @FIXME -> Parsed<Terms>
        match self.current_token.kind() {
            Text => self.parse_txt(),
            Var => self.parse_var(),
            Openv => self.parse_sum(),
            EOF => todo!("end"),
            _ => todo!("error"),
        }
    }
    pub fn parse_var(&self) -> Parsed<Term> {
        let lo = self.current_token.span.lo_as_usize();
        let hi = self.current_token.span.hi_as_usize();
        let name = unsafe { self.src.get_unchecked(lo + 2..hi - 2) };
        Ok(Term::Var(name.into()))
    }

    pub fn parse_txt(&self) -> Parsed<Term> {
        Ok(Term::Text(self.current_token.span))
    }
    pub fn parse_alt(&self) -> Parsed<Terms> {
        todo!();
    }

    pub fn parse_sum(&mut self) -> Parsed<Term> {
        if !self.current_token.is(TokenK::Openv) {
            todo!("error");
        }
        self.next_token(); // openv
        let mut cs = Vec::new();
        loop {
            let c = self.parse_alt();
            match self.current_token.kind() {
                TokenK::Closev => return Ok(Term::Sum { children: cs }),
                TokenK::Sepv => {
                    cs.push(c.unwrap());
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
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Term {
    Text(Span),
    Var(Name),
    Sum { children: Vec<Terms> },
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Debug, Hash)]
pub struct PError {
    error: Option<Error>,
    kind: PErrorKind,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum PErrorKind {}
