pub trait Persist {
    fn save(&self);
}

pub struct Store;

impl Persist for Store {
    fn save(&self) {}
}
