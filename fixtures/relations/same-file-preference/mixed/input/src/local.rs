pub struct Store;

impl Store {
    pub fn flush(&self) {}
}

pub fn local_driver(store: &Store) {
    store.flush();
}
