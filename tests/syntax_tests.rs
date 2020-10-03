use flan::error::{ErrorFlags, Handler};
use flan::sourcemap::Spanned;
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
#[test]
fn lex_vars() {
    use TokenK::*;
    let src = "some text #$_var1# #$_2# #dim{#$inside### more text }# another #$last_var#";
    let tokens = lex_str(src);
    let expected = vec![
        Text, Var, Text, Var, Text, Opend, Var, Sepd, Text, Closed, Text, Var, EOF,
    ];
    assert_eq!(expected, tokens);
}
#[test]
fn parse_vars() {
    use Kind::*;
    let src = "some text #$_var1# #$_2# #dim{#$inside### more text }# another #$last_var#";
    let r_ts = parse_str(src);
    assert!(r_ts.is_ok());
    let ts = r_ts.unwrap();
    let expected = vec![
        Txt,
        Var("_var1".into()),
        Txt,
        Var("_2".into()),
        Txt,
        Dim("dim".into(), vec![vec![Var("inside".into())], vec![Txt]]),
        Txt,
        Var("last_var".into()),
    ];
    assert_eq!(expected, get_kinds(ts));
}
#[test]
fn nothing() {
    let src = "";
    {
        use TokenK::*;
        let tokens = lex_str(src);
        let expected: Vec<TokenK> = vec![EOF];
        assert_eq!(expected, tokens);
    }
    {
        let terms = parse_str(src);
        assert!(terms.is_ok());
        let expected: Vec<Kind> = vec![];
        assert_eq!(expected, get_kinds(terms.unwrap()));
    }
}
#[test]
fn one_var() {
    use Kind::*;
    let src = "#$foo#";
    let r_ts = parse_str(src);
    assert!(r_ts.is_ok());
    let ts = r_ts.unwrap();
    let expected = vec![Var("foo".into())];
    assert_eq!(expected, get_kinds(ts));
}
#[test]
fn empty_choices() {
    use Kind::*;
    let src = "#foo{##}#";
    let r_ts = parse_str(src);
    assert!(r_ts.is_ok());
    let ts = r_ts.unwrap();
    let expected = vec![Dim("foo".into(), vec![vec![], vec![]])];
    assert_eq!(expected, get_kinds(ts));
}

#[test]
fn one_empty_choice() {
    use Kind::*;
    let src = "#foo{}#";
    let r_ts = parse_str(src);
    assert!(r_ts.is_ok());
    let ts = r_ts.unwrap();
    let expected = vec![Dim("foo".into(), vec![vec![]])];
    assert_eq!(expected, get_kinds(ts));
}
