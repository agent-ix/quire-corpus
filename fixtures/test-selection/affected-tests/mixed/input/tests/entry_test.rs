use crate::entry::entry;

// TC-101: the entry point returns the leaf's value.
#[test]
fn entry_returns_the_leaf_value() {
    assert_eq!(entry(), 1);
}
