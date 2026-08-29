pub struct Store;

impl Store {
    pub fn upsert(&self) {}
}

pub fn make() -> Store {
    Store
}

pub fn drive() {
    let store = make();
    store.upsert();
}
