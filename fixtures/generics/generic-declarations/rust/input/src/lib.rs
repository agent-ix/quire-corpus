pub trait Persist {
    fn save(&self);
}

pub struct Runner<T> {
    pub inner: T,
}

impl<T: Persist> Runner<T> {
    pub fn run(&self) {}
}

pub struct Store;

impl Persist for Store {
    fn save(&self) {}
}
