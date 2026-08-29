use crate::barrel::Store;

pub fn drive(store: &Store) {
    store.upsert();
}
