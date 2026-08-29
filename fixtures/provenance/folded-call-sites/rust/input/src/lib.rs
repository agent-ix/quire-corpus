pub struct Store;

impl Store {
    pub fn upsert(&self) {}
}

pub fn first(store: &Store) {
    store.upsert();
}

pub fn second(store: &Store) {
    store.upsert();
    store.upsert();
}
