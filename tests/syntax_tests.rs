use flan::codemap::Spanned;
use flan::error::{ErrorFlags, Handler};
use flan::syntax::lexer::TokenK;
use flan::syntax::{Parsed, TermK, Terms};

type Kinds = Vec<Kind>;
#[derive(Clone, PartialEq, Debug)]
enum Kind {
    /// Text
    Txt,
    /// variable
    Var(String),
    /// dimension name
    Dim(String, Vec<Kinds>),
}
fn get_kinds(ts: Terms) -> Kinds {
    use Kind::*;
    let mut v = Vec::new();
    for Spanned { node, span: _ } in ts {
        match node {
            TermK::Text => v.push(Txt),
            TermK::Var(n) => v.push(Var(n)),
            TermK::Dimension { name, children } => {
                let mut cs = Vec::new();
                for c in children {
                    cs.push(get_kinds(c));
                }
                v.push(Dim(name, cs))
            }
        }
    }
    v
}
fn parse_str(src: &str) -> Parsed<Terms> {
    use flan::driver::string_to_parser;
    let mut h = Handler::new(ErrorFlags::default());
    let p = string_to_parser(&mut h, src.into());
    assert!(p.is_some());
    p.unwrap().parse()
}
fn lex_str(src: &str) -> Vec<TokenK> {
    use flan::driver::source_to_stream;
    let mut h = Handler::new(ErrorFlags::default());
    let s = source_to_stream(&mut h, src);
    assert!(s.is_some());
    s.unwrap().iter().map(|t| t.node).collect()
}

#[test]
fn unnested_seps() {
    use TokenK::*;
    let src = "foo ## bar ## baz";
    let mut ts = lex_str(src);
    assert_eq!(EOF, ts.remove(ts.len() - 1));
    assert!(ts.iter().all(|k| k == &Text));
}
#[test]
fn escaped_seps() {
    use TokenK::*;
    let src = "foo \\## bar \\## baz";
    let mut ts = lex_str(src);
    assert_eq!(EOF, ts.remove(ts.len() - 1));
    assert!(ts.iter().all(|k| k == &Text));
}
#[test]
fn escaped_delims() {
    use TokenK::*;
    let src = "foo \\#foo{ one ## two \\}# baz";
    let mut ts = lex_str(src);
    assert_eq!(EOF, ts.remove(ts.len() - 1));
    assert!(ts.iter().all(|k| k == &Text));
}
#[test]
fn escaped_vars() {
    use TokenK::*;
    let src = "foo \\#$foo# \\#$ \\#$non terminated var";
    let mut ts = lex_str(src);
    assert_eq!(EOF, ts.remove(ts.len() - 1));
    assert!(ts.iter().all(|k| k == &Text));
}
