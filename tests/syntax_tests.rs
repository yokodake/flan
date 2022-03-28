use flan::syntax::lexer::TokenK;

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
    let src = "foo \\#$foo# \\\\ ...";

    let actual = parse_str(src);
    assert!(actual.is_ok());
    let kinds = get_full_kinds(actual.unwrap(), src);
    let mut cnt = 0;
    for k in kinds {
        assert_eq!(k, Kind::Txt);
        if let Kind::Text(s) = k {
            cnt += if s.contains('\\') { 1 } else { 0 };
        } else {
            assert!(false, "{:?}", k);
        }
    }
    assert!(cnt == 1);
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
        Token::new_lit(Text, 0, 10),
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
        Token::new_lit(Var, 0, src.len()),
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
        Token::new_lit(Opend, 0, src.len()),
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
        Token::new_lit(Closed, 0, 2),
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
        Token::new_lit(Opend, 0, 3),
        Token::new_lit(Sepd, 3, 5),
        Token::new_lit(Closed, 5, 7),
        Token::new_lit(EOF, 7, src.len()),
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
        Token::new(Text, 0, 4),
        // '\' is ignored span(4,5)
        Token::new(Text, 5, 6),
        Token::new(EOF, 6, src.len()),
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
        Token::new(Text, 0, 1),
        Token::new(Var, 1, 7),
        Token::new(Text, 7, 8),
        Token::new(EOF, 8, src.len()),
    ];
    assert_eq!(expected, toks);
}
#[test]
fn multi_dim() {
    use flan::syntax::lexer::Token;
    use TokenK::*;
    let src = "#x{foo##bar}##y{hello##world}#";
    let toks = stream_str(src);
    let expected = vec![
        Token::new(Opend,  0,  3),
        Token::new(Text,   3,  6),
        Token::new(Sepd,   6,  8),
        Token::new(Text,   8,  11),
        Token::new(Closed, 11, 13),
        Token::new(Opend,  13, 16),
        Token::new(Text,   16, 21),
        Token::new(Sepd,   21, 23),
        Token::new(Text,   23, 28),
        Token::new(Closed, 28, 30),
        Token::new(EOF, 30, 30),
    ];
    dbg!(get_full_kinds(parse_str(src).unwrap(), src));
    assert_eq!(expected, toks);
}

#[test]
/// write_tests.rs::big_dim_txt failure
fn regtest_big_dim_txt() {
    let src = "#dim2{hello, world ## ignored ##hello, #$name#}# from 2hu"; 
    let expected = vec![kdim("dim2", 
                                    vec![vec![ktext("hello, world ")], 
                                         vec![ktext(" ignored ")],
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
    let src = r#"good morning, #$name# \o \#ItBack"#;
    let expected = 
        vec![ktext("good morning, "),
             kvar("name"),
             ktext(" "),
             ktext(r#"\o "#),
             ktext(r#"#ItBack"#),
            ];
    let r_ts = parse_str(src);
    assert!(r_ts.is_ok());
    let ts = r_ts.unwrap();
    assert_eq!(expected, get_full_kinds(ts, src));
}