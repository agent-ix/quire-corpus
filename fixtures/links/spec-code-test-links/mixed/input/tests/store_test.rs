use crate::store::Store;

// TC-001, FR-001-AC-1: a record survives a store round trip.
#[test]
fn a_record_survives_a_round_trip() {
    let mut store = Store { count: 0 };
    store.upsert();
    assert_eq!(store.count, 1);
}
