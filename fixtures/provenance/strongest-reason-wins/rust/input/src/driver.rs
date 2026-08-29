use crate::store::Store;

pub fn typed(store: &Store) {
    store.upsert();
}

pub fn untyped() {
    let handle = external_thing();
    handle.upsert();
}
