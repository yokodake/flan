//! The Parsing module
//!
//! Aside from variable names, all text parsed is represented as a span to avoid
//! redundant memory usage.
//! The syntax:
//! ```bnf
//! Terms := Term*
//! Term  :=  #$IDENTIFIER#                      // variables
//!        | `#DIMID{` Terms (`##` Terms)* `}#`  // Dimensions
//!        |  Text                               // anything else
//!
//! DIMID := (alpha | `_`)(alphanumeric | `_`)*
//! IDENTIFIER := (alphanumeric | [!%&'*+-./:<=>?@_])+
//! ```
//!
//! A whole lot of ascii symbols are accepted in identifiers, probably too much, but we can and I figured it might
//! be interresting to have variables names of paths to contain slashes for example.
use std::collections::VecDeque;

use crate::error::Handler;
use crate::sourcemap::{Pos, Span, Spanned};
use crate::syntax::lexer::{Token, TokenK};
use crate::syntax::Error;

/// type of a parsed expression
pub type Parsed<T> = Result<T, Error>;

pub struct Parser<'a> {
    // @FIXME can we remove mut
    pub handler: &'a mut Handler,
        current_token: Token,
    pub tokens: TokenStream,
    /// needed?
    pub src: String,
    /// unmatched open delimiters
    /// @FIXME make it a context
        nest: u8,
    /// absolute position in source map
    pub offset: Pos,
    /// current dimension we're parsing (for domination)  
        ctx : Ctx
}
impl Parser<'_> {
    pub fn new<'a>(h: &'a mut Handler, input: String, ts: TokenStream, offset: Pos) -> Parser<'a> {
        let mut p = Parser {
            handler: h,
            current_token: Token::default(),
            tokens: ts,
            src: input,
            nest: 0,
            offset,
            ctx: Ctx::default()
        };
        p.next_token();
        p
    }
    /// entry function for new parser
    pub fn parse(&mut self) -> Parsed<Terms> {
        self.parse_terms().and_then(|ts| {
            if self.handler.err_count > 0 {
                // @TODO could be improved
                // valid parse tree but errors => non fatal lexing errors
                Err(Error::LexerError)
            } else {
                Ok(ts)
            }
        })
    }
    /// parse multiple Terms
    pub fn parse_terms(&mut self) -> Parsed<Terms> {
        let mut terms = Vec::new();
        loop {
            match self.current_token.kind() {
                TokenK::Text => terms.push(self.parse_txt()?),
                TokenK::Var => terms.push(self.parse_var()?),
                TokenK::Opend => {
                    self.nest += 1;
                    let t = self.parse_dim()?;
                    terms.push(t);
                }
                k @ TokenK::Closed | k @ TokenK::Sepd => {
                    if self.nest == 0 {
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
                            .delay();
                        return Err(Error::UnexpectedToken);
                    } else if k == TokenK::Closed {
                        self.nest -= 1;
                    }
                    // return all the terms so far
                    return Ok(terms);
                }
                TokenK::EOF => return Ok(terms),
            };
            self.next_token();
        }
    }
    pub fn parse_var(&self) -> Parsed<Term> {
        let lo = self.src_idx(self.current_token.span.lo);
        let hi = self.src_idx(self.current_token.span.hi);
        // @SAFETY: span is guaranteed to be valid by lexer
        let name = unsafe { self.src.get_unchecked(lo + 2..hi) };
        Ok(Term::var(name.into(), self.current_token.span))
    }
    pub fn parse_txt(&self) -> Parsed<Term> {
        Ok(Term::text(self.current_token.span))
    }
    /// parse a sequence of texts and variables
    pub fn parse_alt(&mut self) -> Parsed<Terms> {
        let mut xs = Vec::new();
        while !self.current_token.is_dimension_or_eof() {
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
    /// extract the name of the dimension form the [`Self::current_token`]
    pub fn get_dim_name(&self) -> Name {
        let lo = self.src_idx(self.current_token.span.lo);
        let hi = self.src_idx(self.current_token.span.hi);
        // @TODO use get_unchecked instead?
        match self.src.get(lo + 1..hi).map(String::from) {
            Some(s) => s,
            None => unreachable!(), // lexer should've failed
        }
    }
    pub fn parse_dim(&mut self) -> Parsed<Term> {
        let start = self.current_token.span;
        let name = self.get_dim_name();
        self.next_token(); // eat Opend
        self.ctx.enter(name.clone()); // enter a new scope
        let mut cs : Vec<Terms> = Vec::new();
        loop {
            let c : Terms = self.parse_terms()?;
            match self.current_token.kind() {
                TokenK::Closed => {
                    cs.push(c);
                    self.ctx.exit(&name);
                    match self.ctx.find(&name) { 
                        None => return Ok(Term::dim(name, cs, start + self.current_token.span)),
                        // perform domination
                        Some(Scope{child,..}) => return Ok(cs.get(child).expect("conflicting child count")),
                    }
                }
                TokenK::Sepd => {
                    cs.push(c);
                    self.next_token(); // eat Sepd
                    self.ctx.next_child(); // change child in dimension
                    continue;
                }
                TokenK::EOF => {
                    self.handler
                        .error("Unclosed dimension delimiter. Expected `}#`.")
                        .with_span(start)
                        .at_span("dimension starts here")
                        .delay();
                    return Err(Error::UnclosedDelimiter);
                }
                _ => unreachable!(),
            }
        }
    }

    fn next_token(&mut self) -> Token {
        self.current_token = match self.tokens.pop_front() {
            Some(t) => t,
            None => Token::new(TokenK::EOF, self.src.len(), self.src.len()),
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
    /// a source_map relative position to index in the source
    fn src_idx(&self, p: Pos) -> usize {
        (p - self.offset).as_usize()
    }
}

/// a Variable or Dimension name.
pub type Name = String;
/// a list of [`Terms`]
pub type Terms = Vec<Term>;
/// a Spanned [`TermK`]
pub type Term = Spanned<TermK>;
impl Term {
    pub fn text(span: Span) -> Term {
        Term {
            node: TermK::Text,
            span,
        }
    }
    pub fn var(name: Name, span: Span) -> Term {
        Term {
            node: TermK::Var(name),
            span,
        }
    }
    pub fn dim(name: Name, children: Vec<Terms>, span: Span) -> Term {
        Term {
            node: TermK::Dimension { name, children },
            span,
        }
    }
    /// returns the span of only the name of a variable or dimension
    /// ```c++
    /// #$foobar#   #dimension{
    ///   ^^^^^^     ^^^^^^^^^
    /// ```
    pub fn name_span(&self) -> Option<Span> {
        match &self.node {
            TermK::Text => None,
            TermK::Var(name) => {
                let s = self.span.subspan(2, name.len() as u64 - 1);
                assert_eq!(s.len(), name.len());
                Some(s)
            }
            TermK::Dimension { name, .. } => {
                let s = self.span.subspan(1, name.len());
                Some(s)
            }
        }
    }
    pub fn opend_span(&self) -> Option<Span> {
        match &self.node {
            TermK::Dimension { name, .. } => {
                let s = self.span.subspan(0, name.len() + 1);
                Some(s)
            }
            _ => None,
        }
    }
}
/// the kind of a Term
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum TermK {
    Text,
    Var(Name),
    Dimension { name: String, children: Vec<Terms> },
}

pub type TokenStream = VecDeque<Token>;

/// @SPEED this will incur extra string copies and comparisons... 
///        to fix copies we need a form of Arena, as the String will be owned by Term too
///        (Since the caller of `parse` could drop as soon as it returns the Term)
///        to fix comparisons a symbol table could be used
///        ...the symbol table could use the arena to fix both
struct Scope {
    dim  : String,
    child: u8,
}
#[derive(Default)]
struct Ctx(VecDeque<Scope>);
impl AsRef<VecDeque<Scope>> for Ctx {
    fn as_ref(&self) -> &VecDeque<Scope> {
        &self.0
    }
}
impl Ctx {
    fn push(&mut self, scope: Scope) {
        self.0.push_front(scope);
    }
    fn pop(&mut self) -> Option<Scope> {
        self.0.pop_front()
    }
    /// enter a new scope
    fn enter(&mut self, dim: String) {
        self.push(Scope{dim, child: 0})
    }
    /// bump the child counter
    fn next_child(&mut self) -> bool {
        match self.0.front_mut() { 
            None => false,
            Some(Scope{child, ..}) => {
                *child += 1;
                true
            },
        }
    }
    /// exit the current scope
    fn exit(&mut self, name: &str) {
        let n = self.pop().expect("expected non-empty Ctx");
        assert!(name == n.dim);
    }
}
