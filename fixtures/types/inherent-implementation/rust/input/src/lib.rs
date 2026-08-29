pub trait Persist {
    fn save(&self);
}

pub struct Store;

impl Store {
    pub fn save(&self) {}
}
