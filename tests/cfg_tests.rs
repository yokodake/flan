use flan::cfg::*;

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
    assert!(Decision::from_str(&"6ajaofjo").is_err());
    assert!(Decision::from_str(&"+ajaofjo").is_err());
}

#[test]
fn valid_id() {
    let expected = Decision::Name("_aja3791o_fjo8319".into());
    let actual = Decision::from_str(&"_aja3791o_fjo8319");
    assert!(actual.is_ok());
    assert_eq!(expected, actual.unwrap());
}

#[test]
fn valid_dim() {
    let expected = Decision::WithDim("foo".into(), Index::Num(0));
    let actual = Decision::from_str(&"foo=0");
    assert!(actual.is_ok());
    assert_eq!(expected, actual.unwrap());

    let expected = Decision::WithDim("foo".into(), Index::Name("bar".into()));
    let actual = Decision::from_str(&"foo=bar");
    assert!(actual.is_ok());
    assert_eq!(expected, actual.unwrap());
}
