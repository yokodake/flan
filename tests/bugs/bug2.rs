// variable and dimension names not parsed correctly

use flan::codemap::Spanned;
use flan::driver::string_to_parser;
use flan::error::{ErrorFlags, Handler};
use flan::syntax::{TermK, Terms};

#[derive(Clone, PartialEq, Debug)]
enum Names {
    /// variable
    V(String),
    /// dimension name
    D(String),
}
use Names::*;

static SRC : &str = "this is some text #$foo##$foo#a other text #dim1{#$bar/baz#some text###dim1{other branch text ## some #$foo# var}# some other text }# more text.";
fn expected() -> Vec<Names> {
    vec![
        V(String::from("foo")),
        V(String::from("foo")),
        D(String::from("dim1")),
        V(String::from("bar/baz")),
        D(String::from("dim1")),
        V(String::from("foo")),
    ]
}

#[test]
pub fn names() {
    let flags = ErrorFlags {
        no_extra: false,
        report_level: 5,
        warn_as_error: false,
    };
    let mut h = Handler::new(flags);
    let p = string_to_parser(&mut h, SRC.into());
    assert!(p.is_some());
    let tree = p.unwrap().parse();
    assert!(tree.is_ok());
    let ns = get_names(tree.unwrap());
    assert_eq!(expected(), ns);
}

fn get_names(ts: Terms) -> Vec<Names> {
    let mut v = Vec::new();
    for Spanned { node, span: _ } in ts {
        match node {
            TermK::Text => {}
            TermK::Var(n) => v.push(V(n)),
            TermK::Dimension { name, children } => {
                v.push(D(name));
                for c in children {
                    v.append(&mut get_names(c));
                }
            }
        }
    }
    v
}
