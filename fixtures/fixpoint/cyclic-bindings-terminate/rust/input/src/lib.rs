pub struct Store;

impl Store {
    pub fn upsert(&self) {}
}

pub struct Cache;

impl Cache {
    pub fn upsert(&self) {}
}

pub fn drive() {
    let a = b;
    let b = c;
    let c = a;
    a.upsert();
}
