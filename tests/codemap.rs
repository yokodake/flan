use flan::codemap;
use flan::codemap::Pos;

static no_nl_128: &str = "aaaabbbbccccddddaaaabbbbccccdddd";
static two_nl_128: &str = "aaa\nbbbbc\nccddddaaaabbbbccccdddd";
static end_nl_128: &str = "aaaabbbbccccddddaaaabbbbccccddd\n";

#[test]
fn no_slow() {
    let mut lines = Vec::new();
    codemap::line_pos_slow(no_nl_128, no_nl_128.len(), Pos(0), &mut lines);
    // line_pos_* does not add first line position
    assert_eq!(lines, vec![]);
}

#[test]
fn two_slow() {
    let mut lines = Vec::new();
    codemap::line_pos_slow(two_nl_128, two_nl_128.len(), Pos(0), &mut lines);
    assert_eq!(
        lines,
        vec![4, 10]
            .iter()
            .map(|i: &u64| Pos(*i))
            .collect::<Vec<_>>()
    );
}
#[test]
fn end_slow() {
    let mut lines = Vec::new();
    codemap::line_pos_slow(end_nl_128, end_nl_128.len(), Pos(0), &mut lines);
    // line_pos_* does not delete redundant eof position
    assert_eq!(lines, vec![Pos::from(end_nl_128.len())]);
}
#[test]
fn all_sse2() {
    let mut l0 = Vec::new();
    let mut l1 = Vec::new();
    let mut l2 = Vec::new();
    let mut k0 = Vec::new();
    let mut k1 = Vec::new();
    let mut k2 = Vec::new();
    codemap::line_pos_slow(no_nl_128, no_nl_128.len(), Pos(0), &mut l0);
    codemap::line_pos_slow(two_nl_128, two_nl_128.len(), Pos(0), &mut l1);
    codemap::line_pos_slow(end_nl_128, end_nl_128.len(), Pos(0), &mut l2);
    unsafe {
        codemap::line_pos_sse2(no_nl_128, Pos(0), &mut k0);
        codemap::line_pos_sse2(two_nl_128, Pos(0), &mut k1);
        codemap::line_pos_sse2(end_nl_128, Pos(0), &mut k2);
    }
    assert_eq!(vec![l0, l1, l2], vec![k0, k1, k2]);
}
#[test]
fn all_avx2() {
    let mut l0 = Vec::new();
    let mut l1 = Vec::new();
    let mut l2 = Vec::new();
    let mut k0 = Vec::new();
    let mut k1 = Vec::new();
    let mut k2 = Vec::new();
    codemap::line_pos_slow(no_nl_128, no_nl_128.len(), Pos(0), &mut l0);
    codemap::line_pos_slow(two_nl_128, two_nl_128.len(), Pos(0), &mut l1);
    codemap::line_pos_slow(end_nl_128, end_nl_128.len(), Pos(0), &mut l2);
    unsafe {
        codemap::line_pos_avx2(no_nl_128, Pos(0), &mut k0);
        codemap::line_pos_avx2(two_nl_128, Pos(0), &mut k1);
        codemap::line_pos_avx2(end_nl_128, Pos(0), &mut k2);
    }
    assert_eq!(vec![l0, l1, l2], vec![k0, k1, k2]);
}
