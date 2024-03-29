use std::collections::HashMap;
use std::iter::FromIterator;

use flan::env::{Dim, Env};
use flan::error::{ErrorFlags, Handler};
use flan::sourcemap::SrcMap;

mod utils;
use utils::{write_str, write_terms};

macro_rules! mock_env {
    () => {
        Env::new(
            HashMap::from_iter(vec![
                ("var1".into(), "val1".into()),
                ("name".into(), "flan".into()),
            ]),
            HashMap::from_iter(vec![
                ("dim0".into(), Dim::new(0)),
                ("dim1".into(), Dim::new(0)),
                ("dim2".into(), Dim::new(2)),
            ]),
            Handler::new(ErrorFlags::default(), SrcMap::new()),
        )
    };
}

#[test]
fn txt_term() {
    let src = "foobar";
    let actual = write_str(src, &mock_env!());
    assert_eq!("foobar", actual);
}
#[test]
fn txt_terms() {
    let src = "foobar";
    let actual = write_str(src, &mock_env!());
    assert_eq!("foobar", actual);
}
#[test]
fn skip_txt() {
    use flan::syntax::{Term, TermK};
    use std::io::Cursor;

    let src = "hello, world!";
    let (mut from, mut to) = (Cursor::new(src.as_bytes()), Cursor::new(Vec::new()));
    let terms = vec![Term::new(TermK::Text, 0, 5), Term::new(TermK::Text, 6, 12)];
    assert!(write_terms(&mut from, 0usize, &mut to, &mock_env!(), &terms).is_ok());
    let actual = std::str::from_utf8(to.get_ref()).unwrap();
    assert_eq!("hello world", actual);
}
#[test]
fn var_term() {
    let src = "#$var1#";
    let actual = write_str(src, &mock_env!());
    assert_eq!("val1", actual);
}
#[test]
fn var_terms() {
    let src = "#$var1##$name#";
    let actual = write_str(src, &mock_env!());
    assert_eq!("val1flan", actual);
}
#[test]
fn simple_dim() {
    let src = "#dim0{hello, world}#";
    let actual = write_str(src, &mock_env!());
    assert_eq!("hello, world", actual);
}
#[test]
fn bigger_dim() {
    let src = "#dim2{hello, world ## ignored ##hello, #$name#}#";
    let actual = write_str(src, &mock_env!());
    assert_eq!("hello, flan", actual);
}
#[test]
fn big_dim_txt() {
    let src = "#dim2{hello, world ## ignored ##hello, #$name#}# from 2hu";
    let actual = write_str(src, &mock_env!());
    assert_eq!("hello, flan from 2hu", actual);
}
#[test]
fn multi_dim() {
    let src = "#dim1{yahallo##hello}#, #dim1{flan##remi}#";
    let actual = write_str(src, &mock_env!());
    assert_eq!("yahallo, flan", actual);
}
#[test]
fn nested_dim() {
    let src = "#dim1{#dim0{yahallo##hello}###byebye!}#, #dim1{flan##remi}#";
    let actual = write_str(src, &mock_env!());
    assert_eq!("yahallo, flan", actual);
}

#[test]
fn escapes() {
    let src = r#"good morning, #$name# \\o \#ItBack"#;
    let expected = r#"good morning, flan \o #ItBack"#;
    let actual = write_str(src, &mock_env!());
    assert_eq!(expected, actual);
}

#[test]
fn escape_end() {
    let src = r#"\# \#"#;
    let expected = "# #";
    let actual = write_str(src, &mock_env!());
    assert_eq!(expected, actual);
    let src = r#"\\ \\"#;
    let expected = r#"\ \"#;
    let actual = write_str(src, &mock_env!());
    assert_eq!(expected, actual);
    let src = r#"\} \}"#;
    let expected = "} }";
    let actual = write_str(src, &mock_env!());
    assert_eq!(expected, actual);
}
