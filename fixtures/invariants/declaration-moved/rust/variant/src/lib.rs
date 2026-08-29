// A comment added above everything, which moves every declaration down.
// Nothing else changes.

pub struct Store;

impl Store {
    pub fn upsert(&self) {}
}

pub fn drive(store: &Store) {
    store.upsert();
}
