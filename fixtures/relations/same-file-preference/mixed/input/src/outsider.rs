pub struct Store;

impl Store {
    pub fn flush(&self) {}
}

pub fn outside_driver(store: &Store) {
    store.flush();
}
