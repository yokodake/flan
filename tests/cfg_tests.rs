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
