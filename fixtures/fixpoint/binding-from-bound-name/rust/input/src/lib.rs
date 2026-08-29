pub struct Store;

impl Store {
    pub fn upsert(&self) {}
}

pub fn drive(first: &Store) {
    let second = first;
    let third = second;
    third.upsert();
}
