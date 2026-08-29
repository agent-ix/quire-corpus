use crate::decoy::decoy_entry;

// TC-102: the decoy chain is independent of the entry chain.
#[test]
fn decoy_is_independent() {
    assert_eq!(decoy_entry(), 2);
}
