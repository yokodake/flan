use flan::cfg::*;
use flan::opt_parse::{Index, OptDec};

#[test]
fn invalid_size_choice() {
    assert!(!Choices::Size(128).valid());
}

#[test]
fn valid_size_choice() {
    assert!(Choices::Size(0).valid());
    assert!(Choices::Size(127).valid());
}

#[test]
fn dups_decisions() {
    let xs = vec!["foo".into(), "bar".into(), "foo".into(), "baz".into()];
    assert!(!Choices::Names(xs).valid());
    let xs = vec!["foo".into(), "bar".into(), "baz".into()];
    assert!(Choices::Names(xs).valid());
}

#[test]
fn invalid_id() {
    assert!(OptDec::parse_decision(&"6ajaofjo").is_err());
    assert!(OptDec::parse_decision(&"+ajaofjo").is_err());
}

#[test]
fn valid_id() {
    let expected = OptDec::Name("_aja3791o_fjo8319".into());
    let actual = opt_parse::OptDec::parse_decision(&"_aja3791o_fjo8319");
    assert!(actual.is_ok());
    assert_eq!(expected, actual.unwrap());
}

#[test]
fn valid_dim() {
    let expected = OptDec::WithDim("foo".into(), Index::Num(0));
    let actual = OptDec::parse_decision(&"foo=0");
    assert!(actual.is_ok());
    assert_eq!(expected, actual.unwrap());

    let expected = OptDec::WithDim("foo".into(), Index::Name("bar".into()));
    let actual = OptDec::parse_decision(&"foo=bar");
    assert!(actual.is_ok());
    assert_eq!(expected, actual.unwrap());
}
