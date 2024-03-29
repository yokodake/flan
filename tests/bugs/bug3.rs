// variable and dimension names not parsed correctly

use flan::driver::{source_to_stream, string_to_parser};
use flan::error::{ErrorFlags, Handler};
use flan::sourcemap::SrcMap;
use flan::syntax::TokenStream;
use flan::syntax::lexer::TokenK;

use crate::utils::{Kind, get_kinds};

static SRC: &str = "begin #$var1##$var2#a txt #dim1{#dim2{#$var/var# text ###dim1{text ##txt & #$var#}### some other text}# end 1st br.##2nd br txt}# end.";
fn expected_tokens() -> Vec<TokenK> {
    use TokenK::*;
    vec![
        Text, Var, Var, Text, Opend, Opend, Var, Text, Sepd, Opend, Text, Sepd, Text, Var, Closed,
        Sepd, Text, Closed, Text, Sepd, Text, Closed, Text, EOF,
    ]
}
fn expected_terms() -> Vec<Kind> {
    use Kind::*;
    vec![
        Txt,
        Var("var1".into()),
        Var("var2".into()),
        Txt, //a txt
        Dim(
            "dim1".into(),
            vec![
                vec![
                    Dim(
                        "dim2".into(),
                        vec![
                            vec![Var("var/var".into()), Txt],
                            vec![Dim(
                                "dim1".into(),
                                vec![vec![Txt], vec![Txt, Var("var".into())]],
                            )],
                            vec![Txt], // some other text
                        ],
                    ),
                    Txt, // end 1st br.
                ],
                vec![Txt], //2nd br txt
            ],
        ),
        Txt, // end.
    ]
}

#[test]
pub fn nesting_parsing() {
    let mut h = Handler::new(ErrorFlags::default(), SrcMap::new());
    let p = string_to_parser(&mut h, SRC.into());
    assert!(p.is_some());
    let terms = p.unwrap().parse();
    assert!(terms.is_ok());
    assert_eq!(expected_terms(), get_kinds(terms.unwrap()))
}
#[test]
pub fn nesting_lexing() {
    let flags = ErrorFlags::default();
    let mut h = Handler::new(flags, SrcMap::new());
    let s = source_to_stream(&mut h, SRC);
    assert!(s.is_some());
    assert_eq!(expected_tokens(), get_tokens(s.unwrap()))
}

fn get_tokens(ts: TokenStream) -> Vec<TokenK> {
    ts.iter().map(|t| t.node).collect()
}
