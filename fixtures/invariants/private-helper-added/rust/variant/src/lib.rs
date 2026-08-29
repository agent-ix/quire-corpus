pub struct Store;

impl Store {
    pub fn upsert(&self) {}
}

pub fn drive(store: &Store) {
    store.upsert();
}

fn private_helper() {}
