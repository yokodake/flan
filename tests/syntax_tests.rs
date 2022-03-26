use TokenK::Opend;
use flan::error::{ErrorFlags, Handler};
use flan::sourcemap::SrcMap;
use flan::syntax::lexer::{Token, TokenK};

mod utils;
use utils::*;

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
fn multi_escapes() {
    use flan::driver::source_to_stream;
    use TokenK::*;
    let src = "foo \\#$foo# \\\\ ...";

    let mut h = Handler::new(ErrorFlags::default(), SrcMap::new());
    let actual = source_to_stream(&mut h, src);
    assert!(actual.is_some());
    let actual: Vec<Token> = actual.unwrap().into_iter().collect();
    let expected = vec![
        Token::new(Text, 0, 3),
        Token::new(Text, 5, 11),
        Token::new(Text, 13, 17),
        Token::new(EOF, 18, 18),
    ];
    assert_eq!(expected, actual);
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
        ktxt(),
        kvar("_var1"),
        ktxt(),
        kvar("_2"),
        ktxt(),
        kdim("dim", vec![vec![kvar("inside")], vec![Txt]]),
        ktxt(),
        kvar("last_var"),
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
    let src = "#$foo#";
    let r_ts = parse_str(src);
    assert!(r_ts.is_ok());
    let ts = r_ts.unwrap();
    let expected = vec![kvar("foo")];
    assert_eq!(expected, get_kinds(ts));
}
#[test]
fn empty_choices() {
    let src = "#foo{##}#";
    let r_ts = parse_str(src);
    assert!(r_ts.is_ok());
    let ts = r_ts.unwrap();
    let expected = vec![kdim("foo", vec![vec![], vec![]])];
    assert_eq!(expected, get_kinds(ts));
}

#[test]
fn one_empty_choice() {
    let src = "#foo{}#";
    let r_ts = parse_str(src);
    assert!(r_ts.is_ok());
    let ts = r_ts.unwrap();
    let expected = vec![kdim("foo", vec![vec![]])];
    assert_eq!(expected, get_kinds(ts));
}

#[test]
fn one_txt_span() {
    use flan::syntax::lexer::Token;
    use TokenK::*;
    let src = "0123456789";
    let toks = stream_str(src);
    let expected = vec![
        Token::new_lit(Text, 0, 9),
        Token::new_lit(EOF, src.len(), src.len()),
    ];
    assert_eq!(expected, toks);
}
#[test]
fn one_var_span() {
    use flan::syntax::lexer::Token;
    use TokenK::*;
    let src = "#$var#";
    let toks = stream_str(src);
    let expected = vec![
        Token::new_lit(Var, 0, src.len() - 1),
        Token::new_lit(EOF, src.len(), src.len()),
    ];
    assert_eq!(expected, toks);
}
#[test]
fn one_opend_span() {
    use flan::syntax::lexer::Token;
    use TokenK::*;
    let src = "#foo{";
    let toks = stream_str(src);
    let expected = vec![
        Token::new_lit(Opend, 0, src.len() - 1),
        Token::new_lit(EOF, src.len(), src.len()),
    ];
    assert_eq!(expected, toks);
}
#[test]
fn one_closed_span() {
    use flan::syntax::lexer::Token;
    use TokenK::*;
    let src = "}#";
    let toks = stream_str(src);
    let expected = vec![
        Token::new_lit(Closed, 0, 1),
        Token::new_lit(EOF, src.len(), src.len()),
    ];
    assert_eq!(expected, toks);
}
#[test]
fn one_sepd_span() {
    use flan::syntax::lexer::Token;
    use TokenK::*;
    let src = "#_{##}#";
    let toks = stream_str(src);
    let expected = vec![
        Token::new_lit(Opend, 0, 2),
        Token::new_lit(Sepd, 3, 4),
        Token::new_lit(Closed, 5, 6),
        Token::new_lit(EOF, src.len(), src.len()),
    ];
    assert_eq!(expected, toks);
}

#[test]
fn escape_at_end() {
    use flan::syntax::lexer::Token;
    use TokenK::*;
    let src = r#"{...\}"#;
    let toks = stream_str(src);
    let expected = vec![
        Token::new(Text, 0, 3),
        Token::new(Text, 5, 5),
        Token::new(EOF, src.len(), src.len()),
    ];
    assert_eq!(expected, toks);
}
#[test]
fn one_char_txt() {
    use flan::syntax::lexer::Token;
    use TokenK::*;
    let src = ".#$foo#.";
    let toks = stream_str(src);
    let expected = vec![
        Token::new(Text, 0, 0),
        Token::new(Var, 1, 6),
        Token::new(Text, 7, 7),
        Token::new(EOF, src.len(), src.len()),
    ];
    assert_eq!(expected, toks);
}

#[test]
/// write_tests.rs::big_dim_txt failure
fn regtest_big_dim_txt() {
    let src = "#dim2{hello, world ## ignored ##hello, #$name#}# from 2hu"; 
    let expected = vec![
        Token::new(Opend, 0, 0)
        ];
    let expected = vec![kdim("dim2", 
                                    vec![vec![ktext("hello, world")], 
                                         vec![ktext("ignored")],
                                         vec![ktext("hello, "), kvar("name")]
                                    ])
                                 ,ktext(" from 2hu")];
    let r_ts = parse_str(src);
    assert!(r_ts.is_ok(), "parsing failed");
    let ts = r_ts.unwrap();
    assert_eq!(expected, get_full_kinds(ts, src));
}
#[test]
/// write_tests.rs::escapes failure
fn regtest_escapes() {
    let src = "good morning, #$name# \\\\o \\#ItBack";
    let expected = vec![ktext("good morning, "),
                                  kvar("name"),
                                  ktext("\\o #Itback"),
                                 ];
    let r_ts = parse_str(src);
    assert!(r_ts.is_ok());
    let ts = r_ts.unwrap();
    assert_eq!(expected, get_full_kinds(ts, src));
}