pub struct Store;

impl Store {
    pub fn upsert(&self) {}
}

pub struct Cache;

impl Cache {
    pub fn upsert(&self) {}
}

pub fn narrow(handle: &Store) {
    handle.upsert();
}

pub fn other(handle: &Cache) {
    handle.upsert();
}
