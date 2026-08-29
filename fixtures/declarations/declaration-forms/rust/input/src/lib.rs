pub mod inner {
    pub fn nested() {}
}

pub struct Store {
    pub count: u32,
}

pub enum Mode {
    On,
    Off,
}

pub trait Persist {
    fn save(&self);
}

pub type Handle = u32;

impl Store {
    pub fn upsert(&self) {}
}

pub fn free_function() {}
