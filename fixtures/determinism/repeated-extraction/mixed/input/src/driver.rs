use crate::store::Store;

pub fn drive(store: &Store) {
    store.upsert();
}
