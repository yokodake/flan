// missing text

use flan::driver::source_to_stream;
use flan::error::{ErrorFlags, Handler};
use flan::sourcemap::SrcMap;
use flan::syntax::lexer::TokenK;
use flan::syntax::TokenStream;

use TokenK::*;

static SRC : &str = "this is some text #$foo##$foo#a other text #dim1{#$bar/baz#some text## some other text }# more text.";
fn expected() -> Vec<TokenK> {
    vec![
        Text, Var, Var, Text, Opend, Var, Text, Sepd, Text, Closed, Text, EOF,
    ]
}

#[test]
pub fn missing_txt() {
    let mut h = Handler::new(ErrorFlags::default(), SrcMap::new());
    let s = source_to_stream(&mut h, SRC);
    assert!(s.is_some());
    assert_eq!(expected(), get_kinds(s.unwrap()))
}

fn get_kinds(ts: TokenStream) -> Vec<TokenK> {
    ts.iter().map(|t| t.node).collect()
}
