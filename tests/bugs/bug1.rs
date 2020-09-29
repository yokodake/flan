// missing text

use flan::driver::source_to_stream;
use flan::error::{ErrorFlags, Handler};
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
    let flags = ErrorFlags {
        no_extra: false,
        report_level: 5,
        warn_as_error: false,
    };
    let mut h = Handler::new(flags);
    let s = source_to_stream(&mut h, SRC);
    assert!(s.is_some());
    assert_eq!(expected(), get_kinds(s.unwrap()))
}

fn get_kinds(ts: TokenStream) -> Vec<TokenK> {
    ts.iter().map(|t| t.node).collect()
}
