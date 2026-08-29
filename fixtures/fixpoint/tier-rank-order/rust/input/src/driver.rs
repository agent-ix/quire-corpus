use crate::cache::Cache;

pub fn drive(store: &Store) {
    store.upsert();
}
