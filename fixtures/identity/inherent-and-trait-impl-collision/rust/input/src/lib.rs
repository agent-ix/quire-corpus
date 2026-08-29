pub trait Persist {
    fn save(&self);
}

pub struct Store;

impl Store {
    pub fn save(&self) {}
}

impl Persist for Store {
    fn save(&self) {
        Store::save(self);
    }
}
