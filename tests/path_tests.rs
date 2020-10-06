use flan::utils::path::normalize_path;
use std::env::current_dir;
use std::path::{Path, PathBuf};

macro_rules! test_dir {
    () => {
        test_dir("")
    };
    ($path:expr) => {
        test_dir($path)
    };
}
fn test_dir<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut p = PathBuf::from(file!());
    p.set_extension(""); // remove '.rs'
    p.push(path); // make it a dir (adds '/')
    p
}

#[test]
pub fn exists() {
    let p = test_dir!("a/b/c/d/e/f/");
    assert!(test_dir!().is_dir());
    assert!(p.is_dir());
}
#[cfg(target_os = "linux")]
#[test]
pub fn symlink_exists() {
    assert!(test_dir!("sym-a-b/c/d/e/sym-d/e/f/").is_dir());
    assert!(!test_dir!("sym-a-b/a/b/c/d/e/sym-d/e/f/").is_dir());
}

#[test]
pub fn dir_current() {
    let expected = test_dir!("a/b/c/d/");
    let actual = normalize_path(test_dir!("a/././b/./c/./d/././."));
    assert!(actual.is_ok());
    assert_eq!(expected, actual.unwrap());
}
#[test]
pub fn dir_parent() {
    let expected = test_dir!("a/b/c/d/");
    let actual = normalize_path(test_dir!("a/../a/b/c/../../b/c/../c/d/"));
    assert!(actual.is_ok());
    assert_eq!(expected, actual.unwrap());
}

#[test]
pub fn parent_of_relative() {
    let mut expected = test_dir!();
    expected.pop();
    expected.push("../tests/path_tests/a/");
    let actual = normalize_path(test_dir!(
        "a/b/../../../path_tests/../../tests/path_tests/a/"
    ));
    assert!(actual.is_ok());
    assert_eq!(expected, actual.unwrap());
}

#[test]
pub fn leading_current() {
    let expected = test_dir!();
    let actual = {
        let mut p = PathBuf::from("././");
        p.push(test_dir!());
        normalize_path(p)
    };
    assert!(actual.is_ok());
    assert_eq!(expected, actual.unwrap())
}

#[test]
pub fn leading_parent() {
    let expected = {
        let cur_dir = current_dir().unwrap();
        let dir = cur_dir.components().last().unwrap().as_os_str();
        let mut p = PathBuf::from("..");
        p.push(dir);
        p.push(test_dir!());
        p
    };
    assert_eq!(expected, normalize_path(&expected).unwrap());
}

#[cfg(target_os = "linux")]
#[test]
pub fn sym_parent() {
    let expected = test_dir!("sym-a-b/../b/c");
    let actual = normalize_path(test_dir!("sym-a-b/../b/c/../../b/c"));
    assert!(actual.is_ok());
    assert_eq!(expected, actual.unwrap());
}
