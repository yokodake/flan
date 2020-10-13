use std::collections::HashMap;
use std::io::{BufRead, Cursor, Read, Write};
use std::iter::FromIterator;
use std::sync::Arc;

use flan::driver::*;
use flan::env::{Dim, Env};
use flan::error::{ErrorFlags, Handler};
use flan::sourcemap::SrcMap;
use flan::syntax::{Term, TermK};
use flan::utils::RelativeSeek;

macro_rules! mock_env {
    () => {
        Env::new(
            HashMap::from_iter(vec![
                ("var1".into(), "val1".into()),
                ("name".into(), "flan".into()),
            ]),
            HashMap::from_iter(vec![
                ("dim0".into(), Dim::new(0)),
                ("dim2".into(), Dim::new(2)),
            ]),
            &mut Handler::new(ErrorFlags::default(), SrcMap::new()),
        )
    };
}
macro_rules! cursors {
    ($e:expr) => {
        (Cursor::new($e.as_bytes()), Cursor::new(vec![]))
    };
}

#[test]
fn txt_term() {
    let (mut from, mut to) = cursors!("foobar");
    let term = Term::new(TermK::Text, 0, 5);
    assert!(write_term(&term, &mut from, &mut to, 0, &mock_env!()).is_ok());
    let actual = std::str::from_utf8(to.get_ref()).unwrap();
    assert_eq!("foobar", actual);
}
#[test]
fn txt_terms() {
    let (mut from, mut to) = cursors!("foobar");
    let terms = vec![Term::new(TermK::Text, 0, 2), Term::new(TermK::Text, 3, 5)];
    assert!(write_terms(&terms, &mut from, &mut to, 0, &mock_env!()).is_ok());
    let actual = std::str::from_utf8(to.get_ref()).unwrap();
    assert_eq!("foobar", actual);
}
#[test]
fn skip_txt() {
    let (mut from, mut to) = cursors!("hello, world");
    let terms = vec![Term::new(TermK::Text, 0, 4), Term::new(TermK::Text, 6, 11)];
    assert!(write_terms(&terms, &mut from, &mut to, 0, &mock_env!()).is_ok());
    let actual = std::str::from_utf8(to.get_ref()).unwrap();
    assert_eq!("hello world", actual);
}
#[test]
fn var_term() {
    let (mut from, mut to) = cursors!("#$var1#");
    let term = Term::new(TermK::Var("var1".into()), 0, 7);
    assert!(write_term(&term, &mut from, &mut to, 0, &mock_env!()).is_ok());
    let actual = std::str::from_utf8(to.get_ref()).unwrap();
    assert_eq!("val1", actual);
}
#[test]
fn var_terms() {
    let (mut from, mut to) = cursors!("#$var1##$name#");
    let terms = vec![
        Term::new(TermK::Var("var1".into()), 0, 7),
        Term::new(TermK::Var("name".into()), 8, 13),
    ];
    assert!(write_terms(&terms, &mut from, &mut to, 0, &mock_env!()).is_ok());
    let actual = std::str::from_utf8(to.get_ref()).unwrap();
    assert_eq!("val1flan", actual);
}
#[test]
fn simple_dim() {
    let (mut from, mut to) = cursors!("#dim0{hello, world}#");
    let term = Term::new(
        TermK::Dimension {
            name: "dim0".into(),
            children: vec![vec![Term::new(TermK::Text, 6, 17)]],
        },
        0,
        19,
    );
    assert!(write_term(&term, &mut from, &mut to, 0, &mock_env!()).is_ok());
    let actual = std::str::from_utf8(to.get_ref()).unwrap();
    assert_eq!("hello, world", actual);
}
#[test]
fn bigger_dim() {
    let (mut from, mut to) = cursors!("#dim2{hello, world ## ignored ##hello, #$name#}#");
    let term = Term::new(
        TermK::Dimension {
            name: "dim2".into(),
            children: vec![
                vec![],
                vec![],
                vec![
                    Term::new(TermK::Text, 32, 38),
                    Term::new(TermK::Var("name".into()), 39, 45),
                ],
            ],
        },
        0,
        47,
    );
    assert!(write_term(&term, &mut from, &mut to, 0, &mock_env!()).is_ok());
    let actual = std::str::from_utf8(to.get_ref()).unwrap();
    assert_eq!("hello, flan", actual);
}
#[test]
fn big_dim_txt() {
    let (mut from, mut to) = cursors!("#dim2{hello, world ## ignored ##hello, #$name#}# from 2hu");
    let terms = vec![
        Term::new(
            TermK::Dimension {
                name: "dim2".into(),
                children: vec![
                    vec![],
                    vec![],
                    vec![
                        Term::new(TermK::Text, 32, 38),
                        Term::new(TermK::Var("name".into()), 39, 45),
                    ],
                ],
            },
            0,
            47,
        ),
        Term::new(TermK::Text, 48, 56),
    ];
    assert!(write_terms(&terms, &mut from, &mut to, 0, &mock_env!()).is_ok());
    let actual = std::str::from_utf8(to.get_ref()).unwrap();
    assert_eq!("hello, flan from 2hu", actual);
}
